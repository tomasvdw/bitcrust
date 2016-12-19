//! Merkle tree implementation
//!
//! Can be calculated "rolling"; it only needs to keep .5 hash per level of the tree


use std::cell::RefCell;
use itertools::fold;

use hash::*;

pub struct MerkleTree {

    hashes: Vec<Hash32Buf>
}

/// This halves the merkle tree leaves, taking it one level up
///
/// Calls itself recursively until one is left
fn shrink_merkle_tree(hashes: Vec<Hash32Buf>) -> Vec<Hash32Buf> {

    if hashes.len() == 1 {
        return hashes;
    }

    let count = (hashes.len() + 1 ) / 2;
    let mut result = Vec::with_capacity(count);


    for n in 0..count {
        let ref first = hashes[n*2];
        let ref second = hashes.get(n*2+1).unwrap_or(&first);

        result.push(Hash32Buf::double_sha256_from_pair(first.as_ref(),second.as_ref()))
    }

    shrink_merkle_tree(result)
}

impl MerkleTree {
    pub fn new() -> MerkleTree {

        MerkleTree {
            hashes: vec![]
        }
    }

    pub fn add_hash(&mut self, hash: Hash32) {

        self.hashes.push(Hash32Buf::from_slice(hash.0));
    }


    pub fn get_merkle_root(&mut self) -> Hash32Buf {

        assert!(!self.hashes.is_empty());

        shrink_merkle_tree(self.hashes.clone())[0]
    }
}