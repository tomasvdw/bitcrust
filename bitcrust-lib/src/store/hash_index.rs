

/// Index to lookup fileptr's from hashes
///
/// This is used for transactions and block-headers
///
/// Specific atomic operations are supported to add a transaction only
/// if all everything verified
///

use std::{fs,mem};
use std::cmp::{Ord,Ordering};

use config;
use hash::*;

use store::fileptr::FilePtr;
use store::flatfileset::FlatFileSet;



const FILE_SIZE:          u32 = 1 * 1024*1024*1024;
const MAX_CONTENT_SIZE:   u32 = FILE_SIZE - 10 * 1024*1024;

const HASH_ROOT_COUNT:  usize = 16 * 1024*1024;

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


impl HashIndex {
    pub fn new(cfg: &config::Config) -> HashIndex {
        let dir = &cfg.root.clone().join("hash_index");

        // recreate dir
        fs::remove_dir_all(dir);
        fs::create_dir_all(dir);


        let mut fileset = FlatFileSet::new(
            dir, "hi-", FILE_SIZE, MAX_CONTENT_SIZE);

        // allocate space for root hash table
        let hash_root_fileptr = fileset.alloc_write_space(64 * 1024 * 1024);

        // and keep a reference to it
        let hash_root_ref: &'static [FilePtr; HASH_ROOT_COUNT]
            = fileset.read_fixed(hash_root_fileptr);

        HashIndex {
            fileset: fileset,
            hash_index_root: hash_root_ref
        }
    }

    // Returns the first 24-bits of the hash
    fn hash_to_index(hash: Hash32) -> usize {
        (hash.0[0] as usize) |
            (hash.0[1] as usize) << 8  |
            (hash.0[2] as usize) << 16

    }


    // Finds the node containing the hash, or the location the hash should be inserted
    fn find_node(&mut self, hash: Hash32) -> FindNodeResult {

        let mut ptr = &self.hash_index_root[HashIndex::hash_to_index(hash)];

        while !ptr.is_null() {
            let node: &Node = self.fileset.read_fixed(*ptr);

            ptr = match hash.0.cmp(&node.hash) {
                Ordering::Less => &node.prev,
                Ordering::Greater => &node.next,
                Ordering::Equal => return FindNodeResult::Found(node)
            };
        }

        FindNodeResult::NotFound(ptr)
    }

    /// Stores a fileptr at the given hash
    ///
    /// This will bail out atomically (do a noop) if there are inputs stored at the hash,
    /// that are not among the passed `verified_inputs`.
    ///
    /// This way, inputs stores at a hash serve as guards that need to be verified before
    /// the transaction can be stored.
    pub fn set_tx_ptr(&mut self, hash: Hash32, ptr_tx: FilePtr, verified_inputs: Vec<FilePtr>) -> Option<FilePtr> {

        // this loops through retries when the CAS operation fails
        loop {
            match self.find_node(hash) {
                FindNodeResult::NotFound(ptr) => {
                    // create and write a node;
                    // then a pointer to to
                    unimplemented!();
                },
                FindNodeResult::Found(node) => {

                    // check the content

                    unimplemented!();
                }
            };
        }
        unimplemented!();
    }

    /// Retrieves the fileptr's of the given hash
    pub fn get_ptr(&self, hash: Hash32) -> Vec<FilePtr> {
        unimplemented!();
    }

    /// Retrieves the fileptr for a tx to verify the output with an input
    ///
    /// If the tx doesn't exist, the input_ptr is added atomically to block further adds
    pub fn get_tx_for_output(&self, hash: Hash32, input_ptr: FilePtr) -> Option<FilePtr> {
        unimplemented!();
    }
}

struct Node {
    hash: [u8;32],
    prev: FilePtr,
    next: FilePtr,
    this: Leaf,
}

impl Node {
    fn new(hash: Hash32, value: FilePtr) -> Node {
        let p: &[u8; 32] = unsafe { mem::transmute( hash.0.as_ptr()) };

        Node {
            hash: *p,
            prev: FilePtr::null(),
            next: FilePtr::null(),
            this: Leaf {
                value: value,
                next: FilePtr::null()
            }
        }
    }
}

struct Leaf {
    value: FilePtr,
    next:  FilePtr,
}

#[cfg(test)]
mod tests {
    use super::{Node,Leaf};
    use std::mem;

    #[test]
    fn test_size_of_node() {
        assert_eq!(mem::size_of::<Node>(), 64);
        assert_eq!(mem::size_of::<Leaf>(), 16);
    }
}