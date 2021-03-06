

/// The spend tree stores the location of transactions in the block-tree
///
/// It is tracks the tree of blocks and is used to verify whether a block can be inserted at a
/// certain location in the tree
///
/// A block consists of the chain of records:
///
/// [start-of-block] <- [transaction] <- [spend-output] <- [spend-output] <- [transaction] <- [end-of-block]...
///
/// One [spend-output] record is added per input of a transaction.
///
/// Blocks are connected with the [start-of-block] record which points to the [end-of-block] of previous block.
/// Often this is the previous records, but blocks can also be added in different order.
/// The [start-of-block] then point to NULL until the previous block comes in.
///


use itertools::Itertools;
use buffer::*;

use config;
use rayon::prelude::*;

use slog;

use store;
use store::{TxPtr,BlockHeaderPtr};
use store::flatfileset::FlatFileSet;

use store::hash_index::{HashIndex,HashIndexGuard};
use store::spend_index::SpendIndex;

use transaction::Transaction;

pub mod record;
pub use self::record::{Record,RecordPtr};

const MB:                 u64 = 1024 * 1024;
const FILE_SIZE:          u64 = 16 * 1024 * MB ;
const MAX_CONTENT_SIZE:   u64 = FILE_SIZE - 10 * MB ;

const SUBPATH: &'static str   = "spend-tree";
const PREFIX:  &'static str   = "st-";


// temporarily we use a vec instead of the dynamic growing flatfileset
// this isn't a big problem because the OS will not allocate the trailing zeros
const VEC_SIZE: usize = 800_000_000;

#[derive(Debug, PartialEq)]
pub enum SpendingError {
    OutputNotFound,
    OutputAlreadySpend,
}

/// A pointer into the spend-tree.
/// This always points to a `start-of-block` record
/// These objects are stored in the block-index, to lookup blocks
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct BlockPtr {
    pub start:    RecordPtr,
    pub length:   u64,

    // A guard is used for wrongly ordered block-insertion. See [hash_index.rs] for details
    pub is_guard: bool
}


impl HashIndexGuard for BlockPtr {
    fn is_guard(self) -> bool { self.is_guard }
}


impl BlockPtr {
    pub fn to_guard(self) -> BlockPtr {
        BlockPtr {
            start:    self.start,
            length:   self.length,
            is_guard: true
        }
    }
    pub fn to_non_guard(self) -> BlockPtr {
        BlockPtr {
            start:    self.start,
            length:   self.length,
            is_guard: false
        }
    }

    pub fn end(self ) -> RecordPtr {
        RecordPtr::new(self.start.to_index() + self.length -1)
    }

}

pub struct SpendTree {

    fileset:    FlatFileSet<RecordPtr>
}



/// Stats are passed around on success for performance monitoring
#[derive(Debug, Default)]
pub struct SpendTreeStats {
    pub blocks:     i64,
    pub inputs:     i64,
    pub seeks:      i64,
    pub total_move: i64,
    pub total_diff: i64,
    pub jumps:      i64,
}

// Make stats additive
impl ::std::ops::Add for SpendTreeStats {
    type Output = SpendTreeStats;

    fn add(self, other: SpendTreeStats) -> SpendTreeStats {

        SpendTreeStats {
            blocks: self.blocks + other.blocks,
            inputs: self.inputs + other.inputs,
            seeks:  self.seeks  + other.seeks,
            total_move: self.total_move + other.total_move,
            total_diff: self.total_diff+ other.total_diff,
            jumps: self.jumps   + other.jumps,
        }
    }
}




/// This is the algorithm to check double-spends and the existence of outputs
/// It will call the verify_spend function on Record in parallel for each output
fn seek_and_set_inputs(
                       records: &[Record],
                       block: &mut [Record],
                       block_idx: usize,
                       spend_index: &SpendIndex,
                       logger: &slog::Logger) -> Result<usize, SpendingError>
{

    // we use the block minus the first and last record (which are just markers
    let len = block.len()-1;
    let results: Vec<Result<usize, SpendingError>> = block[1..len]

        .par_iter_mut()
        .enumerate()
        .map(|(i,rec)| {

            debug_assert!(rec.is_transaction() || rec.is_output());

            rec.verify_spend(spend_index, block_idx+i+1, records, logger)

        })
        .collect();

    // Return the input_count, or an error if any
    results.into_iter().fold_results(Default::default(), |a,b| { a+b } )

}


impl SpendTree {
    pub fn new(cfg: &config::Config) -> SpendTree {

        let dir = &cfg.root.clone().join(SUBPATH);

        SpendTree {
            fileset: FlatFileSet::new(
                dir, PREFIX, FILE_SIZE, MAX_CONTENT_SIZE)
        }
    }

    // Returns the full spend-tree as record-slice.
    pub fn get_all_records(&mut self) -> &[Record] {

        self.fileset.read_mut_slice(RecordPtr::new(0), VEC_SIZE)
    }

    // Returns the given block as a mutable slice
    pub fn get_block_mut(&mut self, block_ptr: BlockPtr) -> &mut [Record] {

        self.fileset.read_mut_slice(block_ptr.start, block_ptr.length as usize)
    }

    /// Retrieves a single record from the spend-tree
    pub fn get_record(&mut self, ptr: RecordPtr) -> Record {

        * self.fileset.read_fixed(ptr)
    }

    /// Stores a block in the spend_tree. The block will be initially orphan.
    ///
    /// The result is a BlockPtr that can be stored in the hash-index
    pub fn store_block(&mut self, block_header_ptr: BlockHeaderPtr, file_ptrs: Vec<Record>) -> BlockPtr {

        let  count = file_ptrs.len();
        let block: Vec<Record> = vec![ Record::new_orphan_block_start()]
            .into_iter()
            .chain(file_ptrs.into_iter())
            .chain(vec![Record::new_block_end(block_header_ptr, count)]).into_iter()
            .collect();


        let result_ptr = self.fileset.write_all(&block);

        BlockPtr {
            start:    result_ptr,
            length:   block.len() as u64,
            is_guard: false
        }
    }


    /// If an orphan block is stored in the spend-tree, some transaction-inputs might not be resolved
    /// to their outputs. These will still be unmatched_output records instead of output-pointers
    ///
    /// This looks up the corresponding outputs; needs to be called before connect_block
    pub fn revolve_orphan_pointers(&mut self,
                                   transactions:  &mut store::Transactions,
                                   tx_index:      &mut HashIndex<TxPtr>,
                                   block:  BlockPtr) {

        let mut input_idx = 0;
        let mut last_tx_ptr: Option<TxPtr> = None;

        for record in self.get_block_mut(block) {

            if record.is_unmatched_input() {

                let bytes   = transactions.read(last_tx_ptr.unwrap());
                let mut buf = Buffer::new(&bytes);
                let tx      = Transaction::parse(&mut buf).unwrap();

                let input = &tx.txs_in[input_idx];

                // find the matching output
                let tx_ptr: TxPtr = *tx_index
                    .get(input.prev_tx_out)
                    .iter()
                    .find(|ptr| !ptr.is_guard())
                    .expect("Could not find input; this must have been resolved before connecting the block!");

                let output_record = Record::new_output(tx_ptr, input.prev_tx_out_idx);

                *record = output_record;

                input_idx   += 1;

            } else if record.is_transaction() {

                last_tx_ptr = Some(record.get_transaction_ptr());
                input_idx   = 0;
            }
            else if record.is_output() {

                input_idx   += 1;
            }

        }

    }


    /// Verifies of each output in the block at target_start
    /// Then lays the connection between previous_end and target_start
    pub fn connect_block(&mut self,
                         spend_index:    &mut SpendIndex,
                         logger:         &slog::Logger,
                         previous_block: BlockPtr,
                         target_block:   BlockPtr) -> Result<(), SpendingError> {

        let timer = ::std::time::Instant::now();

        let block_idx              = target_block.start.to_index();
        let block:   &mut [Record] = self.fileset.read_mut_slice(target_block.start, target_block.length as usize);
        let records: &[Record]     = self.get_all_records();


        // Make the link,
        block[0] = Record::new_block_start(previous_block);


        // Update the spend-index
        // TODO this should jump more blocks back; and register its parent-requirement.
        // This is important once we allow forks
        let s = previous_block.start.to_index() as usize;
        let l = previous_block.length as usize;
        let immutable_block: &[Record] = &records[s+1..s+l-1];
        for rec in immutable_block.iter() {

            spend_index.set(rec.hash());
        }

        // verify all inputs in the spend tree and spend-index
        let input_count = seek_and_set_inputs(records, block, block_idx as usize, spend_index, logger)?;

        let elapsed : isize = timer.elapsed().as_secs() as isize * 1000 +
            timer.elapsed().subsec_nanos() as isize / 1_000_000 as isize;

        info!(logger, "connected";
            "records" => block.len(),
            "inputs" => input_count,
            "ms/input" => (elapsed+1) as f64 / input_count as f64,
        );

        Ok(())
    }

}





#[cfg(test)]
mod tests {

    extern crate tempdir;


    use slog_term;
    use slog;
    use slog::DrainExt;
    use store::flatfileset::FlatFilePtr;
    use super::*;
    use store::spend_index::SpendIndex;
    use store::{BlockHeaderPtr, TxPtr};

    /// Macro to create a block for the spend_tree tests
    /// blockheaders and txs are unqiue numbers (fileptrs but where they point to doesn't matter
    /// for the spend_tree).
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
        (  ( BlockHeaderPtr::new(0,$header), vec![
               $( Record::new_transaction(TxPtr::new(0,$tx))
                $( ,  $( Record::new_output(TxPtr::new(0,$tx_in),$tx_in_idx) ),* ),* ),*
            ])
        )

    }

    impl SpendTree {
        // wrapper around store_block that accepts a tuple instead of two params
        // for easier testing with block! macros
        fn store(&mut self, tuple: (BlockHeaderPtr, Vec<Record>)) -> BlockPtr {
            self.store_block(tuple.0, tuple.1)
        }
    }

    #[test]
    fn test_spend_tree_connect() {
        let log = slog::Logger::root(slog_term::streamer().compact().build().fuse(), o!());

        let mut st  = SpendTree::new(& test_cfg!());
        let mut si  = SpendIndex::new(& test_cfg!());

        let block1 = st.store(block!(blk 1 =>
            [tx 2]
        ));

        let block2a = st.store(block!(blk 3 =>
            [tx 4 => (2;0)]
        ));

        let block2b = st.store(block!(blk 5 =>
            [tx 6 => (2;0)]
        ));


        // create a tree, both 2a and 2b attached to 1
        st.connect_block(&mut si, &log, block1, block2a).unwrap();
        st.connect_block(&mut si, &log, block1, block2b).unwrap();

        // this one should only "fit" onto 2b
        let block3b = st.store(block!(blk 7 =>
            [tx 8 => (6;1)],
            [tx 9 => (2;1)]
        ));


        assert_eq!(
            st.connect_block(&mut si, &log, block2a, block3b).unwrap_err(),
            SpendingError::OutputNotFound);

        let block3b = st.store(block!(blk 7 =>
            [tx 8 => (6;1)],
            [tx 9 => (2;1)]
        ));
        st.connect_block(&mut si, &log, block2b, block3b).unwrap();

        // now this should only fir on 2a and not on 3b as at 3b it is already spend
        let block4a = st.store(block!(blk 10 =>
            [tx 11 => (2;1)],
            [tx 12 => (2;2)]
        ));
        assert_eq!(
            st.connect_block(&mut si, &log, block3b, block4a).unwrap_err(),
            SpendingError::OutputAlreadySpend);

        let block4a = st.store(block!(blk 10 =>
            [tx 11 => (2;1)],
            [tx 12 => (2;2)]
        ));
        st.connect_block(&mut si, &log, block2b, block4a).unwrap();

    }

    #[test]
    fn test_spend_tree1() {
        let log = slog::Logger::root(slog_term::streamer().compact().build().fuse(), o!());


        let block1 = block!(blk 1 =>
            [tx 2 => (2;1),(2;0)],
            [tx 3]
        );


        let mut st  = SpendTree::new(& test_cfg!() );
        let mut si  = SpendIndex::new(& test_cfg!());

        let block_ptr = st.store_block(block1.0, block1.1);

        let block2 = block!(blk 4 =>
            [tx 5 => (2;2),(2;3)],
            [tx 6 ]
        );

        let block_ptr2 = st.store_block(block2.0, block2.1);

        println!("{:?}", block_ptr2.start);

        st.connect_block(&mut si, &log, block_ptr, block_ptr2).unwrap();

        let recs = st.get_all_records();
        assert!(   recs[0].is_block_start() );
        assert!(   recs[1].is_transaction() );
        assert_eq!(recs[1].get_file_offset(), 2);
        assert!(   recs[2].is_output() );
        assert!(   recs[3].is_output() );
        assert!(   recs[4].is_transaction() );
        assert!(   recs[5].is_block_end() );
        assert_eq!(recs[5].get_file_offset(), 1);
        assert!(   recs[6].is_block_start() );
        assert!(   recs[7].is_transaction() );
        assert!(   recs[8].is_output() );
        assert!(   recs[9].is_output() );
        assert!(   recs[10].is_transaction() );
        assert!(   recs[11].is_block_end() );


    }


    #[test]
    fn test_orphan_block() {
        // care must be taken that when a block is added to the spend-tree before one of its
        // predecessors, it may not be complete.
        // This because in the spend tree, the inputs are stored as the outputs they reference,
        // but these inputs may not have been available.

        // The resolve orphan block checks and fixes these "left-out" references.


    }
}

