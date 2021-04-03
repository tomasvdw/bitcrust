//!
//! Module to access bitcoin-core style blk files
//! that store the integral blockchain


extern crate byteorder;

use byteorder::{ReadBytesExt, LittleEndian};


use std::io;

/// Magic number stored at the start of each block
const MAGIC: u32 = 0xD9B4BEF9;


/// Reads a block from a blk_file as used by
/// bitcoin-core and various other implementations
pub fn read_block(rdr: &mut dyn io::Read) -> Result<Option<Vec<u8>>, io::Error> {

    loop {
        let magicnr = rdr.read_u32::<LittleEndian>();
        match magicnr {
            Err(_)     => return Ok(None), // assume EOF
            Ok(m) => match m {

                // TODO investigate; // Can't really find it in the cpp.
                // this happens on bitcrust-1 at block  451327
                // file blk000760, file pos 54391594
                // first 8 zero-bytes before magicnr
                // for now we skip them; not too important as we
                // might not want to support this type of import anyway
                0     => continue,

                MAGIC => break,
                _     =>return Err(io::Error::new(io::ErrorKind::InvalidData, "Incorrect magic number"))
            }
        }

    }


    let length     = rdr.read_u32::<LittleEndian>()?;
    let mut buffer = vec![0; length as usize];


    rdr.read_exact(&mut buffer)?;


    Ok(Some(buffer))


}




