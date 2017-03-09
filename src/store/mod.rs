//!
//!
//! The store consists of 6 filesets.
//!
//! # transactions
//!
//! This contains raw transactions
//! These are directly written to their flatfileset
//!
//! # block_headers
//!
//! This contains raw block-headers
//! These are directly written to their flatfileset
//!
//!
//! # tx_index
//!
//! This maps hashes to transactions
//! A tx-hash can point to a transaction or to a set of inputs;
//! in the latter case, the inputs are guards: these must be verified
//! before the transaction can be inserted
//!
//! # block_index
//!
//! This maps hashes to blocks
//! A block-pointer of a connected block is inserted at its own hash
//! A block-pointer of an orphan block is inserted at the previous-block hash
//! and serves as a *guard*. This ensures that the connection is made when the previous block comes
//! in.
//!
//! # spent_tree
//!
//! The tree that links transactions to blocks. See [[spent_tree]]
//!
//! # spent_index
//!
//! A bit-index that is used as a "broom-wagon" for the spent_tree to prevent deep-tree searching
//!


use slog ;
use slog_term;
use slog::DrainExt;



mod txptr;
mod blockheaderptr;

mod flatfile;
mod flatfileset;

mod hash_index;
mod spent_index;

mod spent_tree;

pub use self::spent_tree::SpendingError;
pub use self::spent_tree::BlockPtr;
pub use self::spent_tree::record::{RecordPtr,Record};

pub use self::txptr::TxPtr;
pub use self::hash_index::{HashIndex, HashIndexGuard};
pub use self::blockheaderptr::BlockHeaderPtr;

pub use self::flatfileset::{FlatFilePtr,FlatFileSet};

pub type TxIndex = HashIndex<TxPtr>;

use config;
use hash::*;



use metrics::Metrics;



const MB:                 u64 = 1024 * 1024;
const FILE_SIZE:          u64 = 2 * 1024 * MB;
const MAX_CONTENT_SIZE:   u64 = FILE_SIZE - 10 * MB as u64 ;



/// This is the accessor to all stuff on disk.
/// A single store cannot be used from multiple threads without precaution,
/// but multiple Stores from different threads/processes can use the same
/// files concurrently
pub struct Store {
    // Flat files contain transactions and blockheaders
    pub transactions: flatfileset::FlatFileSet<TxPtr>,
    pub block_headers: flatfileset::FlatFileSet<BlockHeaderPtr>,

    pub tx_index:      TxIndex,
    pub block_index: hash_index::HashIndex<BlockPtr>,

    pub spent_tree: spent_tree::SpentTree,
    pub spent_index: spent_index::SpentIndex,

    // todo; this needs to go; structured logging is superior
    pub metrics: Metrics,

    pub logger: slog::Logger,

    // needed for cloning
    cfg: config::Config
}


impl Store {

    pub fn new(cfg: &config::Config) -> Store {

        Store {
            transactions:  FlatFileSet::new(
                &cfg.root.clone().join("transactions"),
                "tx",
                FILE_SIZE,
                MAX_CONTENT_SIZE),

            block_headers:  FlatFileSet::new(
                &cfg.root.clone().join("headers"),
                "bh",
                FILE_SIZE,
                MAX_CONTENT_SIZE),

            tx_index:     hash_index::HashIndex::new(&cfg, "tx-index"),
            block_index:  hash_index::HashIndex::new(&cfg, "block-index"),

            spent_tree:   spent_tree::SpentTree::new(&cfg),
            spent_index:  spent_index::SpentIndex::new(&cfg),


            metrics:       Metrics::new(),
            logger:        slog::Logger::root(slog_term::streamer().compact().build().fuse(), o!()),
            cfg:           cfg.clone()
        }
    }




    /// Gets the block hash from a block-ptr;
    /// This follows the indirection through the spent-tree
    ///
    /// Note: This might be not the best spot for this...
    pub fn get_block_hash(&mut self, block_ptr: BlockPtr) -> Hash32Buf {

        // follow indirection through spent-tree
        let block_hdr_rec = self.spent_tree.get_record(block_ptr.end());
        let block_hdr     = self.block_headers.read(block_hdr_rec.get_block_header_ptr());

        Hash32Buf::double_sha256(block_hdr)

    }



}

impl Clone for Store {

    // Clones the store to allow for concurrent access. Not quite cheap
    // so threads should reuse there own store (for instance, with par_chunks)
    fn clone(&self) -> Store {

        Store::new(&self.cfg)
    }
}

unsafe impl Sync for Store {}

#[cfg(test)]
mod tests {

    use super::*;

    use block::BlockHeader;
    use buffer::*;

    #[test]
    fn test_get_block_hash() {

        // Create a fake blockheader
        let block_hdr_raw = [12u8; 80];
        let block_hdr = BlockHeader::parse(&mut Buffer::new(&block_hdr_raw)).unwrap();
        let hash = Hash32Buf::double_sha256(&block_hdr_raw);


        let mut store = Store::new(& test_cfg!());

        let block_hdr_ptr = store.block_headers.write(block_hdr.to_raw());

        let blockptr = store.spent_tree.store_block(block_hdr_ptr, vec![]);

        // both the start end the end should point to the block_content and
        // the hash should be equal to the original
        assert_eq!(hash.as_ref(), store.get_block_hash(blockptr).as_ref());


    }

    #[test]
    fn test_store_new() {
        let _ = Store::new(& test_cfg!());
    }

    // this takes a fake spent tree (created with block! macro's) and use it to construct
    // valid transactions and blocks
    /*fn test_create_store_from_spent_tree(spent_tree: RecordPtr) -> Store {






    }*/

    use config::Config;
    use std::fs;

    //use block::BlockHeader;

    ///
    #[test]
    #[ignore]
    fn reindex() {
        let _ = fs::remove_dir_all("rindex/spent-tree");
        let _ = fs::remove_dir_all("rindex/spent-index");
        let _ = fs::remove_dir_all("rindex/tx-index");
        let _ = fs::remove_dir_all("rindex/block-index");

        let cfg = Config::new("rindex");
        let mut store = Store::new(&cfg);

        let mut pos = TxPtr::new(0, super::flatfile::INITIAL_WRITEPOS);

        for (n, (header, tx_count)) in store.block_headers.read_block_headers().into_iter().enumerate() {

            println!("Block {} with {} transactions", n, tx_count);
            let _ = BlockHeader::parse(&mut Buffer::new(header));

            let (txs,p) = store.transactions.read_set(pos, tx_count);
            pos = p;
            println!("Blockheader {:?}", txs[0]);
        }

    }

}