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


use slog ;

use slog_term;
use slog::DrainExt;

pub mod fileptr;


mod flatfile;
mod flatfileset;

mod block_content;
mod hash_index;
mod spent_tree;

pub use self::spent_tree::SpendingError;
pub use self::spent_tree::record::RecordPtr;

use config;
use hash::*;

use metrics::Metrics;

use self::fileptr::FilePtr;




/// This is the accessor to the
pub struct Store {

    //pub index: index::Index,

    // Flat files
    pub block_content: block_content::BlockContent,
    pub hash_index:    hash_index::HashIndex,
    pub spent_tree:    spent_tree::SpentTree,


    pub metrics:       Metrics,

    pub logger:        slog::Logger
}


impl Store {

    pub fn new(cfg: &config::Config) -> Store {

        Store {
            //index: index::Index::new(cfg),

            block_content: block_content::BlockContent::new(&cfg),
            hash_index:    hash_index::HashIndex::new(&cfg),
            spent_tree:    spent_tree::SpentTree::new(&cfg),

            metrics:       Metrics::new(),
            logger:        slog::Logger::root(slog_term::streamer().build().fuse(), o!()),
        }
    }


    pub fn get_block_hash(&mut self, blockheader_ptr: FilePtr) -> Hash32Buf {

        // follow indirection through spent-tree
        let block_hdr = self.spent_tree.load_data_from_spent_tree_ptr(
            &mut self.block_content,
            blockheader_ptr);

        Hash32Buf::double_sha256(block_hdr)

    }


}




#[cfg(test)]
mod tests {

    use super::*;

    use block::BlockHeader;
    use hash::*;
    use buffer::*;
    use config;

    #[test]
    fn test_get_block_hash() {

        // Create a fake blockheader
        let block_hdr_raw = [12u8; 80];
        let block_hdr = BlockHeader::parse(&mut Buffer::new(&block_hdr_raw)).unwrap();
        let hash = Hash32Buf::double_sha256(&block_hdr_raw);


        let mut store = Store::new(& config::Config::new_test());

        let block_hdr_ptr = store.block_content.write_blockheader(&block_hdr);

        let blockptr = store.spent_tree.store_block(block_hdr_ptr, vec![]);

        // both the start end the end should point to the block_content and
        // the hash should be equal to the original
        assert_eq!(hash.as_ref(), store.get_block_hash(blockptr.start.ptr).as_ref());
        assert_eq!(hash.as_ref(), store.get_block_hash(blockptr.end.ptr).as_ref());



    }

    #[test]
    fn test_store_new() {
        let _ = Store::new(&config::Config::new_test());
    }
}