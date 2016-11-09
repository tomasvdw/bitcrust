

/// Index to lookup fileptr's from hashes
///

use std::{fs};

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

    pub fn set_ptr(&self, hash: Hash32, ptr: FilePtr) -> Option<FilePtr> {
        unimplemented!();
    }

    /// Retrieves the fileptr of the given hash
    pub fn get_tx_ptr(&self, hash: Hash32) -> Option<FilePtr> {
        None
    }
}

struct Node {
    hash: [u8;32],
    prev: FilePtr,
    next: FilePtr,
    this: Leaf,
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