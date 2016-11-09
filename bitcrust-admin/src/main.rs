
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

extern crate bitcrust_lib;

extern crate byteorder;


use std::io::BufReader;
use std::fs::File;

mod blk_file;

use std::fs;

fn main() {


    let mut block = 0;
    for fileno in 0..1 {
        let name = format!("/home/tomas/.bitcoin/blocks/blk{:05}.dat", fileno);
        println!("Processing {}", name);
        let f = File::open(name).unwrap();
        let mut rdr = BufReader::new(f);


        loop {
            let blk = blk_file::read_block(&mut rdr).unwrap();

            if blk.is_none() {
                break;
            }

            let mut store = bitcrust_lib::init();
            bitcrust_lib::add_block(&mut store, &blk.unwrap());

            block += 1;
            if block % 100 == 0 {
                println!("Processed block {}", block);
            }

        }


    }

        //println!("{:?}", blk);
       
        //let serialized = serde_json::to_string(&blk.header).unwrap();

        //println!("{}", serialized);

        
        //let hex = encode(&blk).unwrap();
        
        
        //println!("Block: {:?}", hex);


}

#[cfg(test)]
mod tests {
   // use rustc_serialize::hex::ToHex;
    /*
    #[test]
    fn test_read_block() {
        let f = File::open("/home/tomas/.bitcoin/blocks/blk00000.dat").unwrap();
        let mut rdr = BufReader::new(f);
        let blk = super::read_block(&mut rdr).unwrap();
        
        let serialized = super::serde_json::to_string(&blk.header).unwrap();

    }

    #[bench]
    fn bench_read(b: &mut Bencher) {
        b.iter(|| {
            let f = File::open("/home/tomas/.bitcoin/blocks/blk00020.dat").unwrap();
            let mut rdr = BufReader::new(f);
            let _ = super::read_block(&mut rdr).unwrap();
        
        });
    }
    */
}