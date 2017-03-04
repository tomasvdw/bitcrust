//!
//! A FlatFileSet provides access to a set of files with raw binary data
//!
//! Each file of the set has a fixed size
//! The header of a file consists of 16 bytes
//! Byte 0-3 are a magic number
//! Byte 4-7 indicate the current write position as a native-endian 32-bit integer
//! The other bytes of the header are reserved
//!
//! The flatfiles are suffixed with 4 hex-digits indicating the filenumber
//! An index to a file consists of a 16-bits signed filenumber followed by 30-bit filepos
//! This is passed around as a u64 [`FilePtr`]
//!

use std::path::{Path,PathBuf};
use std::mem;
use std::fs;

use itertools::Itertools;
use itertools::MinMaxResult::{NoElements, OneElement, MinMax};

use store::flatfile::FlatFile;



/// A FlatFilePtr is any object that references data in a flatfileset
pub trait FlatFilePtr {
    fn new(file_number: i16, file_offset: u64) -> Self;
    fn get_file_number(self) -> i16;
    fn get_file_offset(self) -> u64;
}


/// FlatFileSet is a sequential set of files in form of prefixNNNN where NNNN is
/// sequential signed 16 bit big-endian number.
///
/// An instance can be used as context to write and read from such set
///
/// `P` is the type of the pointer to data in the set; it should contain at least
/// a file-number and a file offset
pub struct FlatFileSet<P: FlatFilePtr + Copy + Clone> {
    path:       PathBuf,
    prefix:     &'static str,
    first_file: i16,
    last_file:  i16,
    files:       Vec<Option<FlatFile>>,

    start_size: u64,
    max_size:   u64,

    // We tie the FlatFilePtr to the structure to ensure one ptr type is used
    // for one flatfileset
    phantom:    ::std::marker::PhantomData<P>
}

impl<P : FlatFilePtr + Copy + Clone> Clone for FlatFileSet<P> {

    fn clone(&self) -> FlatFileSet<P> {
        let f = self.files.clone();

        FlatFileSet {
            path:       self.path.clone(),
            prefix:     self.prefix,
            first_file: self.first_file,
            last_file:  self.last_file,
            files:      f,
            start_size: self.start_size,
            max_size:   self.max_size,
            phantom:    ::std::marker::PhantomData
        }
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

/// Constructs a pathname from a filenumber
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



impl<P: FlatFilePtr + Copy + Clone> FlatFileSet<P> {

    /// Loads a fileset
    ///
    /// max_size is the size _after_ which to stop writing
    /// this means that files_size-max_size must be big enough to hold the largest possible write
    pub fn new(
        path:   &Path,
        prefix: &'static str,
        file_size: u64,
        max_size: u64) -> FlatFileSet<P> {

        assert!(file_size >= max_size);

        if !path.exists() {
            fs::create_dir_all(path)
                .expect(&format!("Could not create {:?}", path));
        }

        // Find the range of files currently on disk
        let (min,max) = find_min_max_filenumbers(path, prefix);

        FlatFileSet {
            path:       PathBuf::from(path),
            prefix:     prefix,
            start_size: file_size,
            max_size:   max_size,
            files:       (min..max).map(|_| None).collect(),
            first_file: min,
            last_file:  max,

            phantom:    ::std::marker::PhantomData
        }
    }

    /// Returns a mutable reference to the given Flatfile
    ///
    /// Opens it if needed
    fn get_flatfile(&mut self, fileno: i16) -> &mut FlatFile {

        // convert filenumber to index in file-vector
        let file_idx = (fileno - self.first_file) as usize;

        while self.files.len() <= file_idx {
            self.files.push(None);
        }

        if self.files[file_idx].is_none() {

            let name = fileno_to_filename(
                &self.path,
                self.prefix,
                fileno
            );

            self.files[file_idx] = Some(FlatFile::open(
                &name,
                self.start_size as u64
            ));
        }

        self.files[file_idx].as_mut().unwrap()

    }

    /// Returns a mutable reference to the given Flatfile
    ///
    /// Panics if it is not yet opened
    fn get_flatfile_readonly(&self, fileno: i16) -> &FlatFile {

        // convert filenumber to index in file-vector
        let file_idx = (fileno - self.first_file) as usize;

        if self.files[file_idx].is_none() {

            panic!("get_flatfile_readonly called, but file not yet open");
        }

        self.files[file_idx].as_ref().unwrap()

    }

    /// Reserves `size` bytes in the flatfileset
    ///
    /// Creates a new file if needed
    /// Allocation occurs atomically but lock-free
    /// Returns a pointer to where size bytes can be stored
    pub fn alloc_write_space(&mut self, size: u64) -> P {
        let fileno = self.last_file - 1;
        let max_size = self.max_size;

        // try to allocate some space in the last file
        let write_pos = self
            .get_flatfile(fileno)
            .alloc_write(size, max_size);


        match write_pos {
            None => {

                // we will create space for another file
                self.files.push(None);
                self.last_file += 1;

                // call self using the new new last_file
                self.alloc_write_space(size)

            }
            Some(pos) => {
                P::new(fileno, pos)
            }
        }
    }

    pub fn read_mut_slice<T>(&mut self, ptr: P, count: usize) -> &'static mut [T] {

        let flatfile   = self.get_flatfile(ptr.get_file_number());

        flatfile.get_slice(ptr.get_file_offset() as usize, count)
    }

    pub fn alloc_slice<T>(&mut self, count: usize) -> &'static [T] {

        let ptr        = self.alloc_write_space((mem::size_of::<T>() * count) as u64);
        let flatfile   = self.get_flatfile(ptr.get_file_number());

        flatfile.get_slice(ptr.get_file_offset() as usize, count)
    }

    /// Appends the slice to the flatfileset and returns a filepos
    ///
    /// Internally, this will ensure creation of new files
    pub fn write(&mut self, buffer: &[u8]) -> P {

        let buffer_len = buffer.len() as u32;
        let write_len  = buffer_len + 4; // including size-prefix

        let target_ptr = self.alloc_write_space(write_len as u64);

        let flatfile   = self.get_flatfile(target_ptr.get_file_number());

        // write size & buffer
        flatfile.put(&buffer_len,  target_ptr.get_file_offset() as usize);
        flatfile.put_slice(buffer, (target_ptr.get_file_offset() + 4) as usize);

        target_ptr
    }


    /// Appends the given value to the flatfileset and returns a filepos
    ///
    /// Internally, this will ensure creation of new files
    pub fn write_fixed<T>(&mut self, value: &T) -> P {

        let target_ptr = self.alloc_write_space(mem::size_of::<T>() as u64);

        let flatfile   = self.get_flatfile(target_ptr.get_file_number());

        flatfile.put(value, target_ptr.get_file_offset() as usize);

        target_ptr
    }

    /// Appends the elements of the slice to flatfileset and returns a pointer
    ///
    /// The element count is not stored
    pub fn write_all<T>(&mut self, value: &[T]) -> P {

        let target_ptr = self.alloc_write_space((value.len() * mem::size_of::<T>()) as u64 );

        let flatfile   = self.get_flatfile(target_ptr.get_file_number());

        flatfile.put_slice(value, target_ptr.get_file_offset() as usize);

        target_ptr
    }


    /// Reads the length-prefixed buffer at the given position
    pub fn read(&mut self, pos: P) -> &'static [u8] {

        let fileno   = pos.get_file_number();
        let filepos  = pos.get_file_offset() as usize;
        let file     = self.get_flatfile(fileno);

        let len: u32 = *file.get(filepos as usize);
        file.get_slice(filepos+4, len as usize)
    }

    /// Reads the fixed size buffer at the given position
    pub fn read_fixed<T>(&mut self, pos: P) -> &'static mut T {

        let fileno   = pos.get_file_number();
        let filepos  = pos.get_file_offset();
        let file     = self.get_flatfile(fileno);

        file.get(filepos as usize)
    }

    /// Reads the fixed size buffer at the given position
    /// But panics if this requires to map more files
    ///
    /// This allows it to be called with an immutable reference
    pub fn read_fixed_readonly<T>(&self, pos: P) -> &'static mut T {

        let fileno   = pos.get_file_number();
        let filepos  = pos.get_file_offset();
        let file     = self.get_flatfile_readonly(fileno);

        file.get(filepos as usize)
    }

    fn offset(&mut self, pos: P, bytes: usize) -> P {
        if pos.get_file_offset() > self.max_size {

            P::new(pos.get_file_number() + 1, super::flatfile::INITIAL_WRITEPOS)
        } else {

            P::new(pos.get_file_number(), pos.get_file_offset() + bytes as u64)
        }

    }

    pub fn read_set(&mut self, pos: P, count: usize) -> (Vec<&'static [u8]>, P) {

        let mut result = Vec::with_capacity(count);
        let mut pos = pos;
        for _ in 0..count {
            let blob = self.read(pos);
            result.push(blob);

            pos = self.offset(pos, blob.len() + 4);
        };

        (result, pos)
    }

    /// This is a special reader used for the reindex benchmark
    /// It reads and returns all blockheaders+txcount
    #[test]
    pub fn read_block_headers(&mut self) -> Vec<(&'static [u8], usize)> {
        let mut result = Vec::new();
        let mut pos = P::new(0, super::flatfile::INITIAL_WRITEPOS);
        loop {
            let blob = self.read(pos);
            if blob.len() == 0 {
                break;
            }
            pos = self.offset(pos, blob.len() + 4);
            let tx_count = self.read_fixed(pos);
            pos = self.offset(pos, mem::size_of::<usize>());
            result.push( (blob, *tx_count) );

        };

        result

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


    use std::thread;
    use std::collections;
    use std::path::PathBuf;
    use self::rand::Rng;
    use super::*;
    use store::TxPtr;


    #[test]
    fn flatfile_set() {
        let buf = [1_u8, 0, 0, 0];
        let dir = tempdir::TempDir::new("test1").unwrap();
        let path = dir.path();

        let mut ff: FlatFileSet<TxPtr> = FlatFileSet::new(&path, "tx1-", 1000, 900);

        let in1 = ff.write(&buf);

        let out1 = ff.read(in1);

        assert_eq!(buf, out1);

    }

    #[test]
    fn test_concurrent() {

        const THREADS: usize         = 50;
        const MAX_SIZE: usize        = 2000;
        const PUTS_PER_THREAD: usize = 10;
        const GETS_PER_PUT: usize    = 30;

        let dir = tempdir::TempDir::new("test1").unwrap();
        let path = PathBuf::from(dir.path());

        let handles: Vec<_> = (0..THREADS).map(|_| {
            let path = path.clone();
            thread::spawn(move || {

                let mut rng = rand::thread_rng();

                let mut ff = FlatFileSet::new(&path, "tx2-", 10_000_000, 9_000_000);

                let mut map: collections::HashMap<TxPtr, Vec<u8>> = collections::HashMap::new();

                for _ in 0..PUTS_PER_THREAD {
                    // create some nice data
                    let size: usize = rng.gen_range(10, MAX_SIZE);
                    let mut buf = vec![0; size];
                    rng.fill_bytes(&mut buf.as_mut_slice());

                    let x = ff.write(buf.as_slice());

                    map.insert(x, buf);


                    // 3 gets
                    for _ in 0..GETS_PER_PUT {
                        let n: usize = rng.gen_range(0, map.len());

                        let v = map.values().nth(n).unwrap().as_slice();
                        let k = map.keys().nth(n).unwrap();
                        assert_eq!(v, ff.read(*k));
                        //assert_eq!(3,4);
                    }
                }
            })
        }).collect();

        for h in handles {
            h.join().unwrap();

        }
    }
}