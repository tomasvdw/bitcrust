extern crate memmap;

use bincode;

use std::sync::atomic;

use std::{io, fs, mem, path};
use std::io::Write;
use timer::Timer;
use header;

use io::*;
use values::*;

/// Any `HashStoreError` returned indicates corruption of the database
/// or a non-recoverable IO problem
#[derive(Debug)]
pub enum HashStoreError {
    IoError(io::Error),
    InvalidMagicFileId,
    InvalidRootBits,
    Other
}

impl From<io::Error> for HashStoreError {
    fn from(err: io::Error) -> HashStoreError {
        HashStoreError::IoError(err)
    }
}

impl From<bincode::Error> for HashStoreError {
    fn from(err: bincode::Error) -> HashStoreError {
        match *err {
            bincode::ErrorKind::Io(e) => HashStoreError::IoError(e),
            _ => HashStoreError::Other
        }
    }
}

pub enum SearchDepth {
    FullSearch,
    SearchAfter(u32)
}

impl SearchDepth {
    fn check(&self, time: u32) -> bool {
        match *self {
            SearchDepth::FullSearch     => true,
            SearchDepth::SearchAfter(x) => time >= x
        }

    }
}

enum HashStoreStats {
    Elements = 0,
    WriteTime = 1,
    ReadTime = 2,

}



/// Handle to a hashstore database
///
/// This provides get and set operations
///
/// # Example
///
/// let hs = hashstore::HashStore::new("test", 24);
///
pub struct HashStore {
    // 2 handles to the same file
    rw_file:     fs::File,
    append_file: fs::File,

    // memory map to root table
    _mmap: memmap::Mmap,
    root:    &'static [atomic::AtomicU64],
    stats:   &'static [atomic::AtomicU64],
    extrema: &'static [atomic::AtomicU64],

    root_bits: u8,
}


impl HashStore {

    /// Creates or opens a hashstore
    ///
    /// `root_bits` is the number of bits of each key that are used for the root hash table
    ///
    pub fn new<P : AsRef<path::Path>>(filename: P, root_bits: u8) -> Result<HashStore, HashStoreError> {
        let file_name = filename.as_ref();
        if !file_name.exists() {
            // create path
            if let Some(dir) = file_name.parent() {
                fs::create_dir_all(dir)?;
            };

            // create new file
            let hdr = header::Header::new(root_bits);
            let mut f = fs::File::create(&file_name)?;

            header::Header::write(&mut f, &hdr)?;

            let root_count = 1 << root_bits;
            f.set_len(mem::size_of::<header::Header>() as u64 + root_count * 8)?;
        }

        // open 2 handles
        let mut rw_file   = fs::OpenOptions::new().read(true).write(true).open(&file_name)?;
        let mmap_file     = fs::OpenOptions::new().read(true).write(true).open(&file_name)?;
        let append_file   = fs::OpenOptions::new().append(true).open(&file_name)?;

        // verify header
        let hdr = header::Header::read(&mut rw_file)?;
        let root_count = 1 << hdr.root_bits;

        if !hdr.is_correct_fileid() {
            return Err(HashStoreError::InvalidMagicFileId);
        }
        if hdr.root_bits != root_bits {
            return Err(HashStoreError::InvalidRootBits);
        }

        // setup memmap
        let mut mmap = memmap::Mmap::open_with_offset(
             &mmap_file,
            memmap::Protection::ReadWrite,
            0,
            mem::size_of::<header::Header>() + 8 * root_count
        )?;



        let u64_ptr = mmap.mut_ptr() as *mut atomic::AtomicU64;
        let u64_slice = unsafe { ::std::slice::from_raw_parts(u64_ptr, root_count + header::header_size_u64()) };

        // split our memmap in the root hash-table and stats
        let root = &u64_slice[header::header_size_u64()..];
        let stats = &u64_slice[header::stats_offset_u64()..header::header_size_u64()];
        let extrema = &u64_slice[header::extrema_offset_u64()..header::stats_offset_u64()];

        Ok(HashStore {
            _mmap: mmap,
            root: root,
            stats: stats,
            extrema: extrema,
            rw_file: rw_file,
            append_file: append_file,
            root_bits: root_bits,
        })
    }

    /// Creates a hashstore, and clears it if it already exists
    ///
    /// `root_bits` is the number of bits of each key that are used for the root hash table
    ///
    pub fn new_empty<P : AsRef<path::Path>>(filename: P, root_bits: u8) -> Result<HashStore, HashStoreError> {
        let p = filename.as_ref();
        if p.exists() {
            fs::remove_file(p).unwrap();
        }
        HashStore::new(p, root_bits)
    }


    /// Checks if `key` exists and returns a persistent pointer if it does
    ///
    /// If `depth` is `SearchDepth::SearchAfter(x)` the search is abandoned after an element with
    /// `time < x` is encountered
    pub fn exists(&mut self, key: &[u8; 32], depth: SearchDepth) -> Result<Option<ValuePtr>, HashStoreError>
    {
        let _timer = Timer::new(&self.stats[HashStoreStats::ReadTime as usize]);

        let idx     = get_root_index(self.root_bits, &key);
        let mut ptr = self.root[idx].load(atomic::Ordering::Relaxed);

        // loop over linked list of value-objects at `ptr`
        loop {

            if ptr == 0 {
                return Ok(None);
            }

            let (prefix, _) = read_value_start(&mut self.rw_file, ptr, Some(0))?;

            if prefix.key == *key {
                return Ok(Some(ptr));
            }

            if !depth.check(prefix.time) {
                return Ok(None);
            }
            ptr = prefix.prev_pos;
        }
    }


    /// Directly reads the value pointed to by `ptr`
    ///
    /// The `size` field of `ptr` does not need to be accurate and is used as estimate.
    /// If it is too small, a second read is performed
    pub fn get_by_ptr(&mut self, ptr: ValuePtr) -> Result<Vec<u8>, HashStoreError>
    {
        let _timer = Timer::new(&self.stats[HashStoreStats::ReadTime as usize]);

        let (prefix, mut content) = read_value_start(&mut self.rw_file, ptr, None)?;
        read_value_finish(&mut self.rw_file, &prefix, &mut content)?;
        Ok(content)
    }

    /// Writes a value without key; this can only be accessed by ValuePtr using get_value
    ///
    pub fn set_value(&mut self, value: &[u8]) -> Result<ValuePtr, HashStoreError>
    {
        let _timer = Timer::new(&self.stats[HashStoreStats::WriteTime as usize]);

        write_value_no_prefix(&mut self.rw_file, value)
    }

    /// Writes a value without key; this can only be accessed by ValuePtr using get_value
    ///
    pub fn get_value(&mut self, ptr: ValuePtr) -> Result<Vec<u8>, HashStoreError>
    {
        let _timer = Timer::new(&self.stats[HashStoreStats::WriteTime as usize]);

        read_value_no_prefix(&mut self.rw_file, ptr)
    }


    /// Checks if `key` exists and returns the value if it does
    ///
    /// If `depth` is `SearchDepth::SearchAfter(x)` the search is abandoned after an element with
    /// `time < x` is encountered.
    ///
    /// If found it returns Some((ValuePtr, Vec<u8>)) where ValuePtr is a persistent pointer to where
    /// the value was found
    pub fn get(&mut self, key: &[u8; 32], depth: SearchDepth) -> Result<Option<(ValuePtr, Vec<u8>)>, HashStoreError>
    {
        let _timer = Timer::new(&self.stats[HashStoreStats::ReadTime as usize]);

        let idx = get_root_index(self.root_bits, &key);

        let mut ptr = self.root[idx].load(atomic::Ordering::Relaxed);

        // loop over linked list of value-objects at dataptr
        loop {

            if ptr == 0 {
                return Ok(None);
            }

            let (prefix, mut value) = read_value_start(&mut self.rw_file, ptr, None)?;

            if prefix.key == *key {
                read_value_finish(&mut self.rw_file, &prefix, &mut value)?;
                return Ok(Some((ptr,value)));
            }

            if !depth.check(prefix.time) {
                return Ok(None);
            }
            ptr = prefix.prev_pos;

        }
    }

    /// Stores `value` at `key`
    ///
    /// `time` can be any integer that roughly increases with time (eg a block height),
    /// and is used to query only recent keys
    pub fn set(&mut self, key: &[u8; 32], value: &[u8], time: u32) -> Result<ValuePtr, HashStoreError>
    {
        let _timer = Timer::new(&self.stats[HashStoreStats::WriteTime as usize]);

        let idx = get_root_index(self.root_bits, key);

        // Compare-and-swap loop
        loop {
            let old_ptr = self.root[idx].load(atomic::Ordering::Acquire);

            let prefix = ValuePrefix {
                key: *key,
                prev_pos: old_ptr,
                time: time,
                size: value.len() as u32,
                ..Default::default()
            };

            let new_ptr = write_value(&mut self.append_file, prefix, value)?;

            let swap_ptr = self.root[idx].compare_and_swap
                (old_ptr, new_ptr, atomic::Ordering::Release);

            if swap_ptr == old_ptr {
                self.stats_add(HashStoreStats::Elements, 1);
                return Ok(new_ptr);
            }
            panic!("Hmm; not testing concurrency yet");
        }
    }

    /// Updates part of a value
    ///
    /// The concurrency model only allows updating each byte of a value to a
    /// single deterministic value. The caller must ensure this.
    ///
    /// Specifically, the caller must ensure that if it changes byte N to X, this
    /// byte will never be changed to anything else than X, neither by the caller
    /// not by any other process
    ///
    /// The caller must also ensure that the update is within the bounds of the value
    pub fn update(&mut self, ptr: ValuePtr, value: &[u8], position: usize) -> Result<(), HashStoreError> {
        let _timer = Timer::new(&self.stats[HashStoreStats::WriteTime as usize]);

        update_value(&mut self.rw_file, ptr, value, position + mem::size_of::<header::Header>())?;
        Ok(())
    }

    /// Updates a 0-4 numbered extremum
    ///
    /// The comparison function will be called with the current extremum value.
    /// If it returns true, the extremum will be set to ptr.
    ///
    /// The function may be called multiple times to resolvre concurrent updates in
    /// the compare-and-swap loop
    pub fn update_extremum<F>(&mut self, ptr: ValuePtr, extremum: usize, f: F) -> Result<(), HashStoreError>
        where F: Fn(Vec<u8>) -> bool
    {
        // Compare-and-swap loop
        loop {
            let current_ptr = self.extrema[extremum].load(atomic::Ordering::Acquire);

            if current_ptr != 0 {
                let current_value = self.get_by_ptr(current_ptr)?;
                if !f(current_value) {
                    return Ok(());
                }
            }

            let swap_ptr = self.extrema[extremum].compare_and_swap(current_ptr, ptr, atomic::Ordering::Release);

            if swap_ptr == current_ptr {
                return Ok(());
            }
        }
    }

    pub fn get_extremum(&mut self, extremum: usize) -> Result<Option<[u8;32]>, HashStoreError> {
        let ptr = self.extrema[extremum].load(atomic::Ordering::Relaxed);
        if ptr == 0 {
            return Ok(None);
        }
        let (prefix, _) = read_value_start(&mut self.rw_file, ptr, Some(0))?;
        Ok(Some(prefix.key))

    }

    /// Flushes all pending writes to disk
    pub fn flush(&mut self)  -> Result<(), HashStoreError> {
        self.append_file.flush()?;
        self._mmap.flush()?;
        Ok(())
    }


    pub fn stats(&mut self) -> Result<Vec<u64>, HashStoreError> {
        self.flush()?;
        let mut stats: Vec<u64> = self.stats.iter().map(|x|
            x.load(atomic::Ordering::Relaxed)).collect();
        let metadata: fs::Metadata = self.append_file.metadata()?;
        stats.push(metadata.len());
        Ok(stats)
    }

    fn stats_add(&mut self, field: HashStoreStats, n: u64) {
        self.stats[field as usize].fetch_add(n, atomic::Ordering::Relaxed);
    }

}

// Returns the index into the root hash table for a key
// This uses the first self.root_bits as index
fn get_root_index(root_bits: u8, key: &[u8; 32]) -> usize {
    let bits32 = ((key[0] as usize) << 24) |
        ((key[1] as usize) << 16) |
        ((key[2] as usize) << 8) |
        (key[3] as usize);
    bits32 >> (32 - root_bits)
}


#[cfg(test)]
mod tests {
    extern crate rand;

    use super::*;
    use self::rand::Rng;

    fn random_key<R: Rng>(rng: &mut R) -> [u8; 32] {
        let mut key = [0; 32];
        rng.fill_bytes(&mut key);
        key
    }

    #[test]
    fn test_get_root_index() {
        for _ in 0..100 {
            let x = random_key(&mut rand::thread_rng());

            assert_eq!(get_root_index(2,&x), (x[0] as usize) >> 6 );
            assert_eq!(get_root_index(6,&x), (x[0] as usize) >> 2 );
            assert_eq!(get_root_index(8,&x), x[0] as usize );
            assert_eq!(get_root_index(9,&x), ((x[0] as usize)<<1) | ((x[1]) as usize) >> 7);
        }
    }

    // Pub function tested in /tests
 }

