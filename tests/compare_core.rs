


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
use std::fs;
use std::io::Seek;


mod blk_file;
use std::time::{Instant};

#[derive(Debug,Copy,Clone)]
struct ReadPos {
    file_number: usize,
    file_position: u64
}

fn read_next(pos: ReadPos) -> Option<(ReadPos,Vec<u8>)> {

    // next file available
    if fs::metadata(format!("/home/tomas/.bitcoin/blocks/blk{:05}.dat", pos.file_number + 1)).is_ok() {
        return read_next(ReadPos { file_number: pos.file_number +  1, file_position: 0 });
    }

    let mut name = format!("/home/tomas/.bitcoin/blocks/blk{:05}.dat", pos.file_number);
    if fs::metadata(name.clone()).unwrap().len() <= pos.file_position {
        return None;
    }

    let mut file = File::open(name).unwrap();
    let mut rdr = BufReader::new(file);

    let blk = blk_file::read_block(&mut rdr).unwrap().unwrap();

    Some( (ReadPos {
                file_number: pos.file_number,
                file_position: rdr.seek(std::io::SeekFrom::Current(0)).unwrap()
            }
          , blk))

}



#[test]
#[ignore]
fn compare_core() {

    let mut store = bitcrust_lib::init();
    let mut pos = sync_initial(&mut store);

    store.initial_sync = false;

    println!("No more initial sync; polling files");

    loop {
        match read_next(pos) {
            None => {
                std::thread::sleep_ms(5000);
                continue;
            },
            Some((p, blk)) => {
                pos = p;
                bitcrust_lib::add_block(&mut store, &blk);


            }
        }
    }

}


fn sync_initial(store: &mut bitcrust_lib::Store) -> ReadPos {



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
            let open_result = File::open(name.clone());

            match open_result {
                Ok(f) => {
                    println!("Processing file {}", name);
                    file = f
                },
                Err(_) => {

                    return ReadPos {
                        file_number: fileno,
                        file_position: rdr.seek(std::io::SeekFrom::Current(0)).unwrap() };

                }
            };
            fileno += 1;
            rdr = BufReader::new(file);


        } else {
            bitcrust_lib::add_block(store, &blk.unwrap());

            blocks += 1;
            println!("Processing block {}", blocks);
        }
    }

    unreachable!()

    //bitcoind -blocksonly -printtoconsole -debug=bench
}