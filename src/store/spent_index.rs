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

use std::sync::atomic::{AtomicU64,Ordering};


use config;

use store::flatfileset::FlatFileSet;
use store::RecordPtr;


const MB:                 u64 = 1024 * 1024;
const FILE_SIZE:          u64 = 16 * 1024 * MB ;
const MAX_CONTENT_SIZE:   u64 = FILE_SIZE - 10 * MB ;

const VEC_SIZE:         usize = 500_000_000;

/// Index to lookup fileptr's from hashes
///
/// Internally uses fileset
pub struct SpentIndex {

    #[allow(dead_code)]
    fileset:      FlatFileSet<RecordPtr>,

    bitvector:    &'static [AtomicU64]

}

unsafe impl Sync for SpentIndex {}

impl SpentIndex
{
    /// Opens the hash_index at the location given in the config
    ///
    /// Creates a new fileset if needed
    pub fn new(cfg: &config::Config) -> SpentIndex {
        let dir = &cfg.root.clone().join("spent_index");

        let mut fileset = FlatFileSet::new(
            dir, "si-", FILE_SIZE, MAX_CONTENT_SIZE);

        let bitvector = fileset.read_mut_slice(RecordPtr::new(0), VEC_SIZE);
        SpentIndex {
            fileset: fileset,
            bitvector: bitvector
        }
    }




    /// Tests if the given hash exists.
    pub fn exists(&self, hash: u64) -> bool {
        let idx =  (hash >> 6) as usize;
        (self.bitvector[idx].load(Ordering::Relaxed) & (1 << (hash & 0x3F))) > 0
    }



    /// Stores a recordhash
    pub fn set(&mut self, hash: u64)  {
        loop {
            let idx = (hash >> 6) as usize;
            let org = self.bitvector[idx].load(Ordering::Acquire);
            let new = org | (1 << (hash & 0x3F));

            if self.bitvector[idx].compare_and_swap(org, new, Ordering::Release) == org {
                break;
            }
        }
    }
}





#[cfg(test)]
mod tests {

    extern crate rand;
    use std::collections::HashSet;

    use super::*;


    #[test]
    fn test_seq() {

        let mut idx: SpentIndex = SpentIndex::new(& test_cfg!() );


        let mut set = HashSet::new();

        for n in 0..60000_u64 {
            if n % 3 == 0 {
                set.insert(n);
                idx.set(n);
            }

        }

        for n in 0..60000 {
            if n % 3 == 0 {
                assert!( idx.exists(n));
            }
            else {
                assert!( !idx.exists(n));
            }
        }
    }
}