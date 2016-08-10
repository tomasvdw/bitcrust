


/*#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate serde;
extern crate serde_json;

#[derive(Serialize, Deserialize, Debug)]
struct Point {
    x: i32,
    y: i32,
}

fn main() {
    let point = Point { x: 1, y: 2 };
    let serialized = serde_json::to_string(&point).unwrap();

    println!("{}", serialized);

    let deserialized: Point = serde_json::from_str(&serialized).unwrap();

    println!("{:?}", deserialized);
}

*/
extern crate serde_json;

extern crate bitcrust_lib;

extern crate byteorder;


use std::io::{Read, BufReader};
use std::fs::File;
use byteorder::{ReadBytesExt, LittleEndian};
use std::result::Result;
use bitcrust_lib::{decode,encode};
use bitcrust_lib::block;



fn read_block(rdr: &mut Read) -> Result<block::Block, std::io::Error> {
    let _          = try!(rdr.read_u32::<LittleEndian>());
    let length     = try!(rdr.read_u32::<LittleEndian>());
    let mut buffer = vec![0; length as usize];
    
    try!(rdr.read(&mut buffer));
    Ok(decode(&buffer).unwrap())
    
}



fn main() {
    
    let f = File::open("/home/tomas/.bitcoin/blocks/blk00000.dat").unwrap();
    let mut rdr = BufReader::new(f);

    for _ in 0..100 {
        let blk = read_block(&mut rdr).unwrap();
        
        println!("{:?}", blk);
       
        let serialized = serde_json::to_string(&blk.header).unwrap();

        //println!("{}", serialized);

        
        let hex = encode(&blk).unwrap();
        
        
        //println!("Block: {:?}", hex);
    }

}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::{Read, BufReader, Error, Cursor};
   // use rustc_serialize::hex::ToHex;
    
    #[test]
    fn test_read_block() {
        let f = File::open("/home/tomas/.bitcoin/blocks/blk00000.dat").unwrap();
        let mut rdr = BufReader::new(f);
        let blk = super::read_block(&mut rdr).unwrap();
        
        let serialized = super::serde_json::to_string(&blk.header).unwrap();

    }
}