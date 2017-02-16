

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



use itertools::Itertools;
use buffer::*;

use config;
use rayon::prelude::*;

use slog;

use store::{TxPtr,BlockHeaderPtr};
use store::flatfileset::FlatFileSet;

use store::hash_index::{HashIndex,HashIndexGuard};

use transaction::Transaction;

mod params;

pub mod record;
pub use self::record::{Record,RecordPtr};

const MB:                 u64 = 1024 * 1024;
const FILE_SIZE:          u64 = 1024 * MB ;
const MAX_CONTENT_SIZE:   u64 = FILE_SIZE - 10 * MB ;

const SUBPATH: &'static str   = "spent_tree";
const PREFIX:  &'static str   = "st-";


// temporarily we use a vec instead of the memmap
const VEC_SIZE: usize = 500_000_000;

#[derive(Debug, PartialEq)]
pub enum SpendingError {
    OutputNotFound,
    OutputAlreadySpent,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct BlockPtr {
    pub start:    RecordPtr,
    pub length:   u64,
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
}

pub struct SpentTree {

    fileset:    FlatFileSet<RecordPtr>,

    stats: SpentTreeStats
}



#[derive(Debug, Default)]
pub struct SpentTreeStats {
    pub blocks:     i64,
    pub inputs:     i64,
    pub seeks:      i64,
    pub total_move: i64,
    pub jumps:      i64,
    pub use_diff:   [i64; params::SKIP_FIELDS]
}

// Make stats additive
impl ::std::ops::Add for SpentTreeStats {
    type Output = SpentTreeStats;

    fn add(self, other: SpentTreeStats) -> SpentTreeStats {
        // sum use_diff array
        let mut use_diff: [i64; params::SKIP_FIELDS] = Default::default();
        for n in 0..use_diff.len() { use_diff[n] = self.use_diff[n] + other.use_diff[n] };

        SpentTreeStats {
            blocks: self.blocks + other.blocks,
            inputs: self.inputs + other.inputs,
            seeks:  self.seeks  + other.seeks,
            total_move: self.total_move + other.total_move,
            jumps: self.jumps   + other.jumps,
            use_diff: use_diff
        }
    }
}



/// This is the primary algorithm to check double-spents and the existence of outputs
fn seek_and_set_inputs(
                       records: &[Record],
                       block: &mut [Record],
                       block_idx: usize,
                       logger: &slog::Logger) -> Result<SpentTreeStats, SpendingError>
{
    trace!(logger, format!("Start idx={:?}", block_idx));

    let results: Vec<Result<SpentTreeStats, SpendingError>> = block[1..]

        .par_iter_mut()
        .enumerate()
        .map(|(i,rec)| {

            trace!(logger, format!("Testing;{:?}", rec));
            debug_assert!(rec.is_transaction() || rec.is_output());

            rec.seek_and_set(block_idx+i+1, records, logger)

        })
        .collect();

    results.into_iter().fold_results(Default::default(), |a,b| { a+b } )

}


impl SpentTree {
    pub fn new(cfg: &config::Config) -> SpentTree {

        let dir = &cfg.root.clone().join(SUBPATH);

        let stats: SpentTreeStats = Default::default();



        SpentTree {
            fileset: FlatFileSet::new(
                dir, PREFIX, FILE_SIZE, MAX_CONTENT_SIZE),

            stats: stats
        }
    }


    pub fn get_all_records(&mut self) -> &[Record] {

        self.fileset.read_mut_slice(RecordPtr::new(0), VEC_SIZE)
    }

    pub fn get_block_mut(&mut self, block_ptr: BlockPtr) -> &mut [Record] {

        self.fileset.read_mut_slice(block_ptr.start, block_ptr.length as usize)
    }

    /// Retrieves a single record from the spent-tree
    pub fn get_record(&mut self, ptr: RecordPtr) -> Record {

        * self.fileset.read_fixed(ptr)
    }

    /// Stores a block in the spent_tree. The block will be initially orphan.
    ///
    /// The result is a BlockPtr that can be stored in the hash-index
    pub fn store_block(&mut self, block_header_ptr: BlockHeaderPtr, file_ptrs: Vec<Record>) -> BlockPtr {

        let block: Vec<Record> = vec![ Record::new_orphan_block(block_header_ptr)]
            .into_iter()
            .chain(file_ptrs.into_iter())
            .collect();


        let result_ptr = self.fileset.write_all(&block);

        BlockPtr {
            start:    result_ptr,
            length:   block.len() as u64,
            is_guard: false
        }
    }


    /// If an orphan block is stored in the spent-tree, some transaction-inputs might not be resolved
    /// to their outputs. These will still be null-pointers instead of output-pointers
    ///
    /// This looks up the corresponding outputs; needs to be called before connect_block
    pub fn revolve_orphan_pointers(&mut self,
                                   transactions:  &mut FlatFileSet<TxPtr>,
                                   tx_index:      &mut HashIndex<TxPtr>,
                                   block:  BlockPtr) {

        let mut input_idx = 0;
        let mut last_tx_ptr: Option<TxPtr> = None;

        for record in self.get_block_mut(block) {

            if record.is_unmatched_input() {

                let bytes   = transactions.read(last_tx_ptr.unwrap());
                let mut buf = Buffer::new(bytes);
                let tx      = Transaction::parse(&mut buf).unwrap();

                let input = &tx.txs_in[input_idx];

                // find the matching output
                let tx_ptr: TxPtr = *tx_index
                    .get(input.prev_tx_out)
                    .iter()
                    .find(|ptr| !ptr.is_guard())
                    .expect("Could not find input; this should have been resolved before connecting the block!");

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
                         logger: &slog::Logger,
                         previous_block: BlockPtr,
                         target_block:   BlockPtr) -> Result<(), SpendingError> {

        let timer = ::std::time::Instant::now();

        let block_idx              = target_block.start.to_index();
        let block:   &mut [Record] = self.fileset.read_mut_slice(target_block.start, target_block.length as usize);
        let records: &[Record]     = self.get_all_records();

        // Make the link,
        block[0] = Record::new_block(previous_block, block[0]);


        // verify all inputs and set proper skips
        let stats  = seek_and_set_inputs(records, block, block_idx as usize, logger)?;


        let elaps : isize = timer.elapsed().as_secs() as isize * 1000 +
            timer.elapsed().subsec_nanos() as isize / 1_000_000 as isize;

        info!(logger, "scan_stats";
            "stats" => format!("{:?}", stats),
            "inputs" => stats.inputs,
            "ms/inputs" => (elaps+1) as f64 / stats.inputs as f64,
            "seek_avg" => stats.seeks / (stats.inputs+1)
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
    use store::{BlockHeaderPtr, TxPtr};
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
        (  ( BlockHeaderPtr::new(0,$header), vec![
               $( Record::new_transaction(TxPtr::new(0,$tx))
                $( ,  $( Record::new_output(TxPtr::new(0,$tx_in),$tx_in_idx) ),* ),* ),*
            ])
        )

    }

    impl SpentTree {
        // wrapper around store_block that accepts a tuple instead of two params
        // for easier testing with block! macros
        fn store(&mut self, tuple: (BlockHeaderPtr, Vec<Record>)) -> BlockPtr {
            self.store_block(tuple.0, tuple.1)
        }
    }

    #[test]
    fn test_spent_tree_connect() {
        let log = slog::Logger::root(slog_term::streamer().compact().build().fuse(), o!());

        let mut st  = SpentTree::new(& test_cfg!());

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
        st.connect_block(&log, block1, block2a).unwrap();
        st.connect_block(&log, block1, block2b).unwrap();

        // this one should only "fit" onto 2b
        let block3b = st.store(block!(blk 7 =>
            [tx 8 => (6;1)],
            [tx 9 => (2;1)]
        ));


        assert_eq!(
            st.connect_block(&log, block2a, block3b).unwrap_err(),
            SpendingError::OutputNotFound);

        let block3b = st.store(block!(blk 7 =>
            [tx 8 => (6;1)],
            [tx 9 => (2;1)]
        ));
        st.connect_block(&log, block2b, block3b).unwrap();

        // now this should only fir on 2a and not on 3b as at 3b it is already spent
        let block4a = st.store(block!(blk 10 =>
            [tx 11 => (2;1)],
            [tx 12 => (2;2)]
        ));
        assert_eq!(
            st.connect_block(&log, block3b, block4a).unwrap_err(),
            SpendingError::OutputAlreadySpent);

        let block4a = st.store(block!(blk 10 =>
            [tx 11 => (2;1)],
            [tx 12 => (2;2)]
        ));
        st.connect_block(&log, block2b, block4a).unwrap();

    }

    #[test]
    fn test_spent_tree1() {
        let log = slog::Logger::root(slog_term::streamer().compact().build().fuse(), o!());


        let block1 = block!(blk 1 =>
            [tx 2 => (2;1),(2;0)],
            [tx 3]
        );


        let mut st  = SpentTree::new(& test_cfg!() );

        let block_ptr = st.store_block(block1.0, block1.1);

        let block2 = block!(blk 4 =>
            [tx 5 => (2;2),(2;3)],
            [tx 6 ]
        );

        let block_ptr2 = st.store_block(block2.0, block2.1);

        println!("{:?}", block_ptr2.start);

        st.connect_block(&log, block_ptr, block_ptr2).unwrap();

        let recs = st.get_all_records();
        assert!(   recs[0].is_block() );
        assert_eq!(recs[0].get_file_offset(), 1);
        assert!(   recs[1].is_transaction() );
        assert_eq!(recs[1].get_file_offset(), 2);
        assert!(   recs[2].is_output() );
        assert!(   recs[3].is_output() );
        assert!(   recs[4].is_transaction() );
        assert!(   recs[5].is_block() );
        assert!(   recs[6].is_transaction() );
        assert!(   recs[7].is_output() );
        assert!(   recs[8].is_output() );
        assert!(   recs[9].is_transaction() );


    }


    #[test]
    fn test_orphan_block() {
        // care must be taken that when a block is added to the spent-tree before one of its
        // predecessors, it may not be complete.
        // This because in the spent tree, the inputs are stored as the outputs they reference,
        // but these inputs may not have been available.

        // The resolve orphan block checks and fixes these "left-out" references.


    }
}

