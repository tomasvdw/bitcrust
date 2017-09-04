

//! A BlockheaderPtr is a pointer to a blockheader in the blockheader-content fileset
//! It contains a file_number, a file offset that together point to the data

//!
//! It is not stored directly, but can be extracted from the Record structure, that is stored
//! in the Spend-Tree


use super::flatfileset::FlatFilePtr;

#[derive(Debug, Copy, Clone)]
pub struct BlockHeaderPtr {
    file_offset: u32,
    file_number: i16,
}


impl FlatFilePtr for BlockHeaderPtr {

    fn new(file_number: i16, file_offset: u64) -> Self {
        BlockHeaderPtr {
            file_offset: file_offset as u32,
            file_number: file_number
        }
    }

    fn get_file_number(self) -> i16 { self.file_number }

    fn get_file_offset(self) -> u64 { self.file_offset as u64 }

}
