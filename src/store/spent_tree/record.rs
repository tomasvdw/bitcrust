use slog;

use std::mem;
use std::fmt;

use store::{TxPtr, BlockHeaderPtr};
use store::FlatFilePtr;


use store::spent_tree::SpendingError;
use store::spent_tree::BlockPtr;
use store::spent_index::SpentIndex;

use store::flatfile::INITIAL_WRITEPOS;


// highest 2 bits are record-type
// 11 => start of block
// 10 => end of block
// 00 => transaction
// 01 => spent-output

const RECORD_TYPE:u64    = 0xC000_0000_0000_0000;
const START_OF_BLOCK:u64 = 0xC000_0000_0000_0000;
const END_OF_BLOCK:u64   = 0x8000_0000_0000_0000;
const TRANSACTION:u64    = 0x0000_0000_0000_0000;
const OUTPUT:u64         = 0x4000_0000_0000_0000;

// Record layout
// -------------
//
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

// a orphan start of block is a start of block without a previous
const ORPHAN_START_OF_BLOCK:u64 = 0xC000_0000_0000_0000;


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



    pub fn new_output(tx_ptr: TxPtr, output_index: u32) -> Record {
        assert!(output_index <= 0x3fff); // TODO: this might not be true; fallback is needed

        Record(
            OUTPUT |
                (output_index as u64) << 48 |
                (tx_ptr.get_file_number() as u64) << 32 |
                tx_ptr.get_file_offset()
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

    /// This creates a non-cryptographic but perfect "hash" of the transaction or output
    pub fn hash(self) -> u64 {

        debug_assert!(self.is_transaction() || self.is_output());

        // We drop 4 bits from the filen offset and, for an output  add 1 + the output-index
        // The result is just as unique but smaller; we just drop the info to find the transaction
        // or to find the transaction from an output
        // The resulting number is used for the spent-index


        ((self.0 & 0xFFFF_FFFF_FFFF) >> 4)          // file-offset and file-number
        + (self.0 >> 62)                            // the bit that indicates its an output
        + ((self.0 & 0x3FFF_0000_0000_0000) >> 48)  // output-index
    }


    fn to_transaction(self) -> Record {

        debug_assert!(self.is_output());

        Record(self.0 & 0x0000_FFFF_FFFF_FFFF)
    }


    // Test only as normally it makes no sense to mix up file_offsets from different record-types
    // in the same expression
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

    /// Checks if the output is not double-spent in the bit-index
    fn verify_spent_in_index(&mut self,
                             spent_index: &SpentIndex) -> Result<usize, SpendingError>
    {
        let seek_output      = *self;
        let seek_transaction = self.to_transaction();

        if spent_index.exists(seek_output.hash()) {

            return Err(SpendingError::OutputAlreadySpent);
        }
        else if !spent_index.exists(seek_transaction.hash()) {

            return Err(SpendingError::OutputNotFound);
        }
        else {

            return Ok(1);
        }

    }


    /// Verifies whether this this output is not already spent,
    /// and whether it is stored before self in the blockchain
    pub fn verify_spent(
        &mut self,
        spent_index: &SpentIndex,
        seek_idx: usize,
        records: &[Record],
        logger: &slog::Logger) -> Result<usize, SpendingError>

    {

        trace!(logger, format!("FL# Start search for {:?} {:?} {:?}", self, seek_idx, records[seek_idx]));

        if self.is_transaction() { return Ok(0) }

        let seek_output      = *self; // this may not be found
        let seek_transaction = self.to_transaction(); // this must be found

        debug_assert!(seek_output.is_output());
        debug_assert!(self.0 == records[seek_idx].0);

        let mut seek_idx = seek_idx as u64;

        let mut blocks = 0;

        seek_idx -= 1;
        loop {
            trace!(logger, format!("FL# Search  {:?} @ {:?}", self, seek_idx));


            // TODO: we need to be aware here of the chances of forking.
            // On initial load, the spent-index is always up-to-date after one block
            // so we should use 1 here
            if blocks >= 1 {

                return self.verify_spent_in_index(spent_index)
            }


            let seek_rec = records[seek_idx as usize];

            if seek_rec.0 == ORPHAN_START_OF_BLOCK {

                // Looks like we reached genesis
                return Err(SpendingError::OutputNotFound);
            }
            else if (seek_rec.0 & START_OF_BLOCK) == START_OF_BLOCK {

                // jump to previous block
                blocks += 1;
                seek_idx = seek_rec.0 & !START_OF_BLOCK;

                trace!(logger, format!("FL# Jump to {:?} @ {:?}", seek_rec, seek_idx));

            } else if seek_rec == seek_transaction {

                // Found tx before spent => all ok
                return Ok(1);
            }
            else if seek_rec == seek_output {

                return Err(SpendingError::OutputAlreadySpent);
            }
            else {

                seek_idx -= 1;
            }
        }

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
