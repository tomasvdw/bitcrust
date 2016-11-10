


//! A FilePtr is a pointer to a location in a flatfile.
//!
//! It consists of:
//! 16-bits signed filenumber
//! 30-bits unsigned file-position
//!
//! 18 remaining meta data which meaning depends on the context
//!
//! 1x     -> transaction-output-index(x)  (17-bits)
//! 001x   -> transaction-inpout-index(x)  (15-bits)
//! 010x   -> transaction-output-offset(x) (15-bits)
//! 011x   -> transaction-input-offset(x)  (15-bits)
//! 000x -> transaction
//! 000x -> block
//!
//!
//!
//! A tx is max 1mb, an input is min. 41 bytes, an output is min. 9 bytes
//! So we have max 2^15 inputs, and max 2^17 outputs
//!
//! Offsets to inputs/output are faster as they can be directly read
//! But they do not always fit
//!
//! The index & the block-spent-tree both uses fileptrs
//!
//! The index stores a "transaction"-fileptr for a validated transaction
//!
//!

#[derive(Copy,Clone,PartialEq,Eq,Hash,Debug)]
pub struct FilePtr(u64);

impl FilePtr {
    pub fn new(fileno: i16, filepos: u32 ) -> FilePtr {
        FilePtr(
            (((fileno as u64) << 32) & 0xFFFF_0000_0000) |
                ((filepos as u64) & 0xFFFF_FFFF)
        )
    }
    pub fn file_number(self) -> i16 {
        ((self.0 >> 32) & 0xFFFF) as i16
    }

    pub fn file_pos(self) -> usize {
        (self.0 & 0xFFFF_FFFF) as usize
    }

    pub fn meta(self) -> FilePtrMeta {
        let meta = (self.0 >> 48) & 0xFFFF;

        // check high three bits
        match meta & 0xE000 {

            0x8000 ... 0xE000 =>    FilePtrMeta::TransactionOutputIndex((meta & 0x7fff) as usize),
            0x2000 =>               FilePtrMeta::TransactionInputIndex((meta & 0x1fff) as usize),
            0x4000 =>               FilePtrMeta::TransactionOutputOffset((meta & 0x1fff) as usize),
            0x6000 =>               FilePtrMeta::TransactionInputOffset((meta & 0x1fff) as usize),
            0x0000 =>               if (meta & 0x1000) == 0x1000 {
                                        FilePtrMeta::BlockHeader
                                    } else {
                                        FilePtrMeta::Transaction
                                    },

            _ => panic!("unknown fileptr meta-data")

        }

    }

    pub fn is_null(&self) -> bool {
        self.0 == 0
    }

    pub fn null() -> FilePtr {
        FilePtr(0)
    }


}

pub enum FilePtrMeta {

    Transaction,
    BlockHeader,
    TransactionOutputIndex(usize),
    TransactionOutputOffset(usize),
    TransactionInputIndex(usize),
    TransactionInputOffset(usize),
}