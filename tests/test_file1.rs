
extern crate bitcrust_lib;

extern crate byteorder;


use std::io::BufReader;
use std::fs::File;

mod blk_file;



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


            if blocks == 300000 {
                break;
            }
        }
    }

}