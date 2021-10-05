//!
//! Module to access bitcoin-core style blk files
//! that store the integral blockchain


extern crate byteorder;
extern crate store;

use self::byteorder::{ReadBytesExt, LittleEndian};
use std::io::BufReader;
use std::fs::File;
use std::path::Path;

use std::io;


/// Magic number stored at the start of each block
const MAGIC: u32 = 0xD9B4BEF9;


pub struct BlkFileIterator {
    file_nr: usize,
    reader: BufReader<File>

}

impl Iterator for BlkFileIterator {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Vec<u8>> {

        let blk = read_block(&mut self.reader).unwrap();
        if blk.is_none() {
            self.file_nr += 1;
            let sname = format!("../core-blocks/blk{:05}.dat", self.file_nr);
            let name = Path::new(&sname);
            if !name.exists() {
                return None;
            }
            let f = File::open(name).unwrap();
            self.reader = BufReader::new(f);
            read_block(&mut self.reader).unwrap()

        }
        else {
            blk
        }
    }
}


pub fn read_blocks() -> BlkFileIterator {
    let sname = format!("../core-blocks/blk{:05}.dat", 0);
    let name = Path::new(& sname);
    if !name.exists() {
        panic!("No blk-files found at ./core-blocks. \
                Use 'ln - ~/.bitcoin/blocks ./core-blocks' to link the directory.");
    }
    let f = File::open(name).unwrap();
    let rdr = BufReader::new(f);

    BlkFileIterator { file_nr: 0, reader: rdr }
}

/// Reads a block from a blk_file as used by
/// bitcoin-core and various other implementations
fn read_block(rdr: &mut io::Read) -> Result<Option<Vec<u8>>, io::Error> {

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


    let length     = try!(rdr.read_u32::<LittleEndian>());
    let mut buffer = vec![0; length as usize];


    rdr.read_exact(&mut buffer)?;


    Ok(Some(buffer))


}




