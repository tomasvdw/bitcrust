

/// The spent tree stores the location of transactions in the block-tree
///
/// It is tracks the tree of blocks and is used to verify whether a block can be inserted at a
/// certain location in the tree
///
/// A block consists of the chain of records:
///
/// [block-header] <- [transaction] <- [spent-output] <- [spent-output] <- [transaction] <- ...
///
/// One [spent-output] record is added per input of a transaction.
///
/// Blocks are added regardless of they connect to a previous block. If the previous block comes in
/// later the following blocks are readded.
///
/// Verification takes place on *tip-propagation*. The top pointer is moved forward to the next block
/// after all [spent-outputs] in that block have been checked. This entails scanning back the chain.
/// The scan is succesful if the transaction is found and unsuccesful if it is not found or if the
/// same spent-output is found before the transaction.
///
/// TODO
/// Refactor FilePtr & Record to be typed as what they point to
///
use std::mem;


use config;

use store::fileptr::FilePtr;
use store::flatfileset::FlatFileSet;

mod record;
use self::record::Record;

const MB:                 u32 = 1024 * 1024;
const FILE_SIZE:          u32 = 1024 * MB as u32;
const MAX_CONTENT_SIZE:   u32 = FILE_SIZE - 10 * MB as u32 ;

const SUBPATH: &'static str   = "spent_tree";
const PREFIX:  &'static str   = "st-";

#[derive(Debug)]
pub enum SpendingError {
    OutputNotFound,
    OutputAlreadySpent,
}


pub struct SpentTree {

    fileset:    FlatFileSet,

}

impl SpentTree {
    pub fn new(cfg: &config::Config) -> SpentTree {

        let dir = &cfg.root.clone().join(SUBPATH);

        SpentTree {
            fileset: FlatFileSet::new(
                dir, PREFIX, FILE_SIZE, MAX_CONTENT_SIZE)
        }
    }


    /// Converts the set of block_content-fileptrs
    /// into a set of records to be stored in the spent_tree
    ///
    pub fn create_block(blockheader: FilePtr, file_ptrs: Vec<FilePtr>) -> Vec<Record> {

        let mut result: Vec<Record> = Vec::with_capacity(file_ptrs.len()+1);

        // TODO: stronger typing!
        result.push(Record::new(blockheader));

        let mut previous: Option<FilePtr> = None;
//        println!("PTRS: {:?}", file_ptrs);
        for (idx, ptr) in file_ptrs.iter().enumerate() {

            let mut r = Record::new(*ptr);

            r.set_skip_previous();

            result.push(r);

            previous = Some(*ptr);
        };


        result
    }



    /// Stores a block in the spent_tree. The block will be initially orphan.
    ///
    /// The result is a pointer to the first and last record
    pub fn store_block(&mut self, blockheader: FilePtr, file_ptrs: Vec<FilePtr>) -> (FilePtr, FilePtr) {

        let block = SpentTree::create_block(blockheader, file_ptrs);


        let result_ptr = self.fileset.write_all(&block);
        let end_ptr = result_ptr.offset((block.len()-1) * mem::size_of::<Record>());

        (result_ptr, end_ptr)
    }

    pub fn set_previous(&mut self, target: FilePtr, previous: FilePtr) {

    }

}

