use std::fs;
use std::slice;
use std::mem;
use std::ptr;
use std::thread;
use std::time;
use std::sync::atomic;
use std::sync::atomic::AtomicU32;
use std::io::Write;


use std::path::{Path};
use memmap;

use rand;
use rand::Rng;

const WRITEPOS_OFFSET: isize = 4;
const MAGIC_FILEID:u32       = 0x62634D4B;
const INITIAL_WRITEPOS:u32   = 0x10;

pub struct FlatFile {

    file:      fs::File,
    map:       memmap::Mmap,
    ptr:       *mut u8,
    write_ptr: *mut atomic::AtomicU32
}

impl FlatFile {

    /// Opens the flatfile with the given path and loads the corresponding memory map
    /// Atomically creates the file if needed with the given `initial_size`
    pub fn open(path: &Path, initial_size: u32) -> Self {

        let file = FlatFile::open_or_create(path, initial_size);

        let mut map = memmap::Mmap::open(&file, memmap::Protection::ReadWrite)
            .expect("Failed to open memory map");

        let ptr = map.mut_ptr();

        let write_ptr = unsafe { mem::transmute(ptr.offset(WRITEPOS_OFFSET)) };

        FlatFile {
            file:      file,
            map:       map,
            ptr:       ptr,
            write_ptr: write_ptr
        }
    }

    fn open_or_create(path: &Path, size: u32) -> fs::File {

        // first we try creating a new file
        let mut create_result = fs::OpenOptions::new().read(true).write(true).create_new(true).open(path);

        if let Ok(mut new_file) = create_result {

            // initialize file

            // we use transmute to ensure we're using native-endianness
            let magic:    &[u8;4] = unsafe { mem::transmute( &MAGIC_FILEID) };
            let writepos: &[u8;4] = unsafe { mem::transmute( &INITIAL_WRITEPOS) };
            new_file.write_all(magic);
            new_file.write_all(writepos);

            new_file.set_len(size as u64);

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
        panic!(format!("Data file '{:?}' exists but has invalid size", path));
    }


    pub fn get<T>(&self, filepos: usize) -> &'static T {

        unsafe {
            mem::transmute( self.ptr.offset(filepos as isize))
        }
    }

    pub fn put<T>(&self, value: &T, filepos: usize) {

        let target: &mut T = unsafe {
            mem::transmute( self.ptr.offset(filepos as isize))
        };

        unsafe {
            ptr::copy_nonoverlapping(value, target, 1);
        };
    }

    pub fn get_bytes(&self, filepos: usize, size: usize) -> &'static [u8] {
        unsafe {
            slice::from_raw_parts(self.ptr.offset(filepos as isize), size)
        }

    }

    pub fn put_bytes(&self, value: &[u8], filepos: usize) {

        let target: &mut u8 = unsafe {
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
    pub fn alloc_write(&self, size: u32, max_size: u32) -> Option<u32> {
        loop {

            let write_ptr = unsafe { &*self.write_ptr };

            let write_pos = write_ptr.load(atomic::Ordering::Relaxed);
            if write_pos > max_size {
                return None;
            }

            let old_write_pos = write_ptr.compare_and_swap
                (write_pos, write_pos + size, atomic::Ordering::Relaxed);

            if old_write_pos == write_pos {
                return Some(old_write_pos)
            }
        }

    }

}




#[cfg(test)]
mod tests {
    extern crate tempdir;

    use std::path;
    use std::path::PathBuf;
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