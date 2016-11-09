//!
//!
//! The store consists of three filesets.
//!
//! # block_content
//!
//! This contains transactions & blockheaders.
//! These are directly written to their flatfileset
//!
//! # hash_index
//!
//!

use std::fs;

mod fileptr;
mod flatfile;
mod flatfileset;
mod index;

mod block_content;
mod hash_index;


use config;


use self::flatfileset::FlatFileSet;

const KB: usize = 1024;
const MB: usize = 1024 * KB;
const GB: usize = 1024 * MB;

const FILE_SIZE: u32         = 1 * GB as u32;
const MAX_CONTENT_SIZE: u32  = FILE_SIZE - 10 * MB as u32 ;




/// This is the accessor to the
pub struct Store {

    pub index: index::Index,

    // Flat files
    pub block_content: block_content::BlockContent,
    pub spent_tree:    FlatFileSet,
    pub hash_index:    hash_index::HashIndex,

}


impl Store {

    pub fn new(cfg: &config::Config) -> Store {

        let dir_st = &cfg.root.clone().join("spent_tree");

        // recreate them
        fs::remove_dir_all(dir_st);
        fs::create_dir_all(dir_st);

        Store {
            index: index::Index::new(cfg),

            block_content: block_content::BlockContent::new(&cfg),
            hash_index:    hash_index::HashIndex::new(&cfg),

            spent_tree: FlatFileSet::new(
                dir_st, "st-", FILE_SIZE, MAX_CONTENT_SIZE),


        }
    }

/*
    /// Sets a value in the index
    pub fn set(&self, hash: hash::Hash32, ptr: fileptr::FilePtr) {


    }

    pub fn get(&self, hash: hash::Hash32) -> Option<fileptr::FilePtr> {

        unimplemented!();
    }
*/


}





#[cfg(test)]
pub fn test_store() {

}


#[cfg(test)]
mod tests {
    use super::Store;
    use config;

    #[test]
    fn test_store_new() {
        let _ = Store::new(&config::Config::new_test());
    }
}