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
//! This maps hashes to transaction and blockheaders
//! A tx-hash can point to a transaction or to a set of inputs;
//! in the latter case, the inputs are guards: these must be verified
//! before the transaction can be inserted
//!
//! # spent_tree
//!
//!


pub mod fileptr;

mod flatfile;
mod flatfileset;

mod block_content;
mod hash_index;
mod spent_tree;

pub use self::spent_tree::SpendingError;

use config;

use metrics::Metrics;






/// This is the accessor to the
pub struct Store {

    //pub index: index::Index,

    // Flat files
    pub block_content: block_content::BlockContent,
    pub hash_index:    hash_index::HashIndex,
    pub spent_tree:    spent_tree::SpentTree,


    pub metrics:       Metrics,
}


impl Store {

    pub fn new(cfg: &config::Config) -> Store {

        Store {
            //index: index::Index::new(cfg),

            block_content: block_content::BlockContent::new(&cfg),
            hash_index:    hash_index::HashIndex::new(&cfg),
            spent_tree:    spent_tree::SpentTree::new(&cfg),

            metrics:       Metrics::new(),
        }
    }

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