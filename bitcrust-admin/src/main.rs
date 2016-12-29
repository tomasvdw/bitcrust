///
///
/// Admin tool; currently just used for testing
///
/// This is executed to import & verify blocks from core.
///

extern crate bitcrust_lib;

extern crate byteorder;


use std::io::BufReader;
use std::fs::File;

mod blk_file;


fn main() {


    let mut block = 0;
    for fileno in 0..1 {
        let name = format!("/home/tomas/.bitcoin/blocks/blk{:05}.dat", fileno);
        println!("Processing {}", name);
        let f = File::open(name).unwrap();
        let mut rdr = BufReader::new(f);
        let mut store = bitcrust_lib::init();


        loop {
            let blk = blk_file::read_block(&mut rdr).unwrap();

            if blk.is_none() {
                break;
            }

            bitcrust_lib::add_block(&mut store, &blk.unwrap());

            block += 1;
            if block % 100 == 0 {
                println!("Processed block {}", block);
            }

            if block == 2 {
                break;
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