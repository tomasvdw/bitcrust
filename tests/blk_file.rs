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
pub fn read_block(rdr: &mut io::Read) -> Result<Option<Vec<u8>>, io::Error> {

    let magicnr    = rdr.read_u32::<LittleEndian>();
    if magicnr.is_err() {
        return Ok(None)
    }



    let length     = try!(rdr.read_u32::<LittleEndian>());
    let mut buffer = vec![0; length as usize];

    if magicnr.unwrap() != MAGIC {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Incorrect magic number"));
    }

    try!(rdr.read_exact(&mut buffer));


    Ok(Some(buffer))



    //bitcrust_lib::decode(&buffer)
    //    .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Incorrect length"))

}




