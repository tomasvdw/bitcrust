

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
///
///
///
use std::mem;


use config;

use store::fileptr::FilePtr;
use store::flatfileset::FlatFileSet;

use store::block_content::BlockContent;

use hash::*;

pub mod record;
pub use self::record::{Record,RecordPtr};

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

pub struct BlockPtr {
    pub start: RecordPtr,
    pub end:   RecordPtr
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

        let mut result: Vec<Record> = Vec::with_capacity(file_ptrs.len()+2);

        result.push(Record::new(blockheader.as_block()));

        let mut previous: Option<FilePtr> = None;
//        println!("PTRS: {:?}", file_ptrs);
        for (idx, ptr) in file_ptrs.iter().enumerate() {

            let mut r = Record::new(*ptr);

            r.set_skip_previous();

            result.push(r);

            previous = Some(*ptr);
        };

        let mut rec_end = Record::new(blockheader.as_block());
        rec_end.set_skip_previous();

        result.push(rec_end);


        result
    }


    /// Retrieves the data pointed to by the spent-tree record at `ptr`
    /// This resolves the indirection: The passed ptr points to the spent-tree record
    /// This record points to the block_content
    pub fn load_data_from_spent_tree_ptr<'a>(&'a mut self, block_content: &'a mut BlockContent, ptr: FilePtr) -> &[u8] {
        let rec: &Record = self.fileset.read_fixed(ptr);
        let ptr = rec.ptr;

        block_content.read(ptr)
    }

    /// Stores a block in the spent_tree. The block will be initially orphan.
    ///
    /// The result is a pointer to the first and last record
    pub fn store_block(&mut self, blockheader: FilePtr, file_ptrs: Vec<FilePtr>) -> BlockPtr {

        let block = SpentTree::create_block(blockheader, file_ptrs);


        let result_ptr = self.fileset.write_all(&block);
        let end_ptr = result_ptr.offset(((block.len()-1) * mem::size_of::<Record>()) as i32);

        BlockPtr {
            start: RecordPtr::new(result_ptr),
            end:   RecordPtr::new(end_ptr)
        }
    }

    /// Verifies of each output in the block at target_start
    pub fn connect_block(&mut self, previous_end: RecordPtr, target_start: RecordPtr) -> Result<RecordPtr, SpendingError> {

        target_start.set_skips(&mut self.fileset, Some(previous_end));

        /*
        // find end
        let mut this_ptr = target_start;
        loop {
            this_ptr = this_ptr.offset(1);

            let rec: &Record = self.fileset.read_fixed(target_start);
            if rec.ptr.is_blockheader() {
                return Ok(this_ptr);
            }

            //return Ok(this_ptr);

        }*/
        Ok(target_start)


    }


}


#[cfg(test)]
mod tests {

    extern crate tempdir;
    use store::fileptr::FilePtr;
    use std::path::PathBuf;
    use  config;

    use super::*;

    /// Macro to create a block for the spent_tree tests
    /// blockheaders and txs are unqiue numbers (fileptrs but where they point to doesn't matter
    /// for the spent_tree).
    ///
    /// Construct a block as
    ///
    /// ```
    /// (blk 1 =>                 /* blocknr */
    ///   [tx 2],                /* tx with no inputs  */
    ///   [tx 3 => (2;0),(2;1)]  /* tx with two inputs referencing tx 2 ouput 0 and 1
    /// )
    ///
    macro_rules! block {

        (blk $header:expr =>
          $( [tx $tx:expr  $( => $(   ($tx_in:expr;$tx_in_idx:expr) ),* ),* ] ),*
        )
        =>
        (  ( FilePtr::new(0,$header), vec![
               $( FilePtr::new(0,$tx)  $( ,  $( FilePtr::new(0,$tx_in).as_input($tx_in_idx) ),* ),* ),*
            ])
        )

    }

    #[test]
    fn test_spent_tree() {


        let block1 = block!(blk 1 =>
            [tx 2 => (2;1),(2;0)],
            [tx 3]
        );


        let dir = tempdir::TempDir::new("test1").unwrap();
        //let path = PathBuf::from(dir.path());
        let cfg = config::Config { root: PathBuf::from("tmp")  }; //dir.path())

        let mut st  = SpentTree::new(&cfg);


        let block_ptr = st.store_block(block1.0, block1.1);

        let block2 = block!(blk 4 =>
            [tx 5 => (2;2),(2;3)],
            [tx 6 ]
        );

        let block_ptr2 = st.store_block(block2.0, block2.1);


        st.connect_block(block_ptr.end, block_ptr2.start);



        // now we check if we can browse back



    }
}

