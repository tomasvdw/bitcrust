#[macro_use]
use slog;

use std::mem;
use std::fmt;

use store::{TxPtr, BlockHeaderPtr};
use store::FlatFilePtr;


use store::spent_tree::SpendingError;
use store::spent_tree::BlockPtr;
use store::spent_index::SpentIndex;

use store::flatfile::INITIAL_WRITEPOS;
use super::SpentTreeStats;

use super::params;

// highest 2 bits are record-type
// 11 => start of block
// 10 => end of block
// 00 => transaction
// 01 => transaction-output
const RECORD_TYPE:u64    = 0xC000_0000_0000_0000;
const START_OF_BLOCK:u64 = 0xC000_0000_0000_0000;
const END_OF_BLOCK:u64   = 0x8000_0000_0000_0000;
const TRANSACTION:u64    = 0x0000_0000_0000_0000;
const OUTPUT:u64         = 0x4000_0000_0000_0000;

// START_OF_BLOCK;
// bits 0-61   fileoffset end of the previous block (in spent-tree)
//
// END_OF_BLOCK:
// bits 0 -31   fileoffset of blockheader
// bits 32-61   number of records in block
//
// TRANSACTION:
// bits 0 -31   fileoffset of transaction
// bits 32-47   filenumber of transaction
//
// OUTPUT:
// bits 0 -31   fileoffset of transaction
// bits 32-47   filenumber of transaction
// bits 48-61   output-index

#[derive(Clone,Copy,PartialEq)]
pub struct Record(u64);

impl fmt::Debug for Record {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "REC  {0:016X} ", self.0)
    }
}

impl Record {
    pub fn new_unmatched_input() -> Record {
        Record (0)
    }

    pub fn new_transaction(tx_ptr: TxPtr) -> Record {

        Record(
            (tx_ptr.get_file_number() as u64) << 32 |
            tx_ptr.get_file_offset()
        )
    }

    pub fn new_orphan_block_start() -> Record {

        Record(
            START_OF_BLOCK
        )
    }

    pub fn new_block_start(previous: BlockPtr) -> Record {

        Record(
            START_OF_BLOCK | (previous.start.to_index() + previous.length - 1)
        )

    }
    pub fn new_block_end(block: BlockHeaderPtr, size: usize) -> Record {

        debug_assert!(block.get_file_number() == 0);
        debug_assert!(size <= 0x3FFF_FFFF);

        Record(
            END_OF_BLOCK |
                (block.get_file_offset()) |
                ((size as u64) << 32)
        )
    }



    pub fn new_output(txptr: TxPtr, output_index: u32) -> Record {
        assert!(output_index <= 0x3fff); // TODO: this might not be true; fallback is needed

        Record(
            OUTPUT |
                (output_index as u64) << 48 |
                txptr.get_file_offset()
        )
    }

    /// If called on block-record, returns the index of the block-record and the record-count
    /// of the previous block
    pub fn previous_block(self) -> (usize, usize) {
        unimplemented!()
    }

    pub fn is_transaction(self) -> bool {

        (self.0 & RECORD_TYPE) == TRANSACTION
    }

    pub fn get_transaction_ptr(self) -> TxPtr {

        debug_assert!(self.is_transaction() || self.is_output());

        TxPtr::new(
            ((self.0 & 0xFFFF_0000_0000) >> 32) as i16,
            self.0 & 0xFFFF_FFFF
        )
    }

    pub fn get_block_header_ptr(self) -> BlockHeaderPtr {

        debug_assert!((self.0 & RECORD_TYPE) == END_OF_BLOCK);

        BlockHeaderPtr::new(0, self.0 & 0xFFFF_FFFF)
    }

    pub fn is_output(self) -> bool {

        (self.0 & RECORD_TYPE) == OUTPUT
    }

    pub fn is_block_start(self) -> bool {

        (self.0 & RECORD_TYPE) == START_OF_BLOCK
    }

    pub fn is_block_end(self) -> bool {

        (self.0 & RECORD_TYPE) == END_OF_BLOCK
    }

    pub fn is_unmatched_input(self) -> bool {
        self.0 == 0
    }

    /// This creates a non-cryptographic but perfect hash of the transaction or output
    pub fn hash(self) -> u64 {

        debug_assert!(self.is_transaction() || self.is_output());

        // We drop 4 bits from the filenumber and, for an output  add 1 + the output-index
        // The result is just as unique but smaller; we just drop the info to find the transaction
        // or the transaction from an output
        ((self.0 & 0xFFFF_FFFF_FFFF) >> 4)
        + (self.0 >> 62) // the bit that indicates its an output
        + ((self.0 & 0x1FFF_0000_0000_0000) >> 48)
    }


    fn to_transaction(self) -> Record {

        debug_assert!(self.is_output());

        Record(self.0 & 0x0000_FFFF_FFFF_FFFF)
    }

    fn output_index(self) -> u32 {
        unreachable!()
    }

    // test only as normally it makes no sense to treat file_offsets from different record-types
    // as the same expression
    #[cfg(test)]
    pub fn get_file_offset(self) -> u32 {

        self.0 as u32
    }
}

/// A filepointer that points to a record in the SpentTree
#[derive(PartialEq, Copy, Clone)]
pub struct RecordPtr(u64);

impl FlatFilePtr for RecordPtr {
    fn new(file_number: i16, file_offset: u64) -> RecordPtr {
        assert_eq!(file_number, 0); // can only have one spent-tree file

        RecordPtr((file_offset as u64 - INITIAL_WRITEPOS)
                      / mem::size_of::<Record>() as u64)
    }


    fn get_file_number(self) -> i16 { 0 }
    fn get_file_offset(self) -> u64 {
        INITIAL_WRITEPOS + self.0 * mem::size_of::<Record>() as u64
    }
}

impl fmt::Debug for RecordPtr {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{:?}", self.0)
    }
}



impl RecordPtr {
    pub fn new(ptr: u64) -> Self {
        RecordPtr(ptr)
    }


    pub fn to_index(self) -> u64 {
        self.0
    }
}


impl Record {

    pub fn seek_and_set(
        &mut self,
        spent_index: &SpentIndex,
        seek_idx: usize,
        records: &[Record],
        logger: &slog::Logger) -> Result<SpentTreeStats, SpendingError>

    {


        trace!(logger, format!("FL# Start search for {:?} {:?} {:?}", self, seek_idx, records[seek_idx]));

        let mut stats: SpentTreeStats = Default::default();

        if self.is_transaction() { return Ok(stats) }

        let seek_output      = *self;
        let seek_transaction = self.to_transaction();

        debug_assert!(seek_output.is_output());
        debug_assert!(self.0 == records[seek_idx].0);

        let mut seek_idx = seek_idx as u64;

        seek_idx -= 1;
        loop {


            stats.seeks += 1;

            trace!(logger, format!("FL# Search  {:?} @ {:?}", self, seek_idx));

            let seek_rec = records[seek_idx as usize];

            if seek_rec.0 == START_OF_BLOCK {

                return Err(SpendingError::OutputNotFound);
            }
            else if (seek_rec.0 & START_OF_BLOCK) == START_OF_BLOCK {

                // jump to next block
                stats.jumps += 1;
                seek_idx = seek_rec.0 & !START_OF_BLOCK;
                trace!(logger, format!("FL# Jump to {:?} @ {:?}", seek_rec, seek_idx));
            } else if seek_rec == seek_transaction {

                return Ok(stats);
            }
            else if seek_rec == seek_output {
                return Err(SpendingError::OutputAlreadySpent);
            }
            else {
                seek_idx -= 1;
            }


        }

        //return Ok(stats);
        /*
        let mut stats: SpentTreeStats = Default::default();

        stats.inputs += 1;

        // cur  will walk through the skip-list, starting from
        // the previous record
        let mut cur_idx = seek_idx - 1;

        let seek_filenr_pos = self.filenumber_and_pos();
        let seek_output_index = self.output_index();

        // all filenr_pos we've passed are at least this high
        let mut minimal_filenr_pos = seek_filenr_pos;

        // these are the pointers that will be stored in rec. By default, they just point to the
        // previous
        let mut seek_skips: [i32; params::SKIP_FIELDS] = [-1; params::SKIP_FIELDS];
        let mut seek_skip_blocks = [0_u8,0_u8, 0_u8];

        // for lack of a better name, `skip_r` traces which of the four skips of seek_rec are still
        // "following" the jumps. Initially all are (which will cause seek_rec.skips to be all set
        // to -1 again), and once seek_minus[0] is cur_filenr_pos is too small, skip_r will increase
        let mut skip_r = 0;
        let mut skip_blocks = 0_u8;


        let seek_plus:  Vec<i64> = params::DELTA.iter().map(|n| seek_filenr_pos + n).collect();
        let seek_minus: Vec<i64> = params::DELTA.iter().map(|n| seek_filenr_pos - n).collect();

        loop {
            let cur_rec: Record = records[cur_idx];
            stats.seeks += 1;
            trace!(logger, format!("FL# Seek now at {:?} ", cur_idx));

            if skip_blocks > 3 {

                let output_hash = self.hash();
                let tx_hash = self.to_transaction().hash();

                if spent_index.exists(output_hash) {
                    trace!(logger, format!("FL# Hash exists! {:?} == {:?}", output_hash,self));

                    return Err(SpendingError::OutputAlreadySpent);
                }

                if !spent_index.exists(tx_hash) {
                    return Err(SpendingError::OutputNotFound);
                }

                return Ok(stats);
            }
            // deduces that the wrong seek-val is set @ 106868
            //if seek_idx ==109401 { info!(logger, "FL# Seek now at {:?} ", cur_idx); }

            // See if there are skip-values we need to update in the record were seeking
            while skip_r < params::SKIP_FIELDS && seek_minus[skip_r] >= minimal_filenr_pos {
                skip_r += 1;
            }

            let diff: i64 = cur_idx as i64 - seek_idx as i64;

            if skip_r < 1 && diff > i8::min_value()  as i64 && diff < i8::max_value()  as i64 { seek_skips[0] = diff as i32;  seek_skip_blocks[0] = skip_blocks; }
            if skip_r < 2 && diff > i16::min_value()  as i64 && diff < i16::max_value()  as i64 { seek_skips[1] = diff as i32;  seek_skip_blocks[1] = skip_blocks;  }
            if skip_r < 3 && diff > i16::min_value() as i64 && diff < i16::max_value() as i64 { seek_skips[2] = diff as i32;  seek_skip_blocks[2] = skip_blocks;  }
            //if skip_r < 4 && diff > i32::min_value() as i64 && diff < i32::max_value() as i64 { seek_skips[3] = diff as i32 }


            match cur_rec {
                Record::OrphanBlock { .. } => {
                    // this must mean we've reached the end of the line as we wouldn't find an orphan
                    // block while connecting somewhere in the middle
                    return Err(SpendingError::OutputNotFound);
                },

                Record::Block { prev: p, .. } => {
                    stats.jumps += 1;
                    skip_blocks = skip_blocks.saturating_add(1);

                    cur_idx = p as usize;
                }

                Record::Transaction { file_number: f, file_offset: o, .. } => {
                    let cur_filenr_pos = ((f as i64) << 32) | o as i64;

                    if cur_filenr_pos == seek_filenr_pos {

                        /*if seek_output_index <= u8::max_value() as u32 {
                            *self = Record::Output {
                                file_number: f,
                                file_offset: o,
                                output_index: seek_output_index as u8,
                                skips: Skips {
                                    s1: seek_skips[0] as i8,
                                    s2: seek_skips[1] as i16,
                                    s3: seek_skips[2] as i16,
                                    b1: seek_skip_blocks[0],
                                    b2: seek_skip_blocks[1],
                                    b3: seek_skip_blocks[2]
                                }
                            };
                        }*/

                        // we've found the transaction of the output before we
                        // found the same output. So we're all good
                        stats.total_diff = cur_idx as i64 - seek_idx as i64;
                        return Ok(stats);
                    } else {

                        if minimal_filenr_pos > cur_filenr_pos {
                            minimal_filenr_pos = cur_filenr_pos;
                        }
                        cur_idx -= 1;
                        stats.total_move += 1;
                    }
                },

                Record::Output { file_number: f, file_offset: o, output_index: x, skips: s } => {
                    let cur_filenr_pos = ((f as i64) << 32) | o as i64;

                    if cur_filenr_pos == seek_filenr_pos  && x as u32 == seek_output_index {
                        trace!(logger, format!("FL# Already spent {:?} {:?}={:?}", cur_idx, x, seek_output_index));

                        return Err(SpendingError::OutputAlreadySpent);
                    }
                    let mut skip: i64 = -1;
                    for n in (0..params::SKIP_FIELDS).rev() {
                        if seek_plus[n] < cur_filenr_pos {
                            if n == 2 { skip = s.s3 as i64; skip_blocks = skip_blocks.saturating_add(s.b3); };
                            if n == 1 { skip = s.s2 as i64; skip_blocks = skip_blocks.saturating_add(s.b2); };
                            if n == 0 { skip = s.s1 as i64; skip_blocks = skip_blocks.saturating_add(s.b1); };

                            if minimal_filenr_pos > cur_filenr_pos - params::DELTA[n] {
                                minimal_filenr_pos = cur_filenr_pos - params::DELTA[n];
                            }
                            break;
                        }
                    }

                    stats.total_move += skip;
                    cur_idx = (cur_idx as i64 + skip) as usize;

                    if minimal_filenr_pos > cur_filenr_pos {
                        minimal_filenr_pos = cur_filenr_pos;
                    }
                },

                Record::OutputLarge { file_number: f, file_offset: o, output_index: x } => {
                    let cur_filenr_pos = ((f as i64) << 32) | o as i64;

                    if cur_filenr_pos == seek_filenr_pos  && x as u32 == seek_output_index {
                        trace!(logger, format!("FL# Already spent {:?} {:?}={:?}", cur_idx, x, seek_output_index));

                        return Err(SpendingError::OutputAlreadySpent);
                    }

                    if minimal_filenr_pos > cur_filenr_pos {
                        minimal_filenr_pos = cur_filenr_pos;
                    }
                    cur_idx -= 1;
                    stats.total_move += 1;

                },

                _ => {
                    panic!("Unexpected record type {:?} during set_seek", cur_rec);
                }
            }

*/
            /*
            let mut skip = -1;
            for n in (0..params::SKIP_FIELDS).rev() {
                if seek_plus[n] < cur_filenr_pos {
                    skip = cur_rec.skips[n];

                    if minimal_filenr_pos > cur_filenr_pos - params::DELTA[n] {
                        minimal_filenr_pos = cur_filenr_pos - params::DELTA[n];
                    }
                    break;
                }
            }*/


    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_of_record() {
        assert_eq!(::std::mem::size_of::<Record>(), 8);

    }
}
