#[macro_use]
use slog;

use std::mem;
use std::fmt;

use store::{TxPtr, BlockHeaderPtr};
use store::FlatFilePtr;

use store::flatfileset::FlatFileSet;

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
#[derive(Debug, Copy, Clone)]
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
                Record::Block { file_number: n, file_offset: o, prev: previous.start.to_index() },

            _ => panic!("Expecting orphan block record")
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
            _ => panic!("transaction record expected")

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

    // test only as normally it makes no sense to treat file_offsets from different record-types
    // as a single value
    #[cfg(test)]
    pub fn get_file_offset(self) -> u32 {
        match self {
            Record::OrphanBlock { file_offset: f, .. } => f,
            Record::Block { file_offset: f, .. } => f,
            Record::Transaction { file_offset: f, .. } => f,
            Record::Output { file_offset: f, .. } => f,
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
        trace!(logger, format!("FL# Start search for {:?} {:?} ", self, seek_idx));

        let mut stats: SpentTreeStats = Default::default();
        stats.inputs += 1;
        Ok(stats)
        /*
        // cur  will walk through the skip-list, starting from
        // the previous record
        let mut cur_idx = seek_idx - 1;

        // this is the transaction position we seek (the filepos stripped of the output-index metadata)
        let seek_filenr_pos = self.ptr.filenumber_and_pos();

        // all filenr_pos we've passed are at least this high
        let mut minimal_filenr_pos = seek_filenr_pos;

        // these are the pointers that will be stored in rec. By default, they just point to the
        // previous
        self.skips = [-1; params::SKIP_FIELDS];


        // for lack of a better name, `skip_r` traces which of the four skips of seek_rec are still
        // "following" the jumps. Initially all are (which will cause seek_rec.skips to be all set
        // to -1 again), and once seek_plus[0] is cur_filenr_pos is too small, skip_r will increase
        let mut skip_r = 0;

        let seek_plus:  Vec<i64> = params::DELTA.iter().map(|n| seek_filenr_pos + n).collect();
        let seek_minus: Vec<i64> = params::DELTA.iter().map(|n| seek_filenr_pos - n).collect();

        let is_tx = self.ptr.is_transaction();

        //if is_tx { return Ok(0); }

        loop {

            let cur_rec: Record = records[cur_idx];

            stats.seeks += 1;

            let cur_filenr_pos = cur_rec.ptr.filenumber_and_pos();

            if cur_rec.skips == [0; params::SKIP_FIELDS] {
                // we're at the end

                if is_tx {
                    // a transaction is supposed to be not found
                    trace!(logger, format!("FL# Done after {:?} seeks", stats.seeks));

                    return Ok(stats);
                } else {
                    //panic!(format!("Output {:?} not found at {:?}", seek_rec, cur.ptr));
                    return Err(SpendingError::OutputNotFound);
                }
            } else if cur_rec.ptr.is_blockheader() || cur_rec.ptr.is_guard_blockheader() {
                // blockheaders don't have skips; interpret as a full fileptr
                trace!(logger, format!("FL# {:?}-{:?} JUMPING", cur_idx, cur_rec));

                stats.jumps += 1;

                cur_idx = cur_rec.get_ptr_from_skips().to_index();
                continue;
            }

            if seek_filenr_pos == cur_filenr_pos {
                if is_tx || cur_rec.ptr.is_transaction() {
                    // we've found the transaction of the output before we
                    // found the same output. So we're all good

                    return Ok(stats);

                } else if cur_rec.ptr.output_index() == self.ptr.output_index() {

                    //panic!(format!("Output {:?} double spent {:?}", seek_rec, cur.ptr));
                    return Err(SpendingError::OutputAlreadySpent);
                }
            }


            // See if there are skip-values we need to update in the record were seeking
            if skip_r < params::DELTA.len() {
                let diff: i64 = cur_idx as i64 - seek_idx as i64;
                if diff > i16::min_value() as i64 && diff < i16::max_value() as i64 {
                    for n in skip_r..params::DELTA.len() {
                        self.skips[n] = diff as i16;
                    }
                }
                /*else {
                    println!("Proc Diff too large {:?}", diff);
                }*/

                if minimal_filenr_pos > cur_filenr_pos {
                    minimal_filenr_pos = cur_filenr_pos;
                }


                while skip_r < params::SKIP_FIELDS && seek_minus[skip_r] >= minimal_filenr_pos {
                    skip_r += 1;


                    if is_tx && skip_r >= params::TX_NEEDED_SKIPS {

                        return Ok(stats);
                    }
                }
            }

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
            }


            /*if full_dump {
                let mut cur = RecordPtr::new(FilePtr::new(0, (16 + cur_idx * mem::size_of::<Record>()) as u32));
                let target = cur.offset(skip);
                loop {
                    let c: &Record = fileset.read_fixed(cur.ptr);
                    println!("FL# {:?}-{:?} <seek {:?}={:?}>", cur, c, skip, target);
                    cur = cur.prev(fileset);
                    let c: &Record = fileset.read_fixed(cur.ptr);

                    if cur.ptr == target.ptr {
                        println!("FL# {:?}-{:?} FOUND", cur, c);
                        break;
                    }
                }

            }*/

            cur_idx = (cur_idx as i64 + skip as i64) as usize;
        }
        */
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
