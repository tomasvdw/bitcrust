


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

use std::mem;
use std::sync::atomic;

use std::fmt::{Debug,Formatter,Error};

#[derive(Copy,Clone,PartialEq,Eq,Hash)]
pub struct FilePtr(u64);

impl FilePtr {
    pub fn new(fileno: i16, filepos: u32 ) -> FilePtr {
        FilePtr(
            (((fileno as u64) << 30) & 0x3FFF_C000_0000) |
                ((filepos as u64) & 0x3FFF_FFFF)
        )
    }

    pub fn new_input(fileno: i16, filepos: u32, input: u32 ) -> FilePtr {
        FilePtr(
               (((fileno as u64) << 30) & 0x0000_3FFF_C000_0000) |
                ((filepos as u64)       & 0x0000_0000_3FFF_FFFF) |
                                          0x2000_0000_0000_0000  |
               (((input as u64)  << 46) & 0x1FFF_C000_0000_0000)
        )
    }

    pub fn file_number(self) -> i16 {
        ((self.0 >> 30) & 0x3FFF) as i16
    }

    pub fn file_pos(self) -> usize {
        (self.0 & 0x3FFF_FFFF) as usize
    }

    fn meta(self) -> FilePtrMeta {
        let meta = (self.0 >> 46) & 0x3FFFF;

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

    /// Set self to `new_value` only if it is now set to `current_value`
    ///
    /// Returns true if the update has taken place
    pub fn atomic_replace(&self, current_value: FilePtr, new_value: FilePtr) -> bool {

        let atomic_self: *mut atomic::AtomicU64 = unsafe { mem::transmute( self ) };

        let prev = unsafe { (*atomic_self).compare_and_swap(current_value.0, new_value.0,
                                     atomic::Ordering::Relaxed) };

        prev == current_value.0

    }

    pub fn is_transaction(&self) -> bool {
        match self.meta() {
            FilePtrMeta::Transaction => true,
            _ => false
        }
    }

    pub fn is_input(&self) -> bool {
        match self.meta() {
            FilePtrMeta::TransactionInputIndex(_) => true,
            _ => false
        }
    }

}

#[derive(Debug)]
enum FilePtrMeta {

    Transaction,
    BlockHeader,
    TransactionOutputIndex(usize),
    TransactionOutputOffset(usize),
    TransactionInputIndex(usize),
    TransactionInputOffset(usize),
}


impl Debug for FilePtr {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "fi={},fp={}, mt={:?}", self.file_number(), self.file_pos(), self.meta())

        //fmt.write_str(&x)
    }
}

#[cfg(test)]
mod tests {
    extern crate tempdir;
    extern crate rand;
    use std::path::PathBuf;

    use std::thread;
    use std::time;

    use store::flatfileset::FlatFileSet;
    use super::*;
    use self::rand::Rng;


    #[test]
    fn test_atomic_update() {
        let dir = tempdir::TempDir::new("test1").unwrap();
        let path = dir.path();
        let mut ff1 = FlatFileSet::new(&path, "at-", 10_000_000, 9_000_000);
        let mut ff2 = FlatFileSet::new(&path, "at-", 10_000_000, 9_000_000);

        // write siome
        let p = FilePtr(123);
        let pos = ff1.write_fixed(&p);

        let p1: &FilePtr = ff1.read_fixed(pos);
        let p2: &FilePtr = ff2.read_fixed(pos);

        assert_eq!(p2.0, 123);

        assert_eq!(p1.atomic_replace(FilePtr(123),FilePtr(124)), true);
        assert_eq!(p1.atomic_replace(FilePtr(123),FilePtr(125)), false);
        assert_eq!(p1.atomic_replace(FilePtr(124),FilePtr(125)), true);

    }

    #[test]
    fn test_concurrency() {
        const THREADS: usize    = 50;
        const LOOPS: usize  = 10;


        let dir = tempdir::TempDir::new("test1").unwrap();
        let path = PathBuf::from(dir.path());//PathBuf::from("./tmp2");

        let mut ff1 = FlatFileSet::new(&path, "at-", 10_000_000, 9_000_000);
        let p = FilePtr(0);
        let pos = ff1.write_fixed(&p);


        let handles: Vec<_> = (0..THREADS).map(|_| {
            let path = path.clone();
            thread::spawn(move || {

                let mut ff2 = FlatFileSet::new(&path, "at-", 10_000_000, 9_000_000);
                let mut rng = rand::thread_rng();


                for lp in 0..LOOPS {
                    // CAS retry loop
                    loop {
                        let fp: &FilePtr = ff2.read_fixed(pos);
                        let fp_org = *fp;

                        let fp2 = FilePtr(fp_org.0 + 1);

                        let res = fp.atomic_replace(fp_org, fp2);
                        if res {
                            break;
                        }
                    }
                }
            })
        }).collect();


        for h in handles {
            h.join().unwrap();
        }

        // check total sum

        let p1: &FilePtr = ff1.read_fixed(pos);
        assert_eq!(p1.0, THREADS as u64 * LOOPS as u64);

    }

}


