use std::fs;
use std::slice;
use std::mem;
use std::ptr;

use std::path::{Path};
use memmap;

extern crate nix;
use std::os::unix::io::AsRawFd;
use self::nix::fcntl::{flock, FlockArg};

pub struct FlatFile {
    pub file: fs::File,
    pub map:  memmap::Mmap,
    pub ptr:  *mut u8

}

impl FlatFile {
    pub fn open(path: &Path) -> Self {

        let file = fs::OpenOptions::new().read(true).write(true).append(true).open(path)
            .expect(&format!("Failed to open file {:?} for writing", path));

        let mut map = memmap::Mmap::open(&file, memmap::Protection::ReadWrite)
            .expect("Failed to open memory map");

        let ptr = map.mut_ptr();

        FlatFile {
            file: file,
            map: map,
            ptr: ptr
        }
    }

    pub fn lock(&mut self) {

        let fd = self.file.as_raw_fd();
        flock(fd, FlockArg::LockExclusive).unwrap();
    }

    pub fn unlock(&mut self) {

        let fd = self.file.as_raw_fd();
        flock(fd, FlockArg::Unlock).unwrap();
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

    pub fn put_size(&self, size: u32) {
        self.put(&size, 4)
    }

    pub fn get_size(&self) -> u32 {
        *self.get(4)
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
        let flatfile = FlatFile::open(&path.join("tx-0001"));

        let val: &u32 = flatfile.get(0);
        assert_eq!(*val, 1);

    }

    #[test]
    fn test_seq() {
        let buf = [1_u8, 0, 0, 0];

        let dir = tempdir::TempDir::new("test1").unwrap();
        let path = PathBuf::from(".");
        {
            let _ = fs::File::create(path.join("tx-0001")).unwrap().write_all(&buf).unwrap();
        }
        let flatfile = FlatFile::open(&path.join("tx-0001"));

        let inval: u32 = 12;
        flatfile.put(&inval, 100);
        let val: &u32 = flatfile.get(100);
        assert_eq!(*val, inval);

    }
}