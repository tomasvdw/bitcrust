
use std::path::{Path,PathBuf};
use std::mem;
use std::slice;
use std::fs;
use std::io;

use memmap;



///
pub struct FlatFileSet {
    path:       PathBuf,
    prefix:     &'static str,
    first_file: i16,
    last_file:  i16,
    maps:       Vec<Option<memmap::Mmap>>,

    max_size:   u32,

    start_size: u32,
}


struct FilenameParseError;

impl FlatFileSet {


    /// Interprets the given name as
    /// prefixNNNN where NNNN is big-endian
    /// 16-bit signed int, and returns the number
    fn filename_to_fileno(&self, name: &Path) -> Result<i16, FilenameParseError> {

        fn charToHex(byte: u8) -> Result<i16, FilenameParseError> {
            Ok(match byte {
                b'A' ... b'F' => byte - b'A' + 10,
                b'a' ... b'f' => byte - b'a' + 10,
                b'0' ... b'9' => byte - b'0',
                _             => return Err(FilenameParseError)
            } as i16)
        }

        let name = name.file_name().ok_or(FilenameParseError);

        return Ok(3);
    }

    /// Loads a fileset
    ///
    /// max_size is the size _after_ which to stop writing
    /// this means it needs enough space
    pub fn new(
        path:   &Path,
        prefix: &'static str,
        max_size: u32,
        start_size: u32) -> FlatFileSet {

        if start_size == 0 || start_size > max_size {
            panic!("Invalid start_size");
        }

        let dir = path
            .read_dir()
            .expect("Cannot read from data directory")
            .map   (|direntry| direntry.unwrap().path())
            .filter(|direntry| direntry.starts_with(prefix))
            .map   (|direntry| direntry.starts_with(prefix))
        ;



        FlatFileSet {
            path: PathBuf::from(path),
            prefix: prefix,
            max_size: max_size,
            start_size: start_size,
            maps: Vec::new(),
            first_file: 0,
            last_file: 0
        }
    }

    fn load_map(&self, fileno: i16) -> &memmap::Mmap {
        unimplemented!();
    }

    pub fn write(buffer: &[u8]) -> u64 {

        unimplemented!();
    }

    pub fn read(&mut self, pos: u64) -> &[u8] {

        let file = (((pos >> 32) & 0xFFFF) as i32 - self.first_file as i32) as i16;
        let filepos = (pos & 0x7FFFFFFF) as isize;

        let map = match self.maps[file as usize] {

            None        => { self.load_map(file) },
            Some(ref m) => m
        };

        let p = map.ptr();
        let len: usize = unsafe {
            (*p.offset(filepos) as usize) |
            (*p.offset(filepos + 1) as usize) << 8 |
            (*p.offset(filepos + 1) as usize) << 16 |
            (*p.offset(filepos + 1) as usize) << 24
        };


        let result = unsafe { slice::from_raw_parts(p, len) };

        result
    }
}

