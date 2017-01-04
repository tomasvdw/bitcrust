


//! A FilePtr is a pointer to a location in a flatfile & some meta data.
//!
//! It consists of:
//!
//! 16-bits signed filenumber
//! 30-bits unsigned file-position
//!
//! 18 remaining bits meta data whose meaning depends on the context
//!
//! 1x   -> transaction-output-index(x)  (x=17-bits)
//! 001x -> transaction-input-index(x)   (x=15-bits)
//! 000x -> transaction                  (x=ignored)
//! 010x -> block                        (x=ignored)
//! 011x -> guard-block                  (x=ignored)
//!
//!
//! Note:
//! A transaction is max 1mb, an input is min. 41 bytes, an output is min. 9 bytes
//! So we have < 2^15 inputs, and < 2^17 outputs
//!
//! Note:
//! The hash-index & the spent-tree both uses fileptrs
//!
//! The hash-index stores all but transaction-output-index(x) fileptrs
//! The spent-tree stores block, transaction & transaction-output-index(x) fileptrs
//!

use std::mem;
use std::sync::atomic;

use std::fmt::{Debug,Formatter,Error};


// TODO: we should use set/get bits helpers instead of these masks

const MASK_TYPE:    u64 = 0xE000_0000_0000_0000; // 3-bits

const MASK_INDEX1:  u64 = 0x1FFF_C000_0000_0000; // 15-bits
const MASK_INDEX2:  u64 = 0x7FFF_C000_0000_0000; // 17-bits

const MASK_FILENO:  u64 = 0x0000_3FFF_C000_0000; // 16-bits
const MASK_FILEPOS: u64 = 0x0000_0000_3FFF_FFFF; // 30-bits

const TYPE_INPUT:       u64 = 0x2000_0000_0000_0000;
const TYPE_OUTPUT_MIN:  u64 = 0x8000_0000_0000_0000;
//const TYPE_OUTPUT_MAX:  u64 = 0xE000_0000_0000_0000;
const TYPE_BLOCK:       u64 = 0x4000_0000_0000_0000;
const TYPE_GUARD_BLOCK: u64 = 0x6000_0000_0000_0000;
const TYPE_TRANSACTION: u64 = 0x0000_0000_0000_0000;

/// A pointer to data in a flatfile
#[derive(Copy,Clone,PartialEq,Eq,Hash)]
pub struct FilePtr(u64);

impl FilePtr {

    /// Constructs a new _transaction_ fileptr
    /// It can be modified after to change the type
    pub fn new(fileno: i16, filepos: u32 ) -> FilePtr {

        let fileno  = fileno as u64;
        let filepos = filepos as u64;

        FilePtr(
            ((fileno << 30) & MASK_FILENO) |
            (filepos & MASK_FILEPOS)
        )
    }

    /// Creates a new fileptr from an existing one as input
    ///
    pub fn to_input(self, index: u32) -> FilePtr {

        let index = index as u64;

        FilePtr(
            self.0
                | TYPE_INPUT
                | ((index << 46) & MASK_INDEX1)
        )
    }


    /// Creates a new fileptr from an existing one as input
    ///
    pub fn to_output(self, index: u32) -> FilePtr {

        let index = index as u64;

        FilePtr(
            self.0
                | TYPE_OUTPUT_MIN
                | ((index << 46) & MASK_INDEX2)
        )
    }

    pub fn to_guardblock(self) -> FilePtr {

        FilePtr(
            self.0 | TYPE_GUARD_BLOCK
        )
    }

    pub fn to_block(self) -> FilePtr {

        FilePtr(
            self.0 | TYPE_BLOCK
        )
    }

    pub fn to_u64(self) -> u64 {
        self.0
    }

    pub fn from_u64(v: u64) -> Self {
        FilePtr(v)
    }

    pub fn input_index(self) -> u32 {
        ((self.0 & MASK_INDEX1) >> 46) as u32
    }

    pub fn output_index(self) -> u32 {
        ((self.0 & MASK_INDEX2) >> 46) as u32
    }

    pub fn file_number(self) -> i16 {

        ((self.0 & MASK_FILENO) >> 30)  as i16
    }

    pub fn file_pos(self) -> usize {
        (self.0 & MASK_FILEPOS) as usize
    }


    pub fn offset(self, offset : i32) -> FilePtr

    {
        FilePtr((self.0 as i32 + offset) as u64)
    }

    /// We use a null value as magic NULL value. This is safe
    /// because we're never pointing to the header
    pub fn is_null(&self) -> bool {
        self.0 == 0
    }

    pub fn null() -> FilePtr {
        FilePtr(0)
    }


    /// Set self to `new_value` only if it is now set to `current_value` using Compare-And-Swap
    /// semantics
    ///
    /// Returns true if the update has taken place
    pub fn atomic_replace(&self, current_value: FilePtr, new_value: FilePtr) -> bool {

        let atomic_self: *mut atomic::AtomicU64 = unsafe { mem::transmute( self ) };

        let prev = unsafe {
            (*atomic_self).compare_and_swap(
                    current_value.0,
                    new_value.0,
                    atomic::Ordering::Relaxed)
        };

        prev == current_value.0

    }



    pub fn is_transaction(&self) -> bool {
        (self.0 & MASK_TYPE) == TYPE_TRANSACTION
    }

    pub fn is_input(&self) -> bool {
        (self.0 & MASK_TYPE) == TYPE_INPUT
    }

    pub fn is_blockheader(&self) -> bool {
        (self.0 & MASK_TYPE) == TYPE_BLOCK
    }

    pub fn is_guard_blockheader(&self) -> bool {
        (self.0 & MASK_TYPE) == TYPE_GUARD_BLOCK
    }

    pub fn is_output(&self) -> bool {
        (self.0 & TYPE_OUTPUT_MIN) == TYPE_OUTPUT_MIN
    }


}



impl Debug for FilePtr {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        if self.is_transaction() {
            write!(fmt, "TX  {:x}.{:x} T={:x}", self.file_number(), self.file_pos(), self.0)
        } else if self.is_blockheader() {
            write!(fmt, "BLK {:x}.{:x} T={:x}", self.file_number(), self.file_pos(), self.0)
        } else if self.is_guard_blockheader() {
            write!(fmt, "BLG {:x}.{:x} T={:x}", self.file_number(), self.file_pos(), self.0)
        }
        else if self.is_input() {
            write!(fmt, "INP {:x}.{:x} idx={:x} T={:x}", self.file_number(), self.file_pos(), self.input_index(), self.0)
        }
        else if self.is_output() {
            write!(fmt, "OUT {:x}.{:x} idx={:x} T={:x}", self.file_number(), self.file_pos(), self.output_index(), self.0)
        }
        else {
            write!(fmt, "ERR T={:x}", self.0)
        }



        //fmt.write_str(&x)
    }
}

#[cfg(test)]
mod tests {
    extern crate tempdir;
    extern crate rand;
    use std::path::PathBuf;

    use std::thread;

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


                for _ in 0..LOOPS {
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


