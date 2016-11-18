

//! Index that maps hashes to fileptrs
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
//!  The block is found. The pointer points to a record in the spent-tree; _that_ pointer points to the blockheader
//!  in block_content
//!
//! * a set of fileptr pointing to guard blocks
//!  The block cannot be found, but the blocks pointed to by these ptrs are having the given hash as previous block;
//!  they are "expecting" this block
//!
//!

use std::{mem};
use std::cmp::{Ord,Ordering};

use config;
use hash::*;

use store::fileptr::FilePtr;
use store::flatfileset::FlatFileSet;



const FILE_SIZE:          u32 = 1 * 1024*1024*1024;
const MAX_CONTENT_SIZE:   u32 = FILE_SIZE - 10 * 1024*1024;

const HASH_ROOT_COUNT:  usize = 256*256*256;

/// Index to lookup fileptr's from hashes
///
/// Internally uses fileset
pub struct HashIndex {

    fileset:    FlatFileSet,

    hash_index_root:    &'static [FilePtr; HASH_ROOT_COUNT]

}


/// The result used internally when searched for hash
enum FindNodeResult {

    /// Tha hash is found and the location is returned
    Found(&'static Node),

    /// The hash is not found; the location where the node should be inserted
    /// is returned
    NotFound(&'static FilePtr)
}

/// Structures as stored in the fileset
#[derive(Debug)]
struct Node {
    hash: [u8; 32],
    prev: FilePtr,  // to Node
    next: FilePtr,  // to Node
    leaf: FilePtr,  // to Leaf
}

/// Leaf of the binary tree
struct Leaf {
    value: FilePtr, /// to Data file
    next:  FilePtr, // to Leaf
}


impl Leaf {
    fn new(value: FilePtr) -> Self {
        Leaf {
            value: value,
            next: FilePtr::null()
        }
    }
}


impl Node {
    fn new(hash: Hash32, leaf_ptr: FilePtr) -> Self {
        Node {
            hash: *hash.0,
            prev: FilePtr::null(),
            next: FilePtr::null(),
            leaf: leaf_ptr
        }
    }
}

impl HashIndex {

    /// Opens the hash_index at the location given in the config
    ///
    /// Creates a new fileset if needed
    pub fn new(cfg: &config::Config) -> HashIndex {
        let dir = &cfg.root.clone().join("hash_index");

        let is_new = !dir.exists();

        let mut fileset = FlatFileSet::new(
            dir, "hi-", FILE_SIZE, MAX_CONTENT_SIZE);

        let hash_root_fileptr = if is_new {

            // allocate space for root hash table
            fileset.alloc_write_space(mem::size_of::<[FilePtr; HASH_ROOT_COUNT]>() as u32)
        }
        else {
            // hash root must have been the first thing written
            FilePtr::new(0, 0x10)
        };

        // and keep a reference to it
        let hash_root_ref: &'static [FilePtr; HASH_ROOT_COUNT]
            = fileset.read_fixed(hash_root_fileptr);

        HashIndex {
            fileset: fileset,
            hash_index_root: hash_root_ref
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

    /// Collects all the values stored at the given node
    fn collect_node_values(&mut self, node: &Node) -> Vec<FilePtr> {

        let mut result : Vec<FilePtr> = Vec::new();
        let mut leaf_ptr = node.leaf;

        while !leaf_ptr.is_null() {
            let leaf: &Leaf = self.fileset.read_fixed(leaf_ptr);
            result.push(leaf.value);

            leaf_ptr = leaf.next;
        }
        result
    }

    // Finds the node containing the hash, or the location the hash should be inserted
    fn find_node(&mut self, hash: Hash32) -> FindNodeResult {

        // use the first 24-bit as index in the root hash table
        let mut ptr = &self.hash_index_root[HashIndex::hash_to_index(hash)];

        // from there, we follow the binary tree
        while !ptr.is_null() {
            let node: &Node = self.fileset.read_fixed(*ptr);

            ptr = match hash.0.cmp(&node.hash) {
                Ordering::Less    => &node.prev,
                Ordering::Greater => &node.next,
                Ordering::Equal   => return FindNodeResult::Found(node)
            };
        }

        FindNodeResult::NotFound(ptr)
    }


    /// Retrieves the fileptr's of the given hash
    pub fn get_ptr(&mut self, hash: Hash32) -> Vec<FilePtr> {

        match self.find_node(hash) {
            FindNodeResult::NotFound(_) => {
                Vec::new()
            },
            FindNodeResult::Found(node) => {

                self.collect_node_values(node)
            }
        }
    }

    /// Stores a fileptr at the given hash
    ///
    /// This will bail out atomically (do a noop) if there are inputs stored at the hash,
    /// that are not among the passed `verified_inputs`.
    ///
    /// This way, inputs stores at a hash serve as guards that need to be verified before
    /// the transaction can be stored.
    pub fn set_tx_ptr(&mut self, hash: Hash32, ptr_tx: FilePtr, verified_inputs: Vec<FilePtr>) -> bool {

        // this loops through retries when the CAS operation fails
        loop {
            match self.find_node(hash) {
                FindNodeResult::NotFound(ptr) => {

                    // create and write a leaf;
                    let new_leaf     = Leaf::new(ptr_tx);
                    let new_leaf_ptr = self.fileset.write_fixed(&new_leaf);

                    // create and write a node holding the leaf
                    let new_node     = Node::new(hash, new_leaf_ptr);
                    let new_node_ptr = self.fileset.write_fixed(&new_node);

                    // then atomically update the pointer
                    if ptr.atomic_replace(FilePtr::null(), new_node_ptr) {
                        return true;
                    }

                },
                FindNodeResult::Found(node) => {

                    let first_value_ptr = node.leaf;

                    // check if there is anything waiting that is not supplied in verified_inputs
                    if !self
                        .collect_node_values(node)
                        .into_iter()
                        .any(|val| verified_inputs.contains(&val)) {

                        return false;
                    }

                    // We don't need to keep the input-pointers
                    // Replace all with a new leaf
                    let new_leaf = Leaf::new(ptr_tx);
                    let new_leaf_ptr = self.fileset.write_fixed(&new_leaf);

                    // then atomically update the pointer
                    if node.leaf.atomic_replace(first_value_ptr, new_leaf_ptr) {
                        return true;
                    }

                }
            };
        }
    }


    /// Retrieves the fileptr for a tx to verify the output with an input
    ///
    /// If the tx doesn't exist, the given input_ptr is added atomically to block further adds
    pub fn get_tx_for_output(&mut self, hash: Hash32, input_ptr: FilePtr) -> Option<FilePtr> {

        // this loops through retries when the CAS operation fails
        loop {
            match self.find_node(hash) {

                FindNodeResult::NotFound(ptr) => {

                    // The transaction doesn't exist; we insert input_ptr instead

                    // create and write a leaf;
                    let new_leaf = Leaf::new(input_ptr);
                    let new_leaf_ptr = self.fileset.write_fixed(&new_leaf);

                    // create and write a node holding the leaf
                    let new_node = Node::new(hash, new_leaf_ptr);
                    let new_node_ptr = self.fileset.write_fixed(&new_node);

                    // then atomically update the pointer
                    if ptr.atomic_replace(FilePtr::null(), new_node_ptr) {
                        return None;
                    }
                },

                FindNodeResult::Found(node) => {

                    // load first leaf
                    let first_value_ptr = node.leaf;
                    let leaf: &Leaf = self.fileset.read_fixed(first_value_ptr);

                    if leaf.value.is_transaction() {
                        return Some(leaf.value);
                    }

                    // create a new leaf, pointing to the previous one
                    let new_leaf     = Leaf { value: input_ptr, next: node.leaf };
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
    use super::{Node,Leaf};
    use std::mem;

    extern crate tempdir;
    extern crate rand;
    use std::path::PathBuf;

    use std::thread;

    use store::fileptr::FilePtr;
    use store::flatfileset::FlatFileSet;
    use super::*;
    use self::rand::Rng;
    use config;
    use hash::Hash32Buf;

    #[test]
    fn test_size_of_node() {
        assert_eq!(mem::size_of::<Node>(), 56);
        assert_eq!(mem::size_of::<Leaf>(), 16);
    }

    #[test]
    fn test_seq() {

        const DATA_SIZE: u32 = 100000;
        const THREADS: usize = 100;
        const LOOPS: usize = 100;

        let dir = tempdir::TempDir::new("test1").unwrap();
        let path = PathBuf::from(dir.path());
        let cfg = config::Config { root: path.clone() };

        let idx = HashIndex::new(&cfg);

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

                let mut idx = HashIndex::new(&cfg);

                for _ in 0..LOOPS {

                    let tx = FilePtr::new(0, rng.gen_range(10, DATA_SIZE));
                    let tx_hash = hash(tx.file_pos());

                    let found_txs = idx.get_ptr(tx_hash.as_ref());

                    if !found_txs.is_empty() {

                        // check that x is either a tx or a set of inputs
                        if found_txs.clone().into_iter().all(|tx| tx.is_transaction() ) && found_txs.len() == 1 {
                            assert_eq!(found_txs[0].file_pos(), tx.file_pos());
                            //println!("Found 1 transaction");
                            continue;
                        }
                        if found_txs.clone().into_iter().all(|tx| tx.is_input() )  {
                            //println!("Found {} inputs", found_txs.len());
                            continue;
                        }
                        //panic!("Expected only 1 tx or 1..n inputs");
                    }
                    else {
                        if tx.file_pos() > 2 {

                            // some messy ops for messy tests

                            let output_tx1_ptr = FilePtr::new(0, tx.file_pos() as u32 -1);
                            let output_hash = hash(output_tx1_ptr.file_pos());

                            let input_ptr = FilePtr::new(0, tx.file_pos() as u32).as_input(1);

                            let output_tx1 = idx.get_tx_for_output(output_hash.as_ref(), input_ptr);

                            if let Some(x) = output_tx1 {
                                assert_eq!(x.is_transaction(), true);

                                // script validation goes here
                            }

                            idx.set_tx_ptr(tx_hash.as_ref(), tx, vec![input_ptr]);

                        }
                        else {
                            idx.set_tx_ptr(tx_hash.as_ref(), tx, Vec::new());
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