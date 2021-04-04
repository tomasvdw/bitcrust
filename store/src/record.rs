
use std::fmt;

// Record layout
// -------------
//
// TRANSACTION:
// bits 0 -45   valueptr of transaction >> 3
// bits 46-62   zero
// bits 63      1
//
// PREVOUT:
// bits 0 -45   valueptr of transaction >> 3
// bits 46-62   output index + 1
// bits 63      1
//
// PREVOUT COINBASE:
// bits 0 -45   0
// bits 46-62   0
// bits 63      1
//
// START-Of BLOCK
// bits 1-62    record-count
// bits 63      0

//
//
// fileoffset == 0  => transaction not found at the time
// fileoffset == -1 => script validation failed


#[derive(Clone,Copy,PartialEq, Serialize, Deserialize)]
pub struct Record(pub u64);

impl fmt::Debug for Record {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "REC  {0:016X} ", self.0)
    }
}

impl Record {

    // Various constructors
    pub fn new_transaction(tx_ptr: ::ValuePtr) -> Record {

        Record((1<<63) | ((tx_ptr >> 3) & 0x1fff_ffff_ffff))
    }

    pub fn new_coinbase() -> Record {

        Record(1<<63)
    }

    pub fn new_output(tx_ptr: ::ValuePtr, output: u32) -> Record {
        Record((1<<63) | ((tx_ptr >> 3) & 0x1fff_ffff_ffff) | ((output as u64) << 46))
    }

    pub fn new_start_of_block(record_count: usize) -> Record {
        Record(record_count as u64)
    }

    pub fn to_bytes<'a>(records: &'a Vec<Record>) -> &'a[u8] {
        let size = records.len() * 8;
        unsafe { ::std::slice::from_raw_parts(records.as_ptr() as *const u8, size) }
    }

    pub fn get_output_index(&self) -> u32 {
        debug_assert_eq!(self.0 & (1<<63), (1<<63));
        (self.0 >> 46 & ((1 << 17)-1)) as u32
    }
}

