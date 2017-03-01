//! Index that stores transactions and spent outputs
//!
//! This serves as a broom-wagon for the spent-tree.
//! After X blocks, outputs and transactions are stored here,
//! and when outputs aren't found in the spent tree for X blocks,
//! they are searched here.
//!
//! The data-structure here is similar to hash-index:
//! a large root hash-table with each element pointing to an unbalanced binary tree
//!

use std::{mem};
use std::sync::atomic;
use std::cmp::{Ord,Ordering};

use config;
use hash::*;

use store::hash_index::IndexPtr;
use store::FlatFilePtr;
use store::flatfileset::FlatFileSet;

use store::Record;


const FILE_SIZE:          u64 = 1 * 1024*1024*1024;
const MAX_CONTENT_SIZE:   u64 = FILE_SIZE - 10 * 1024*1024;

const ROOT_COUNT:       usize = 256*256*256;



/// Index to lookup fileptr's from hashes
///
/// Internally uses fileset
pub struct SpentIndex {

    fileset:    FlatFileSet<IndexPtr>,

    index_root: &'static [IndexPtr; ROOT_COUNT],

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
struct Node  {
    hash: [u8;8],
    prev: IndexPtr,  // to Node
    next: IndexPtr,  // to Node

}



impl Node {
    fn new(hash: [u8;8]) -> Self {
        Node {
            hash: hash,
            prev: IndexPtr::null(),
            next: IndexPtr::null()
        }
    }
}


// Returns the first 24-bits of the hash
//
// This is the index into the root-hash table
fn hash_to_index(hash: [u8;8]) -> usize {

    (hash[0] as usize) |
        (hash[1] as usize) << 8  |
        (hash[2] as usize) << 16

}

unsafe impl Sync for SpentIndex {}

impl SpentIndex
{

    /// Opens the hash_index at the location given in the config
    ///
    /// Creates a new fileset if needed
    pub fn new(cfg: &config::Config) -> SpentIndex {
        let dir = &cfg.root.clone().join("spent_index");

        let is_new = !dir.exists();

        let mut fileset = FlatFileSet::new(
            dir, "si-", FILE_SIZE, MAX_CONTENT_SIZE);

        let hash_root_fileptr = if is_new {

            // allocate space for root hash table
            fileset.alloc_write_space(mem::size_of::<[IndexPtr; ROOT_COUNT]>() as u64)
        }
        else {
            // hash root must have been the first thing written
            IndexPtr::new(0, super::flatfile::INITIAL_WRITEPOS)
        };

        // and keep a reference to it
        let hash_root_ref: &'static [IndexPtr; ROOT_COUNT]
            = fileset.read_fixed(hash_root_fileptr);

        SpentIndex {
            fileset: fileset,
            index_root: hash_root_ref,

        }
    }



    // Finds the node containing the hash, or the location the hash should be inserted
    fn find_node(&self, hash: [u8;8]) -> FindNodeResult {

        // use the first 24-bit as index in the root hash table
        let mut ptr = &self.index_root[hash_to_index(hash)];

        // from there, we follow the binary tree
        while !ptr.is_null() {
            let node: &Node = self.fileset.read_fixed_readonly(*ptr);

            ptr = match hash.cmp(&node.hash) {
                Ordering::Less    => &node.prev,
                Ordering::Greater => &node.next,
                Ordering::Equal   => return FindNodeResult::Found(node)
            };
        }

        FindNodeResult::NotFound(ptr)
    }


    /// Tests if the given hash exists.
    pub fn exists(&self, hash: [u8;8]) -> bool {

        match self.find_node(hash) {
            FindNodeResult::NotFound(_) => false,
            FindNodeResult::Found(_) => true
        }
    }



    /// Stores a recordhash
    pub fn set(&mut self, hash: [u8;8])  {


        // this loops through retries when the CAS operation fails
        loop {
            match self.find_node(hash) {
                FindNodeResult::NotFound(target) => {

                    // create and write a node holding the leaf
                    let new_node     = Node::new(hash);
                    let new_node_ptr = self.fileset.write_fixed(&new_node);

                    // then atomically update the pointer
                    if target.atomic_replace(IndexPtr::null(), new_node_ptr) {
                        break;
                    }

                },
                FindNodeResult::Found(node) => {

                    // this can happen by a concurrent block;
                    // no problem
                    break;
                }
            };
        }


    }
}





#[cfg(test)]
mod tests {
    use super::{Node};
    use std::mem;

    extern crate rand;
    use std::path::PathBuf;
    use std::collections::HashSet;

    use std::thread;

    use super::*;
    use self::rand::Rng;
    use config;
    use hash::Hash32Buf;
    use store::TxPtr;
    use store::flatfileset::FlatFilePtr;

    #[test]
    fn test_size_of_node() {
        assert_eq!(mem::size_of::<Node>(), 24);

    }

    #[test]
    fn test_seq() {

        const DATA_SIZE: u32 = 100000;
        const THREADS: usize = 1;
        const LOOPS: usize = 10000;

        let mut idx: SpentIndex = SpentIndex::new(& test_cfg!() );


        let mut set = HashSet::new();

        for n in 0..255_u8 {
            for m in 0..256_u8 {
                let elm = [n,1,2,m,1,2,3,4];
                set.insert(elm);
                idx.set(elm);
            }
        }

        for n in 0..255_u8 {
            for m in 0..256_u8 {
                let elm = [n,1,2,m,1,2,3,4];

                assert!( idx.exists([n,1,2,m,1,2,3,4]));
                assert!(!idx.exists([n,2,2,m,1,2,3,4]));
                assert!(!idx.exists([n,1,2,m,1,2,3,3]));
            }
        }





    }
}