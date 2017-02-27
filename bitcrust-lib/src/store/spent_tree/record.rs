#[macro_use]
use slog;

use std::mem;
use std::fmt;

use store::{TxPtr, BlockHeaderPtr};
use store::FlatFilePtr;


use store::spent_tree::SpendingError;
use store::spent_tree::BlockPtr;

use store::flatfile::INITIAL_WRITEPOS;
use super::SpentTreeStats;

use super::params;

#[derive(Copy, Clone)]
pub struct Skips {
    pub b1: u8,
    pub b2: u8,
    pub b3: u8,
    pub s1: i8,
    pub s2: i16,
    pub s3: i16,

}

impl Skips {
    fn new() -> Skips
    {
        Skips { s1:-1, s2: -1, s3: -1, b1:0, b2:0, b3: 0 }
    }
}

/// A record is a 16 byte structure that points to either a
/// * blockheader
/// * transaction
/// * transaction-output
///
/// The skips point to other Records; at least the previous.
///
/// The exact format is still in work-in-progress.
///
#[derive(Copy, Clone)]
pub enum Record {

    OrphanBlock {
        file_number: i16,
        file_offset: u32,
    },
    Block {
        file_number: i16,
        file_offset: u32,
        prev:        u64
    },
    Transaction {
        file_number: i16,
        file_offset: u32,
        skips:       Skips, //[i16; params::SKIP_FIELDS]
    },
    OutputLarge {
        file_number:  i16,
        file_offset:  u32,
        output_index: u32

    },
    Output {
        output_index: u8,
        file_number:  i16,
        file_offset:  u32,
        skips:       Skips, //[i16; params::SKIP_FIELDS]
    },

    // If a referenced output doesn't exist at the time of block-insertion,
    // the record references an input instead
    UnmatchedInput

}

impl fmt::Debug for Record {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {

            Record::Block   { file_number: n, file_offset: o , prev: p } =>
                write!(fmt, "BLK  {0:>04X}:{1:08x}        (TO {2:29})", n, o, p),
            Record::Output  { file_number: n, file_offset: o , output_index: x, skips: s } =>
                write!(fmt, "OUT  {0:>04X}:{1:08x} i{2:<4}  ({3:06} {4:06} {5:06} {6:03} {7:03} {8:03})", n, o, x, s.s1, s.s2, s.s3,s.b1,s.b2,s.b3),
            Record::OutputLarge  { file_number: n, file_offset: o , output_index: x } =>
                write!(fmt, "OUL  {0:>04X}:{1:08x} i{2:<6}                                  ", n, o, x),
            Record::Transaction  { file_number: n, file_offset: o , skips: s } =>
                write!(fmt, "TX   {0:>04X}:{1:08x}        ({2:06} {3:06} {4:06} {5:03} {6:03} {7:03})", n, o, s.s1, s.s2, s.s3,s.b1,s.b2,s.b3),
            _ =>
                write!(fmt, "???")
        }
    }
}

impl Record {
    pub fn new_unmatched_input() -> Record {
        Record::UnmatchedInput
    }

    pub fn new_transaction(tx_ptr: TxPtr) -> Record {

        Record::Transaction {
            file_number: tx_ptr.get_file_number(),
            file_offset: tx_ptr.get_file_offset() as u32,
            skips: Skips::new()
        }
    }

    pub fn new_orphan_block(block_header_ptr: BlockHeaderPtr) -> Record {
        Record::OrphanBlock {
            file_number: block_header_ptr.get_file_number(),
            file_offset: block_header_ptr.get_file_offset() as u32

        }

    }

    pub fn new_block(previous: BlockPtr, orphan_block_record: Record) -> Record {

        match orphan_block_record {
            Record::OrphanBlock { file_number: n, file_offset: o } =>
                Record::Block { file_number: n, file_offset: o, prev: previous.start.to_index() + previous.length - 1 },

            _ => panic!("Expecting orphan block record. Tried to link {:?} to prev {:?} ", orphan_block_record, previous)
        }

    }


    pub fn new_output(txptr: TxPtr, output_index: u32) -> Record {
        if output_index <= u8::max_value() as u32 {
            Record::Output {
                output_index: output_index as u8,
                file_number:  txptr.get_file_number(),
                file_offset:  txptr.get_file_offset() as u32,
                skips:        Skips::new()
            }
        }
        else {
            Record::OutputLarge {
                output_index: output_index ,
                file_number:  txptr.get_file_number(),
                file_offset:  txptr.get_file_offset()  as u32,
            }
        }

    }

    pub fn is_transaction(self) -> bool {
        match self {
            Record::Transaction { ..}  => true,
            _ => false
        }
    }

    pub fn get_transaction_ptr(self) -> TxPtr {
        match self {
            Record::Transaction { file_number: n, file_offset: o, .. } => TxPtr::new(n,o as u64),
            _ => panic!("get_transaction_ptr transaction record expected, got  {:?}", self)

        }
    }

    pub fn get_block_header_ptr(self) -> BlockHeaderPtr {
        match self {
            Record::Block       { file_number: n, file_offset: o, .. } => BlockHeaderPtr::new(n,o as u64),
            Record::OrphanBlock { file_number: n, file_offset: o, .. } => BlockHeaderPtr::new(n,o as u64),
            _ => panic!("transaction record expected")

        }
    }

    pub fn is_output(self) -> bool {
        match self {
            Record::Output { .. }      => true,
            Record::OutputLarge {..} => true,
            _ => false
        }

    }

    pub fn is_block(self) -> bool {
        match self {
            Record::Block { .. }      => true,
            Record::OrphanBlock {..} => true,
            _ => false
        }

    }

    pub fn is_unmatched_input(self) -> bool {
        match self {
            Record::UnmatchedInput { .. } => true,
            _ => false
        }
    }

    /// This creates a non-cryptographic perfect hash of the transaction or output
    pub fn hash(self) -> [u8;8] {


        fn gen_hash(file_number: i16, file_offset: u32, output: Option<u32>) -> [u8;8] {

            // we can add the output index to the file-offset because the output-count
            // is always smaller then the transaction size
            let f = file_offset + output.map(|x| x+1).unwrap_or(0);
            let n = file_number as u32;

            [
                 (f >> 8) as u8,
                 (f >> 0) as u8,
                 (n >> 0) as u8,
                 (f >> 16) as u8,
                 (n >> 8) as u8,
                 (f >> 24) as u8,
                 0 as u8,
                 0 as u8
            ]
        }

        match self {
            Record::Transaction { file_number: n, file_offset: f, .. } =>                  gen_hash(n, f, None),
            Record::Output      { file_number: n, file_offset: f, output_index: o, .. } => gen_hash(n, f, Some(o as u32)),
            Record::OutputLarge { file_number: n, file_offset: f, output_index: o, .. } => gen_hash(n, f, Some(o as u32)),
            Record::Block       { file_number: n, file_offset: f, .. } =>                  gen_hash(n, f, None),
            _ => unreachable!()
        }
    }

    fn filenumber_and_pos(self) -> i64 {
        match self {
            Record::Transaction { file_number: n, file_offset: f, .. } => ((n as i64) << 32)  |  f as i64,
            Record::Output      { file_number: n, file_offset: f, .. } => ((n as i64) << 32)  |  f as i64,
            Record::OutputLarge { file_number: n, file_offset: f, .. } => ((n as i64) << 32)  |  f as i64,
            _ => unreachable!()
        }
    }

    fn output_index(self) -> u32 {
        match self {
            Record::Output      { output_index: x, .. } => x as u32,
            Record::OutputLarge { output_index: x, .. } => x as u32,
            _ => unreachable!()
        }
    }

    // test only as normally it makes no sense to treat file_offsets from different record-types
    // as a single value
    #[cfg(test)]
    pub fn get_file_offset(self) -> u32 {
        match self {
            Record::OrphanBlock { file_offset: f, .. } => f,
            Record::Block       { file_offset: f, .. } => f,
            Record::Transaction { file_offset: f, .. } => f,
            Record::Output      { file_offset: f, .. } => f,
            Record::OutputLarge { file_offset: f, .. } => f,
            _ => unimplemented!()
        }
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
        seek_idx: usize,
        records: &[Record],
        logger: &slog::Logger) -> Result<SpentTreeStats, SpendingError>

    {


        trace!(logger, format!("FL# Start search for {:?} {:?} {:?}", self, seek_idx, records[seek_idx]));

        let mut stats: SpentTreeStats = Default::default();

        if self.is_transaction() { return Ok(stats); }

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

                        if seek_output_index <= u8::max_value() as u32 {
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
                        }

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_of_record() {
        assert_eq!(::std::mem::size_of::<Record>(), 16);

    }
}
