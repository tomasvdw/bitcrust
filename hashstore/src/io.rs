///
/// Input/Output helpers to read values and their prefixes

use std::{io,mem};
use bincode;


use HashStoreError;
use values::*;


// write a value and its prefix and return the ValuePtr to the new object
pub fn write_value<W: io::Write + io::Seek>(wr: &mut W, prefix: ValuePrefix, content: &[u8])
    -> Result<ValuePtr, HashStoreError>
{

    let mut buffer: Vec<u8> = bincode::serialize(&prefix, bincode::Infinite)?;
    debug_assert!(buffer.len() == mem::size_of::<ValuePrefix>());
    buffer.extend_from_slice(content);


    wr.write_all(&buffer)?;
    let new_pos = wr.seek(io::SeekFrom::Current(0))?
        - content.len() as u64;

    Ok(ptr_new(new_pos, content.len()))
}

// write a value without prefix
pub fn write_value_no_prefix<W: io::Write + io::Seek>(wr: &mut W, content: &[u8])
                                            -> Result<ValuePtr, HashStoreError>
{
    wr.write_all(&content)?;
    let new_pos = wr.seek(io::SeekFrom::Current(0))?
        - content.len() as u64;

    Ok(ptr_new(new_pos, content.len()))
}



// Writes part of a value
pub fn update_value<W: io::Write + io::Seek>(wr: &mut W, ptr: ValuePtr, content: &[u8], position: usize)
    -> Result<(), HashStoreError>
{
    wr.seek(io::SeekFrom::Start(ptr_file_pos(ptr) + position as u64))?;
    wr.write_all(content)?;

    Ok(())
}


// Read the prefix from the specified location, and (part of the) value
// The value may not be read in full if the size estimate is incorrect
// in which case read_value_full must be called afterwards if the value is needed
pub fn read_value_start<R: io::Read + io::Seek>(rd: &mut R, ptr: ValuePtr, size_needed: Option<usize>)
    -> Result<(ValuePrefix, Vec<u8>), HashStoreError>
{
    // use either passed `size_needed` or estimate from ptr
    let prefix_size = mem::size_of::<ValuePrefix>();
    let read_size = prefix_size + size_needed.unwrap_or(ptr_size_est(ptr));

    rd.seek(io::SeekFrom::Start(ptr_file_pos(ptr) - prefix_size as u64))?;
    let mut buffer = vec![0u8; read_size];

    if let Err(e) = rd.read_exact(&mut buffer) {
        // EOF can happen as the size from datapos can be bigger
        // than the actual size; this is solved in read_value_end
        if e.kind() != io::ErrorKind::UnexpectedEof {

            return Err(HashStoreError::IoError(e));
        }
    }
    // split buffer in prefix and content
    let prefix  = bincode::deserialize(&buffer[0..mem::size_of::<ValuePrefix>()])?;
    let content = &buffer[mem::size_of::<ValuePrefix>()..];
    Ok((prefix, content.to_vec()))
}

// Reads any remaining bytes of the value
// Must be called after read_value_start if the full value is needed
pub fn read_value_finish<R: io::Read>(rd: &mut R, prefix: &ValuePrefix, content: &mut Vec<u8>)
    -> Result<(), HashStoreError>
{

    if prefix.size as usize > content.len() {
        let bytes_todo = prefix.size as usize - content.len();

        let mut buffer = vec![0; bytes_todo];
        rd.read_exact(&mut buffer)?;
        content.append(&mut buffer);

    }
    Ok(())
}


// reads a value without prefix
pub fn read_value_no_prefix<R: io::Read + io::Seek>(rd: &mut R, ptr: ValuePtr)
                                                      -> Result<Vec<u8>, HashStoreError>
{
    let read_size = ptr_size_est(ptr);

    rd.seek(io::SeekFrom::Start(ptr_file_pos(ptr)))?;
    let mut buffer = vec![0u8; read_size];

    if let Err(e) = rd.read_exact(&mut buffer) {
        // EOF can happen as the size from datapos can be bigger
        // than the actual size; this is solved in read_value_end
        if e.kind() != io::ErrorKind::UnexpectedEof {

            return Err(HashStoreError::IoError(e));
        }
    }

    Ok(buffer)
}


#[cfg(test)]
mod tests {
    extern crate rand;
    use super::*;
    use std::fs;
    use values::ValuePrefix;

    fn random_value<R : rand::Rng>(rng: &mut R, size: u32) -> Vec<u8> {

        let mut value = vec![0; size as usize];
        rng.fill_bytes(&mut value);
        value
    }

    fn random_key<R: rand::Rng>(rng: &mut R) -> [u8; 32] {
        let mut key = [0; 32];
        rng.fill_bytes(&mut key);
        key
    }

    fn do_write<W: ::std::io::Write + ::std::io::Seek>(wr: &mut W, size: usize) -> (ValuePtr, Vec<u8>) {
        let mut rng = rand::weak_rng();
        let v1 = random_value(&mut rng, size as u32);
        let v1_prefix = ValuePrefix {
            key: random_key(&mut rng),
            size: v1.len() as u32,
            ..Default::default()
        };
        let ptr = write_value(wr, v1_prefix, &v1).unwrap();
        (ptr, v1)
    }


    #[test]
    fn test_io() {
        fs::create_dir_all("testdb").unwrap();
        let mut fr = fs::OpenOptions::new().write(true).read(true).create(true).open("./testdb/io").unwrap();
        let mut fw = fs::OpenOptions::new().append(true).open("./testdb/io").unwrap();

        // small power of two should work in one go
        let (ptr, v) = do_write(&mut fw, 256 );
        let (_, res) = read_value_start(&mut fr, ptr, None).unwrap();
        assert_eq!(&res, &v);

        // a bit larger needs truncating
        let (ptr, v) = do_write(&mut fw, 500);
        let (prefix, mut res) = read_value_start(&mut fr, ptr, None).unwrap();
        assert_ne!(&res, &v);
        assert_eq!(&res[0..v.len()], &v[..]);
        // finishing is a No-Op
        read_value_finish(&mut fr, &prefix, &mut res).unwrap();
        assert_eq!(&res[0..v.len()], &v[..]);

        // larger than passed size_needing needs another read
        // a bit larger needs truncating
        let (ptr, v) = do_write(&mut fw, 5_000_000);
        let (prefix, mut res) = read_value_start(&mut fr, ptr, Some(1000)).unwrap();
        assert_ne!(&res, &v);
        read_value_finish(&mut fr, &prefix, &mut res).unwrap();
        assert_eq!(&res, &v);
    }
}