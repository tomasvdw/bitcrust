

//! A TxPtr is a pointer to a transaction in the transaction-content
//! It contains a file_number, a file offset that together point to the data
//! And optionally a input-index.
//!
//! TxPtr's are stored in the TransactionIndex; if they contain an input index
//! they are used as `guard`: When a transaction is stored but the prev_tx needed
//! for validation is not found, a link to the input that still awaits validation
//! is stored as a guard in the prev_tx location in the TransactionIndex


use super::flatfileset::FlatFilePtr;
use super::hash_index::HashIndexGuard;

const INPUT_INDEX_NULL: u16 = 0xFFFF;

#[derive(Debug, PartialEq, Clone, Copy, Hash, Eq)]
pub struct TxPtr {
    file_offset: u32,
    file_number: i16,
    input_index: u16
}


impl FlatFilePtr for TxPtr {

    fn new(file_number: i16, file_offset: u64) -> Self {
        TxPtr {
            file_offset: file_offset as u32,
            file_number: file_number,
            input_index: INPUT_INDEX_NULL
        }
    }

    fn get_file_number(self) -> i16 { self.file_number }

    fn get_file_offset(self) -> u64 { self.file_offset as u64 }

}

impl HashIndexGuard for TxPtr {

    fn is_guard(self) -> bool { self.input_index != INPUT_INDEX_NULL }
}

impl TxPtr {
    pub fn get_input_index(self) -> u16 {

        debug_assert!(self.is_guard());

        self.input_index
    }

    pub fn to_input(self, input_index: u16) -> TxPtr {

        debug_assert!(input_index != INPUT_INDEX_NULL);

        TxPtr {
            file_offset: self.file_offset,
            file_number: self.file_number,
            input_index: input_index
        }
    }

    pub fn first() -> TxPtr {
        TxPtr {
            file_number: 0,
            file_offset: super::flatfile::INITIAL_WRITEPOS as u32,
            input_index: INPUT_INDEX_NULL
        }
    }

    pub fn offset(self, offset: u32) -> TxPtr {
        if self.file_offset + offset > super::MAX_CONTENT_SIZE as u32 {
            println!("Next file!");
            TxPtr {
                file_number: self.file_number + 1,
                file_offset: super::flatfile::INITIAL_WRITEPOS as u32,
                input_index: INPUT_INDEX_NULL
            }
        } else {
            TxPtr {
                file_number: self.file_number,
                file_offset: self.file_offset + offset,
                input_index: INPUT_INDEX_NULL
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use super::super::flatfile::INITIAL_WRITEPOS;
    #[test]
    fn test_skip()
    {
        let x = TxPtr::first();
        let y = x.offset(1000);
        assert_eq!(x.file_number, 0);
        assert_eq!(y.file_offset, 1000 + INITIAL_WRITEPOS as u32);

    }
}
