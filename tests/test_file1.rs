
extern crate bitcrust_lib;

extern crate byteorder;


use std::io::BufReader;
use std::fs::File;

mod blk_file;

use std::time::{Instant};



#[test]
#[ignore]
fn load_file1() {

    let mut store = bitcrust_lib::init();

    let fileno = 0;
    let name = format!("./data/blk{:05}.dat", fileno);
    println!("Processing {}", name);
    let f = File::open(name).unwrap();
    let mut rdr = BufReader::new(f);

    let mut blocks = 0;
    loop {
        let blk = blk_file::read_block(&mut rdr).unwrap();

        if blk.is_none() {
            break;
        }

        bitcrust_lib::add_block(&mut store, &blk.unwrap());

        blocks += 1;




        if blocks == 2 {
            break;
        }

    }




}





#[test]
#[ignore]
fn load_file_large() {

    let mut store = bitcrust_lib::init();
    let mut blocks = 0;
    let start = Instant::now();
    const BLOCK_COUNT: u64 = 150000;

    for fileno in 0..99 {
        let name = format!("./data/blk{:05}.dat", fileno);
        println!("Processing {}", name);
        let f = File::open(name).unwrap();
        let mut rdr = BufReader::new(f);

        loop {
            let blk = blk_file::read_block(&mut rdr).unwrap();

            if blk.is_none() {
                break;
            }

            bitcrust_lib::add_block(&mut store, &blk.unwrap());


            blocks += 1;

            if blocks % 100 == 0 {


                println!("Processed {} blocks in {} sec at {}/s", blocks, start.elapsed().as_secs(),
                         blocks / (start.elapsed().as_secs()+1));

            }

            if blocks >= BLOCK_COUNT {
                break;
            }
        }

        if blocks >= BLOCK_COUNT {
            break;
        }
    }

    println!("DONE: Processed {} blocks in {} sec at {}/s", blocks, start.elapsed().as_secs(),
             blocks / (start.elapsed().as_secs()+1));


}