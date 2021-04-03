//! Index that stores transactions and spend outputs
//!
//! This serves as a broom-wagon for the spend-tree.
//! After X blocks, outputs and transactions are stored here,
//! and when outputs aren't found in the spend tree for X blocks,
//! they are searched here.
//!
//! The data-structure here is a simple bit-index where each transaction and each spend-output
//! are given a unique bit which is set if the given transaction or spend exists

use std::sync::atomic::{AtomicU64,Ordering};

use config;
use store::flatfileset::FlatFileSet;
use store::RecordPtr;


const MB:                 u64 = 1024 * 1024;
const FILE_SIZE:          u64 = 16 * 1024 * MB ;
const MAX_CONTENT_SIZE:   u64 = FILE_SIZE - 10 * MB ;

// TODO; make this dynamic using fileset continuation;
// this isn't hastily needed as the OS does not actually allocate
// all the space; (compare ls with du).
const VEC_SIZE:         usize = 500_000_000;

/// Index to lookup spends
///
/// Internally uses fileset
///
pub struct SpendIndex {

    #[allow(dead_code)]
    fileset:      FlatFileSet<RecordPtr>,

    bitvector:    &'static [AtomicU64]

}

unsafe impl Sync for SpendIndex {}

impl SpendIndex
{
    /// Opens the spend_index at the location given in the config
    ///
    /// Creates a new fileset if needed
    pub fn new(cfg: &config::Config) -> SpendIndex {
        let dir = &cfg.root.clone().join("spend-index");

        let mut fileset = FlatFileSet::new(
            dir, "si-", FILE_SIZE, MAX_CONTENT_SIZE);

        let bitvector = fileset.read_mut_slice(RecordPtr::new(0), VEC_SIZE);
        SpendIndex {
            fileset: fileset,
            bitvector: bitvector
        }
    }




    /// Tests if the given hash exists.
    pub fn exists(&self, hash: u64) -> bool {

        let idx =  (hash >> 6) as usize;
        (self.bitvector[idx].load(Ordering::Relaxed) & (1 << (hash & 0x3F))) > 0
    }


    /// Stores a record hash; this should uniquely identify an output or a transaction
    pub fn set(&mut self, hash: u64)  {

        // CAS-loop
        loop {
            let idx = (hash >> 6) as usize;
            let org = self.bitvector[idx].load(Ordering::Acquire);
            let new = org | (1 << (hash & 0x3F));

            if self.bitvector[idx].compare_exchange(org, new, Ordering::Release, Ordering::Release) == Ok(org) {
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

        let mut idx: SpendIndex = SpendIndex::new(& test_cfg!() );


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