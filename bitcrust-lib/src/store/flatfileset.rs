//!
//! A FlatFileSet provides access to a set of files with raw binary data
//!
//! Each file of the set has a fixed size
//! The header of a file consists of 16 bytes
//! Byte 0-3 are a magic number
//! Byte 4-7 indicate the current write position as a host-endian 32-bit integer
//! The other bytes of the header are reserved
//!
//! The flatfiles are suffixed with 4 hex-digits indicating the filenumber
//! An index to a file consists of a 16-bits signed filenumber followed by 32-bit filepos
//! This is passed around as a u64
//!

use std::path::{Path,PathBuf};
use std::slice;
use std::fs;
use std::io;

use std::io::{Write};

use itertools::Itertools;
use itertools::MinMaxResult::{NoElements, OneElement, MinMax};


use store::flatfile::FlatFile;

/// FlatFileSet is a sequential set of files in form of prefixNNNN where NNNN is
/// sequential signed 16 bit big-endian number.
///
/// An instance can be used as context to write and read from such set
pub struct FlatFileSet {
    path:       PathBuf,
    prefix:     &'static str,
    first_file: i16,
    last_file:  i16,
    maps:       Vec<Option<FlatFile>>,

    start_size: u32,
    max_size:   u32,
}


/// A FilePtr consists of a 16-bits signed filenumber and a 32-bits unsigned file-position
/// The first 16 bits are ignored
#[derive(Copy,Clone,PartialEq,Eq,Hash,Debug)]
pub struct FilePtr(u64);

impl FilePtr {
    pub fn new(fileno: i16, filepos: u32) -> FilePtr {
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
}

/// An error used internally for filenames that do not match the pattern
#[derive(Debug)]
struct FilenameParseError;


/// Interprets the given name as
/// prefixNNNN where NNNN is big-endian
/// 16-bit signed int, and returns the number
fn filename_to_fileno(prefix: &str, name: &Path) -> Result<i16, FilenameParseError> {

    fn char_to_hex(byte: u8) -> Result<i16, FilenameParseError> {
        Ok(match byte {
            b'A' ... b'F' => byte - b'A' + 10,
            b'a' ... b'f' => byte - b'a' + 10,
            b'0' ... b'9' => byte - b'0',
            _             => return Err(FilenameParseError)
        } as i16)
    }

    // grab the name utf
    let name = name
        .file_name().ok_or(FilenameParseError)?
        .to_str().ok_or(FilenameParseError)?;

    // check prefix
    if !name.starts_with(prefix) {
        return Err(FilenameParseError);
    }

    // check length
    let name = name.as_bytes();
    let p = prefix.len();
    if name.len() != p + 4 {
        return Err(FilenameParseError);
    }

    // parse hex-chars
    Ok(
        (char_to_hex(name[p  ])? as i16) << 12 |
        (char_to_hex(name[p+1])? as i16) << 8 |
        (char_to_hex(name[p+2])? as i16) << 4 |
        (char_to_hex(name[p+3])? as i16) << 0
    )
}

/// Constructs a pathname from a fileno
fn fileno_to_filename(path: &Path, prefix: &str, fileno: i16) -> PathBuf {

    PathBuf::from(path)
        .join(format!("{}{:02x}{:02x}",
              prefix,
              (fileno >> 8) & 0xFF,
              (fileno & 0xFF))
        )

}

/// Find the smallest and largest filenumbers with the given prefix
fn find_min_max_filenumbers(path: &Path, prefix: &str) -> (i16,i16) {

    let minmax = path
        .read_dir()
        .expect("Cannot read from data directory")
        .map   (|direntry| direntry.unwrap().path())
        .filter_map(|direntry| filename_to_fileno(prefix, &direntry).ok())
        .minmax();

    match minmax {
        NoElements    => (0,1),
        OneElement(n) => (n, n+1),
        MinMax(n,m)   => (n, m+1)
    }
}



impl FlatFileSet {

    /// Loads a fileset
    ///
    /// max_size is the size _after_ which to stop writing
    /// this means it needs enough space the largest possible write
    pub fn new(
        path:   &Path,
        prefix: &'static str,
        start_size: u32,
        max_size: u32)
    -> FlatFileSet {

        let (min,max) = find_min_max_filenumbers(path, prefix);


        FlatFileSet {
            path:       PathBuf::from(path),
            prefix:     prefix,
            start_size: start_size,
            max_size:   max_size,
            maps:       (min..max).map(|_| None).collect(),
            first_file: min,
            last_file:  max
        }
    }

    /// Returns a mutable reference to the given Flatfile
    ///
    /// Opens it first if needed
    fn get_flatfile(&mut self, fileno: i16) -> &mut FlatFile {

        // convert filenumber to index in file-vector
        let file_idx = (fileno - self.first_file) as usize;

        if self.maps[file_idx].is_none() {

            let name = fileno_to_filename(
                &self.path,
                self.prefix,
                fileno
            );

            self.maps[file_idx] = Some(FlatFile::open(
                &name,
                self.start_size
            ));
        }

        self.maps[file_idx].as_mut().unwrap()

    }

    /// Appends the slice to the flatfileset and returns a filepos
    ///
    /// Internally, this will ensure proper locking and creation of new files
    pub fn write(&mut self, buffer: &[u8]) -> FilePtr {


        let fileno     = self.last_file - 1;
        let max_size   = self.max_size;
        let buffer_len = buffer.len() as u32;
        let write_len  = buffer_len + 4; // including size-prefix

        // allocate some space to write
        let write_pos = self
            .get_flatfile(fileno)
            .alloc_write(write_len, max_size);


        match write_pos {
            None => {

                // create another file
                self.maps.push(None);
                self.last_file += 1;

                // call self using the new new last_file
                self.write(buffer)

            }

            Some(pos) => {
                // we have enough room;
                let flatfile = &mut self.get_flatfile(fileno);

                // write length & data
                flatfile.put(&buffer_len, pos as usize);
                flatfile.put_bytes(buffer, (pos + 4) as usize);

                FilePtr::new(fileno, pos  )
            }
        }

    }

    /// Reads the length-prefixed buffer at the given position
    pub fn read(&mut self, pos: FilePtr) -> &[u8] {

        let fileno   = pos.file_number();
        let filepos  = pos.file_pos();
        let map      = self.get_flatfile(fileno);

        let len: u32 = *map.get(filepos);
        map.get_bytes(filepos+4, len as usize)
    }
}


/* Tests */

#[test]
fn test_filename_to_fileno() {

    fn name_to_no(s: &'static str) -> i16 {
        filename_to_fileno("tx-", Path::new(s)).unwrap()
    }

    assert_eq!(0xab, name_to_no("tx-00ab"));
    assert_eq!(-1_i16, name_to_no("tx-ffff"));
    assert_eq!(255_i16, name_to_no("tx-00ff"));

}

#[test]
fn test_fileno_to_filename() {

    assert_eq!("/tmp/tx-0001",
            fileno_to_filename(
                &PathBuf::from("/tmp"),
                "tx-",
                1
            ).to_str().unwrap()
        );

    assert_eq!("/tmp/tx-fffe",
            fileno_to_filename(
                &PathBuf::from("/tmp"),
                "tx-",
                -2
            ).to_str().unwrap()
        );
}



#[cfg(test)]
mod tests {
    extern crate tempdir;
    extern crate rand;


    use std::fs;
    use std::thread;
    use std::collections;
    use std::path;
    use std::path::PathBuf;
    use self::rand::Rng;
    use super::*;


    #[test]
    fn flatfile_set() {
        let buf = [1_u8, 0, 0, 0];
        //let dir = tempdir::TempDir::new("test1").unwrap();
        let path = PathBuf::from(".");

        let mut ff = FlatFileSet::new(&path, "tx1-", 1000, 900);

        let in1 = ff.write(&buf);

        let out1 = ff.read(in1);

        assert_eq!(buf, out1);
        //fs::File::create(path.join("tx-FFFF")).unwrap().write_all(b"abc").unwrap();
        //fs::File::create(path.join("tx-0001")).unwrap().write_all(b"abc").unwrap();


    }

    #[test]
    fn test_concurrent() {
        let handles = (0..100).map(|_|
            thread::spawn(|| {
                let mut rng = rand::thread_rng();

                //let dir = ;//tempdir::TempDir::new("test1").unwrap();
                let path = PathBuf::from(".");//dir.path();
                let mut ff = FlatFileSet::new(&path, "tx2-", 10_000_000, 9_000_000);

                let mut map: collections::HashMap<FilePtr,  Vec<u8>> = collections::HashMap::new();

                for _ in 0..50 {

                    // create some nice data
                    let size: usize = rng.gen_range(10, 2000);
                    let mut buf = vec![0; size];
                    rng.fill_bytes(&mut buf.as_mut_slice());

                    let x = ff.write(buf.as_slice());

                    map.insert(x, buf);


                    // 3 gets
                    for _ in 0..30 {
                        let n: usize = rng.gen_range(0, map.len());

                        let v = map.values().nth(n).unwrap().as_slice();
                        let k = map.keys().nth(n).unwrap();
                        assert_eq!(v, ff.read(*k));
                        //assert_eq!(3,4);
                    }

                }
            })
        );

        for h in handles {
            h.join().unwrap();

        }
    }
}