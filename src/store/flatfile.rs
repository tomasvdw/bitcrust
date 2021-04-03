//!
//! Wraps a single flat-file from a flat-file-set
//!
//! All access occurs through flatfileset
//!
//!

use std::fs;
use std::slice;
use std::mem;
use std::ptr;
use std::thread;
use std::time;
use std::sync::atomic;
use std::io::Write;
use std::fmt::{Debug,Formatter,Error};

use std::path::{Path};
use memmap;
//use std::cmp::Ordering;

const WRITEPOS_OFFSET  : isize = 8;
const MAGIC_FILEID     : u64   = 0x62634D4B_00000000;
pub const INITIAL_WRITEPOS : u64   = 0x10;

pub struct FlatFile {

    file:      Option<fs::File>,
    map:       Option<memmap::Mmap>,
    ptr:       *mut u8,
    write_ptr: *mut atomic::AtomicU64
}


impl Clone for FlatFile {

    /// Only explicit cloning is allowed
    fn clone(&self) -> FlatFile {

        FlatFile {
            file:     None,
            map:      None,
            ptr:      self.ptr,
            write_ptr:self.write_ptr
        }
    }
}

impl FlatFile {

    /// Opens the flatfile with the given path and loads the corresponding memory map
    /// Atomically creates the file if needed with the given `initial_size`
    pub fn open(path: &Path, initial_size: u64) -> Self {

        let file = FlatFile::open_or_create(path, initial_size);

        let mut map = memmap::Mmap::open(&file, memmap::Protection::ReadWrite)
            .expect("Failed to open memory map");

        let ptr = map.mut_ptr();

        let write_ptr = unsafe { mem::transmute(ptr.offset(WRITEPOS_OFFSET)) };

        FlatFile {
            file:      Some(file),
            map:       Some(map),
            ptr:       ptr,
            write_ptr: write_ptr
        }
    }

    fn open_or_create(path: &Path, size: u64) -> fs::File {

        // first we try creating a new file
        let create_result = fs::OpenOptions::new().read(true).write(true).create_new(true).open(path);

        if let Ok(mut new_file) = create_result {

            // initialize file

            // we use transmute to ensure we're using native-endianness
            // (admittedly, a questionable design decision).
            let magic:    &[u8;8] = unsafe { mem::transmute( &MAGIC_FILEID) };
            let writepos: &[u8;8] = unsafe { mem::transmute( &INITIAL_WRITEPOS) };

            new_file.write_all(magic)
                .expect(&format!("Could not write header to {}", path.to_str().unwrap_or("")));

            new_file.write_all(writepos)
                .expect(&format!("Could not write header to {}", path.to_str().unwrap_or("")));


            new_file.set_len(size as u64)
                .expect(&format!("Could not allocate {} bytes for {}", size, path.to_str().unwrap_or("")));

            return new_file;
        }


        // if we can't create, we'll try a few times to open it
        const RETRIES: isize = 50;
        for _ in 0..RETRIES {

            let open_result = fs::OpenOptions::new().read(true).write(true).open(path);

            if let Ok(file) = open_result {
                // check if full initialized using its length
                let file_len = file.metadata().expect("Cannot query file info").len();
                if file_len == size as u64 {
                    return file;
                }
            }
            thread::sleep(time::Duration::from_millis(50));
        }
        panic!("Data file '{:?}' exists but has invalid size", path)
    }


    /// Returns the object at the given filepos
    ///
    /// No checks are done: filepos must point to a correct location
    ///
    /// The resulting reference is static as it points to a never-closing memmap
    pub fn get<T>(&self, filepos: usize) -> &'static mut T {

        unsafe {
            mem::transmute( self.ptr.offset(filepos as isize))
        }
    }


    /// Stores an object at the given filepos
    ///
    /// It must be already verified to fit
    pub fn put<T>(&self, value: &T, filepos: usize) {

        let target: &mut T = unsafe {
            mem::transmute( self.ptr.offset(filepos as isize))
        };

        unsafe {
            ptr::copy_nonoverlapping(value, target, 1);
        };
    }

    pub fn get_slice<T>(&self, filepos: usize, size: usize) -> &'static mut [T] {
        unsafe {
            let typed_ptr: *mut T = mem::transmute(self.ptr.offset(filepos as isize));
            slice::from_raw_parts_mut(typed_ptr, size)
        }

    }

    pub fn put_slice<T>(&self, value: &[T], filepos: usize) {

        let target: &mut T = unsafe {
            mem::transmute( self.ptr.offset(filepos as isize))
        };

        unsafe {
            ptr::copy_nonoverlapping(value.get_unchecked(0), target, value.len());
        };
    }

    /// Reserves `size` bytes for writing, updates the write_pos atomically
    /// and returns the position at which the bytes can be written
    ///
    /// If no more then max_size bytes are available, None is returned
    pub fn alloc_write(&self, size: u64, max_size: u64) -> Option<u64> {

        // loop retries for compare-and-swap
        loop {

            let write_ptr = unsafe { &*self.write_ptr };

            let write_pos = write_ptr.load(atomic::Ordering::Relaxed);
            if write_pos > max_size {
                return None;
            }

            let old_write_pos = write_ptr.compare_exchange
                (write_pos, write_pos + size, atomic::Ordering::Relaxed, atomic::Ordering::Relaxed);

            // Only if we are overwriting the old value we are ok; otherwise retry
            if old_write_pos == Ok(write_pos) {
                return Some(old_write_pos.unwrap())
            }
        }

    }

}

impl Debug for FlatFile {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "{:?}, {:?}", self.file, self.map)

    }
}


#[cfg(test)]
mod tests {
    extern crate tempdir;

    use std::fs;
    use std::io::{Write};

    use super::*;

    #[test]
    fn test_get() {
        let buf = [1_u8, 0, 0, 0];

        let dir = tempdir::TempDir::new("test1").unwrap();
        let path = dir.path();
        {
            let _ = fs::File::create(path.join("tx-0001")).unwrap().write_all(&buf).unwrap();
        }
        let flatfile = FlatFile::open(&path.join("tx-0001"),4);

        let val: &u32 = flatfile.get(0);
        assert_eq!(*val, 1);

    }


}