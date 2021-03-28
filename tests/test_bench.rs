
extern crate bitcrust_lib;

extern crate byteorder;


use std::io::BufReader;
use std::fs::File;

mod blk_file;

use std::time::{Instant};
extern crate rayon;



#[test]
#[ignore]
fn load_bench_init() {

    let mut store = bitcrust_lib::init();

    for fileno in 0..750 {
        let name = format!("./core-blocks/blk{:05}.dat", fileno);
        println!("Processing {}", name);
        let f = File::open(name).unwrap();
        let mut rdr = BufReader::new(f);

        let mut _blocks = 0;
        loop {
            let blk = blk_file::read_block(&mut rdr).unwrap();

            if blk.is_none() {
                break;
            }

            bitcrust_lib::add_block(&mut store, &blk.unwrap());

            _blocks += 1;
       }
    }

}

#[test]
#[ignore]
fn load_bench_next() {
    let mut store = bitcrust_lib::init();
    store.initial_sync = false;

    let fileno = 750;

    let name = format!("./core-blocks/blk{:05}.dat", fileno);
    println!("Processing {}", name);
    let f = File::open(name).unwrap();

    let mut rdr = BufReader::new(f);

    let mut blocks = 0;
    let start = Instant::now();
    loop {
        let blk = blk_file::read_block(&mut rdr).unwrap();

        if blk.is_none() {
            break;
        }

        bitcrust_lib::add_block(&mut store, &blk.unwrap());

        blocks += 1;
    }

    let elapsed = Instant::now() - start;
    let elapsed = elapsed.as_secs() as u64 * 1000 + elapsed.subsec_nanos() as u64 / 1000_000;
    println!("Processes {} blocks in {} ms ({} ms/block)", blocks, elapsed, elapsed / blocks )

}

