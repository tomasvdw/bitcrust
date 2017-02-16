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
        skips:       [i16; params::SKIP_FIELDS]
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
        skips:       [i16; params::SKIP_FIELDS]
    },

    // If a referenced output doesn't exist at the time of block-insertion,
    // the record references an input instead
    UnmatchedInput

}

impl fmt::Debug for Record {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {

            Record::Block   { file_number: n, file_offset: o , prev: p } =>
                write!(fmt, "BLK  {0:>04X}:{1:08x}        (TO {2:24})", n, o, p),
            Record::Output  { file_number: n, file_offset: o , output_index: x, skips: s } =>
                write!(fmt, "OUT  {0:>04X}:{1:08x} i{2:<4}  ({3:06} {4:06} {5:06} {6:06})", n, o, x, s[0], s[1], s[2], s[3]),
            Record::Transaction  { file_number: n, file_offset: o , skips: s } =>
                write!(fmt, "TX   {0:>04X}:{1:08x}        ({2:06} {3:06} {4:06} {5:06})", n, o, s[0], s[1], s[2], s[3]),
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
            skips: [-1; params::SKIP_FIELDS]
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
                skips:        [-1; params::SKIP_FIELDS]
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
        RecordPtr (ptr )
    }


    pub fn to_index(self) -> u64 {
        self.0
    }

/*
    pub fn iter(self, fileset: &mut FlatFileSet) -> RecordBlockIterator {
        RecordBlockIterator {
            cur_ptr: self.next_in_block(),
            fileset: fileset
        }
    }
    */
}
/*
pub struct RecordBlockIterator<'a> {
    cur_ptr: RecordPtr,
    //fileset: &'a mut FlatFileSet<P
}


/// Browsing backwards over the entire tree
impl<'a> Iterator for RecordBlockIterator<'a> {
    type Item = RecordPtr;

    fn next(&mut self) -> Option<RecordPtr> {
        let rec: Record = *self.fileset.read_fixed::<Record>(self.cur_ptr.ptr);

        if rec.ptr.is_blockheader() {
            None
        } else {
            let result = self.cur_ptr;
            self.cur_ptr = self.cur_ptr.next_in_block();

            Some(result)
        }
    }
}
*/

impl Record {
    /*
    pub fn previous(&self, fileset: &mut FlatFileSet) -> Option<&Record> {
        match self.bits(0,2) {
            0 => Some( fileset.read_fixed(self.skips_as_fileptr()) ),
            1 => Some( self.before_in_memory(1) ),
            2 => Some( self.before_in_memory( self.skips_bit_3_to_16())),
            3 => None,
            _ => unreachable!()
        }
    }
    */

    pub fn seek(&self) -> Option<&Record> {
        None
    }

    /// This is a preliminary new. To set the proper skip pointers
    /// we must now where we are in the file so we do this aferwards in set_skips
    /*pub fn new(content: FilePtr) -> Self {
        Record {
            ptr: content,
            skips: [0; params::SKIP_FIELDS]
        }
    }*/

/*
    fn set_ptr_in_skips(&mut self, ptr: RecordPtr) {
        let cv: [u64; 1] = [ptr.ptr.to_u64()];
        self.skips = unsafe { mem::transmute(cv) };
    }

    fn get_ptr_from_skips(&self) -> RecordPtr {
        let cv: [u64; 1] = unsafe { mem::transmute(self.skips) };

        RecordPtr::new(FilePtr::from_u64(cv[0]))
    }
*/

    pub fn seek_and_set(
        &mut self,
        seek_idx: usize,
        records: &[Record],
        logger: &slog::Logger) -> Result<SpentTreeStats, SpendingError>

    {

        trace!(logger, format!("FL# Start search for {:?} {:?} {:?}", self, seek_idx, records[seek_idx]));

        let mut stats: SpentTreeStats = Default::default();

        // maybe we would like to set skips but there is nothing to actually verify
        if self.is_transaction() { return Ok(stats); }

        stats.inputs += 1;

        // cur  will walk through the skip-list, starting from
        // the previous record
        let mut cur_idx = seek_idx - 1;

        // this is the transaction position we seek (the filepos stripped of the output-index metadata)
        let seek_filenr_pos = self.filenumber_and_pos();
        let seek_output_index = self.output_index();

        // all filenr_pos we've passed are at least this high
        let mut minimal_filenr_pos = seek_filenr_pos;

        // these are the pointers that will be stored in rec. By default, they just point to the
        // previous
        let mut seek_skips = [-1; params::SKIP_FIELDS];

        // for lack of a better name, `skip_r` traces which of the four skips of seek_rec are still
        // "following" the jumps. Initially all are (which will cause seek_rec.skips to be all set
        // to -1 again), and once seek_plus[0] is cur_filenr_pos is too small, skip_r will increase
        let mut skip_r = 0;

        let seek_plus:  Vec<i64> = params::DELTA.iter().map(|n| seek_filenr_pos + n).collect();
        let seek_minus: Vec<i64> = params::DELTA.iter().map(|n| seek_filenr_pos - n).collect();




        loop {
            let cur_rec: Record = records[cur_idx];
            stats.seeks += 1;
            trace!(logger, format!("FL# Seek now at {:?} ", cur_idx));

            match cur_rec {
                Record::OrphanBlock { .. } => {
                    // this must mean we've reached the end of the line as we wouldn't find an orphan
                    // block while connecting somewhere in the middle
                    return Err(SpendingError::OutputNotFound);
                },

                Record::Block { prev: p, .. } => {
                    stats.jumps += 1;

                    cur_idx = p as usize;
                }

                Record::Transaction { file_number: f, file_offset: o, .. } => {
                    let cur_filenr_pos = ((f as i64) << 32) | o as i64;

                    if cur_filenr_pos == seek_filenr_pos {
                        // we've found the transaction of the output before we
                        // found the same output. So we're all good
                        stats.total_move = cur_idx as i64 - seek_idx as i64;
                        return Ok(stats);
                    } else {

                        if minimal_filenr_pos > cur_filenr_pos {
                            minimal_filenr_pos = cur_filenr_pos;
                        }
                        cur_idx -= 1;
                    }
                },

                Record::Output { file_number: f, file_offset: o, output_index: x, .. } => {
                    let cur_filenr_pos = ((f as i64) << 32) | o as i64;

                    if cur_filenr_pos == seek_filenr_pos  && x as u32 == seek_output_index {
                        trace!(logger, format!("FL# Already spent {:?} {:?}={:?}", cur_idx, x, seek_output_index));
                        return Err(SpendingError::OutputAlreadySpent);
                    }

                    if minimal_filenr_pos > cur_filenr_pos {
                        minimal_filenr_pos = cur_filenr_pos;
                    }

                    cur_idx -= 1;
                },

                Record::OutputLarge { .. } => {
                    unimplemented!()
                },

                _ => {
                    panic!("Unexpected record type {:?} during set_seek", cur_rec);
                }
            }

            // See if there are skip-values we need to update in the record were seeking
            if skip_r < params::DELTA.len() {
                let diff: i64 = cur_idx as i64 - seek_idx as i64;
                if diff > i16::min_value() as i64 && diff < i16::max_value() as i64 {
                    for n in skip_r..params::DELTA.len() {
                        seek_skips[n] = diff as i16;
                    }
                }

                while skip_r < params::SKIP_FIELDS && seek_minus[skip_r] >= minimal_filenr_pos {
                    skip_r += 1;
                }
            }



            /*
            let mut skip = -1;
            for n in (0..params::SKIP_FIELDS).rev() {
                if seek_plus[n] < cur_filenr_pos {
                    skip = cur_rec.skips[n];

                    stats.total_move += (skip as i64).abs();
                    stats.use_diff[n] += 1;

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
