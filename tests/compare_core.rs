


/// Tool to compare the block processing time of core with bitcrust
///
/// Initialization consists of 2 phases
/// - Run core until progress=1 (core=up-to-date)
/// - Sync bitcrust from blk files to add_block
///
/// Then we wait for incoming blocks in core, add them to bitcrust and compare the result
/// from log


extern crate bitcrust_lib;
extern crate byteorder;
use std::io::BufReader;
use std::fs::File;


mod blk_file;
use std::time::{Instant};





#[test]
#[ignore]
fn compare_core() {

    let mut synced = false;

    let mut store = bitcrust_lib::init();

    // Step one; load existing data from blk files
    let mut fileno = 0;
    let mut name = format!("/home/tomas/.bitcoin/blocks/blk{:05}.dat", fileno);
    println!("Processing {}", name);
    let mut file = File::open(name).unwrap();
    let mut rdr = BufReader::new(file);

    let mut blocks = 0;
    loop {
        let blk = blk_file::read_block(&mut rdr).unwrap();

        if blk.is_none() {
            name = format!("./data/blk{:05}.dat", fileno+1);


            println!("Processing file {}", name);
            let open_result = File::open(name);
            match open_result {
                Ok(f) => file = f,
                Err(_) => {
                    synced = true;
                    store.initial_sync = false;
                    ::std::thread::sleep_ms(5000);
                    println!("No more initial sync; polling files");
                    continue;
                }
            };
            fileno += 1;
            rdr = BufReader::new(file);


        } else {
            bitcrust_lib::add_block(&mut store, &blk.unwrap());

            blocks += 1;
            println!("Processing block {}", blocks);
        }
    }

    //bitcoind -blocksonly -printtoconsole -debug=bench
}