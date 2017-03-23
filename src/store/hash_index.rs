

//! Index that maps hashes to content pointers
//!
//! This is used for transactions & blockheaders; the values found for a hash can be:
//!
//!  * a single fileptr pointing to a transaction
//!  The transaction can be found at the ptr in block_content and is fully validated
//!
//!  * a set of fileptr pointing to inputs
//!  Transaction cannot be found, but these inputs need this transaction
//!  and are already assumed to be valid. The transaction may not be inserted before these
//!  scripts are checked
//!
//!  * a single fileptr pointing to a pointer to blockheader
//!  The block is found. The pointer points to a record in the spend-tree; _that_ pointer points to the blockheader
//!  in block_content
//!
//! * a set of fileptr pointing to guard blocks
//!  The block cannot be found, but the blocks pointed to by these ptrs are having the given hash as previous block;
//!  they are "expecting" this block, and should be appended when this block comes in
//!
//! The implementation is a large root hash table, with colliding keys added to an unbalanced binary tree.
//!
//!  TODO There is probably quite some gain to be made by moving to HAMT instead unbalanced binary trees
//!  for the branches; especially for low-resource uses.


use std::{mem};
use std::sync::atomic;
use std::cmp::{Ord,Ordering};

use config;
use hash::*;

use store::FlatFilePtr;
use store::flatfileset::FlatFileSet;



const FILE_SIZE:          u64 = 1 * 1024*1024*1024;
const MAX_CONTENT_SIZE:   u64 = FILE_SIZE - 10 * 1024*1024;

const HASH_ROOT_COUNT:  usize = 256*256*256;


/// Trait for objects that can be used as a guard
/// This is required for types that are stored in the hash-index
pub trait HashIndexGuard {
    fn is_guard(self) -> bool;
}


/// Index to lookup fileptr's from hashes
///
/// Internally uses fileset
pub struct HashIndex<T : HashIndexGuard + Copy + Clone> {

    fileset:         FlatFileSet<IndexPtr>,

    hash_index_root: &'static [IndexPtr; HASH_ROOT_COUNT],

    phantom:         ::std::marker::PhantomData<T>

}

impl<T : HashIndexGuard + Copy + Clone> Clone for HashIndex<T> {

    // Explicit cloning can be used to allow concurrent access.
    fn clone(&self) -> HashIndex<T> {

        let mut fileset = self.fileset.clone();
        let root = fileset.read_fixed(IndexPtr::new(0, super::flatfile::INITIAL_WRITEPOS));
        HashIndex {

            fileset:         fileset,
            hash_index_root: root,
            phantom:         ::std::marker::PhantomData

        }

    }
}

/// A persistent pointer into the hash-index
#[derive(Debug, Clone, Copy)]
pub struct IndexPtr {
    file_offset: u32,
    file_number: i16,
    zero: u16
}

impl FlatFilePtr for IndexPtr {
    fn new(file_number: i16, file_offset: u64) -> Self {

        IndexPtr {
            file_offset: file_offset as u32,
            file_number: file_number,
            zero: 0  // we must pad with zero to ensure atomic CAS works
        }
    }


    fn get_file_offset(self) -> u64 { self.file_offset as u64 }
    fn get_file_number(self) -> i16 { self.file_number }


}

impl IndexPtr {
    pub fn null() -> Self { IndexPtr::new(0, 0) }

    pub fn is_null(&self) -> bool { self.file_offset == 0 && self.file_number == 0 }


    /// atomically replaces a hash indexptr value with a new_value,
    /// fails if the current value is no longer the value supplied
    pub fn atomic_replace(&self, current_value: IndexPtr, new_value: IndexPtr) -> bool {

        let atomic_self: *mut atomic::AtomicU64 = unsafe { mem::transmute( self ) };

        let prev = unsafe {
            (*atomic_self).compare_and_swap(
                mem::transmute(current_value),
                mem::transmute(new_value),
                atomic::Ordering::Relaxed)
        };

        prev == unsafe { mem::transmute(current_value) }

    }
}


/// The result used internally when searched for hash
enum FindNodeResult {

    /// Tha hash is found and the location is returned
    Found(&'static Node),

    /// The hash is not found; the location where the node should be inserted
    /// is returned
    NotFound(&'static IndexPtr)
}

/// Structures as stored in the fileset
#[derive(Debug)]
struct Node {
    hash: Hash32Buf,
    prev: IndexPtr,  // to Node
    next: IndexPtr,  // to Node
    leaf: IndexPtr,  // to Leaf
}

/// Leaf of the binary tree
/// The supplied Type is the type of the elements that are stored in the tree
struct Leaf<T : HashIndexGuard> {
    value: T, /// to Data file
    next: IndexPtr, // to Leaf
}


impl<T : HashIndexGuard> Leaf<T> {
    fn new(value: T) -> Self {
        Leaf {
            value: value,
            next: IndexPtr::null()
        }
    }
}


impl Node {
    fn new(hash: Hash32, leaf_ptr: IndexPtr) -> Self {
        Node {
            hash: hash.as_buf(),
            prev: IndexPtr::null(),
            next: IndexPtr::null(),
            leaf: leaf_ptr
        }
    }
}


// Returns the first 24-bits of the hash
//
// This is the index into the root-hash table
fn hash_to_index(hash: Hash32) -> usize {

    (hash.0[0] as usize) |
        (hash.0[1] as usize) << 8  |
        (hash.0[2] as usize) << 16

}


impl<T :'static> HashIndex<T>
    where T : HashIndexGuard + PartialEq + Copy + Clone
{

    /// Opens the hash_index at the location given in the config
    ///
    /// Creates a new fileset if needed
    pub fn new(cfg: &config::Config, dir: &str) -> HashIndex<T> {
        let dir = &cfg.root.clone().join(dir);

        let is_new = !dir.exists();

        let mut fileset = FlatFileSet::new(
            dir, "hi-", FILE_SIZE, MAX_CONTENT_SIZE);

        let hash_root_fileptr = if is_new {

            // allocate space for root hash table
            fileset.alloc_write_space(mem::size_of::<[IndexPtr; HASH_ROOT_COUNT]>() as u64)
        }
        else {
            // hash root must have been the first thing written
            IndexPtr::new(0, super::flatfile::INITIAL_WRITEPOS)
        };

        // and keep a reference to it
        let hash_root_ref: &'static [IndexPtr; HASH_ROOT_COUNT]
            = fileset.read_fixed(hash_root_fileptr);

        HashIndex {
            fileset: fileset,
            hash_index_root: hash_root_ref,
            phantom: ::std::marker::PhantomData
        }
    }


    /// Collects all the values stored at the given node
    fn collect_node_values(&mut self, node: &Node) -> Vec<T> {

        let mut result : Vec<T> = Vec::new();
        let mut leaf_ptr = node.leaf;

        while !leaf_ptr.is_null() {
            let v: &T = self.fileset.read_fixed(leaf_ptr);
            let leaf: Leaf<T> = Leaf::new(*v);
            result.push(leaf.value);

            leaf_ptr = leaf.next;
        }
        result
    }

    // Finds the node containing the hash, or the location the hash should be inserted
    fn find_node(&mut self, hash: Hash32) -> FindNodeResult {

        // use the first 24-bit as index in the root hash table
        let mut ptr = &self.hash_index_root[hash_to_index(hash)];

        // from there, we follow the binary tree
        while !ptr.is_null() {
            let node: &Node = self.fileset.read_fixed(*ptr);

            ptr = match hash.0.cmp(&node.hash.as_ref().0) {
                Ordering::Less    => &node.prev,
                Ordering::Greater => &node.next,
                Ordering::Equal   => return FindNodeResult::Found(node)
            };
        }

        FindNodeResult::NotFound(ptr)
    }


    /// Retrieves the fileptr'` of the given hash
    pub fn get(&mut self, hash: Hash32) -> Vec<T> {

        match self.find_node(hash) {
            FindNodeResult::NotFound(_) => {
                Vec::new()
            },
            FindNodeResult::Found(node) => {

                self.collect_node_values(node)
            }
        }
    }

    /// Stores a T at the given hash
    ///
    /// This will bail out atomically (do a noop) if there are existing Ts stored at the hash,
    /// that are not among the passed `verified_ptrs`.
    ///
    /// This way, inputs stores at a hash serve as guards that need to be verified before
    /// the transaction can be stored.
    ///
    /// The force_store flag can be used to overrule this behaviour and store anyway
    ///
    /// Similarly, blockheader_guards need to be connected before a block can be stored
    pub fn set(&mut self, hash: Hash32, store_ptr: T, verified_ptrs: &[T], force_store: bool) -> bool {

        assert!(! store_ptr.is_guard());
        assert!(verified_ptrs.iter().all(|p| p.is_guard()));

        // this loops through retries when the CAS operation fails
        loop {
            match self.find_node(hash) {
                FindNodeResult::NotFound(target) => {

                    // create and write a leaf;
                    let new_leaf     = Leaf::new(store_ptr);
                    let new_leaf_ptr = self.fileset.write_fixed(&new_leaf);

                    // create and write a node holding the leaf
                    let new_node     = Node::new(hash, new_leaf_ptr);
                    let new_node_ptr = self.fileset.write_fixed(&new_node);

                    // then atomically update the pointer
                    if target.atomic_replace(IndexPtr::null(), new_node_ptr) {
                        return true;
                    }

                },
                FindNodeResult::Found(node) => {

                    let first_value_ptr = node.leaf;

                    // check if there is anything waiting that is not supplied in `verified_ptrs`
                    if !force_store &&
                        !self
                        .collect_node_values(node)
                        .into_iter()
                        .any(|val| verified_ptrs.contains(&val)) {

                        return false;
                    }

                    // We don't need to keep the verified-ptrs
                    // Replace all with a new leaf
                    let new_leaf = Leaf::new(store_ptr);
                    let new_leaf_ptr = self.fileset.write_fixed(&new_leaf);

                    // then atomically update the pointer
                    if node.leaf.atomic_replace(first_value_ptr, new_leaf_ptr) {
                        return true;
                    }

                }
            };
        }
    }


    /// Retrieves the fileptr
    ///
    /// If there is no primary ptr (block/tx) for the given hash
    /// the given guard_ptr is added atomically to block further adds
    pub fn get_or_set(&mut self, hash: Hash32, guard_ptr: T) -> Option<T> {

        debug_assert!(guard_ptr.is_guard());
        // this loops through retries when the CAS operation fails
        loop {
            match self.find_node(hash) {

                FindNodeResult::NotFound(ptr) => {

                    // The transaction doesn't exist; we insert guard_ptr instead

                    // create and write a leaf;
                    let new_leaf = Leaf::new(guard_ptr);
                    let new_leaf_ptr = self.fileset.write_fixed(&new_leaf);

                    // create and write a node holding the leaf
                    let new_node = Node::new(hash, new_leaf_ptr);
                    let new_node_ptr = self.fileset.write_fixed(&new_node);

                    // then atomically update the pointer
                    if ptr.atomic_replace(IndexPtr::null(), new_node_ptr) {
                        return None;
                    }
                },

                FindNodeResult::Found(node) => {

                    // load first leaf
                    let first_value_ptr = node.leaf;
                    let val: &T         = self.fileset.read_fixed(first_value_ptr);
                    let leaf: Leaf<T>   = Leaf::new(*val);


                    if !leaf.value.is_guard() {
                        return Some(leaf.value);
                    }

                    // create a new leaf, pointing to the previous one
                    let new_leaf     = Leaf { value: guard_ptr, next: node.leaf };
                    let new_leaf_ptr = self.fileset.write_fixed(&new_leaf);

                    // then atomically update the pointer
                    if node.leaf.atomic_replace(first_value_ptr, new_leaf_ptr) {
                        return None;
                    }

                }
            }
        }

    }
}





#[cfg(test)]
mod tests {
    use super::Node;
    use std::mem;

    extern crate tempdir;
    extern crate rand;
    use std::path::PathBuf;

    use std::thread;

    use super::*;
    use self::rand::Rng;
    use config;
    use hash::Hash32Buf;
    use store::TxPtr;
    use store::flatfileset::FlatFilePtr;

    #[test]
    fn test_size_of_node() {
        assert_eq!(mem::size_of::<Node>(), 56);

    }

    #[test]
    fn test_seq() {

        const DATA_SIZE: u32 = 100000;
        const THREADS: usize = 100;
        const LOOPS: usize = 500;

        let dir = tempdir::TempDir::new("test1").unwrap();
        let path = PathBuf::from(dir.path());
        let cfg = config::Config { root: path.clone() };

        let _idx: HashIndex<TxPtr> = HashIndex::new(& cfg, "test" );

        // We create a little transaction world:
        // The "transactions" are file pointers 1 to DATA_SIZE
        // Function to hash them:
        fn hash(n: usize) -> Hash32Buf {
            let s = format!("{}",n);
            Hash32Buf::double_sha256(s.into_bytes().as_ref())
        }


        let handles: Vec<_> = (0..THREADS).map(|_| {
            let path = path.clone();
            thread::spawn( move | | {
                let mut rng = rand::thread_rng();
                let cfg = config::Config { root: path };

                let mut idx = HashIndex::new(&cfg, "test");

                for _ in 0..LOOPS {

                    let tx = TxPtr::new(0, rng.gen_range(10, DATA_SIZE) as u64);
                    let tx_hash = hash(tx.get_file_offset() as usize);

                    let found_txs = idx.get(tx_hash.as_ref());

                    if !found_txs.is_empty() {

                        // check that x is either a tx or a set of inputs
                        if found_txs.clone().into_iter().all(|tx: TxPtr| !tx.is_guard() ) && found_txs.len() == 1 {
                            assert_eq!(found_txs[0].get_file_offset(), tx.get_file_offset());
                            continue;
                        }
                        if found_txs.clone().into_iter().all(|tx| tx.is_guard() )  {
                            continue;
                        }
                        panic!("Expected only 1 tx or 1..n inputs");
                    }
                    else {
                        if tx.get_file_offset() > 2 {

                            // some messy ops for messy tests

                            let output_tx1_ptr = TxPtr::new(0, tx.get_file_offset()  -1);
                            let output_hash = hash(output_tx1_ptr.get_file_offset() as usize);

                            let input_ptr = TxPtr::new(0, tx.get_file_offset()).to_input(1);

                            let output_tx1 = idx.get_or_set(output_hash.as_ref(), input_ptr);

                            if let Some(x) = output_tx1 {
                                assert_eq!(!x.is_guard(), true);

                                // script validation goes here
                            }

                            idx.set(tx_hash.as_ref(), tx, &[input_ptr], false);

                        }
                        else {
                            idx.set(tx_hash.as_ref(), tx, &[], false);
                        }
                    }

                }

        })
        }).collect();


        for h in handles {
            h.join().unwrap();

        }
    }
}