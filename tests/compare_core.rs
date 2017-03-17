


/// Tool to compare the block processing time of core with bitcrust
///
/// Will run in two phases
/// - Sync bitcrust from blk files to add_block with initial_sync=true until no more blocks are in
/// - Then poll the blk files every 5 sec to see if a block came in and add it with initial_sync=false
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

/// can be used for fast startup in combination with the no_clear_data feature
const SKIP_FILES: usize = 0;

fn blk_file_name(file_number: usize) -> String {
    format!("/home/tomas/.bitcoin/blocks/blk{:05}.dat", file_number)
}

/// A reference to a position in a blk file
#[derive(Debug,Copy,Clone)]
struct ReadPos {
    pub file_number: usize,
    pub file_position: u64
}

/// Reads a block from the blk file position
/// This is used on live reading, as we need to reopen the file to come out of EOF position.
fn read_next(pos: ReadPos) -> Option<(ReadPos,Vec<u8>)> {

    // next file available?
    if fs::metadata(blk_file_name(pos.file_number + 1)).is_ok() {
        return read_next(ReadPos { file_number: pos.file_number +  1, file_position: 0 });
    }

    let name = blk_file_name(pos.file_number);

    let file = File::open(name).unwrap();
    let mut rdr = BufReader::new(file);

    let _ = rdr.seek(std::io::SeekFrom::Start(pos.file_position)).unwrap();


    let blk = blk_file::read_block(&mut rdr).unwrap();
    match blk {
        None => {
            None
        },
        Some(blk) => {
            Some( (ReadPos {
                file_number: pos.file_number,
                file_position: rdr.seek(std::io::SeekFrom::Current(0)).unwrap()

                }, blk))

        }

    }

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
                std::thread::sleep(std::time::Duration::new(5,0));
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
    let mut read_pos = ReadPos {
        file_number: SKIP_FILES,
        file_position: 0
    };

    let mut name = blk_file_name(read_pos.file_number);

    let mut file = File::open(name).unwrap();
    let mut rdr = BufReader::new(file);

    let mut blocks = 0;
    loop {
        let blk = blk_file::read_block(&mut rdr).unwrap();

        if blk.is_none() {

            // next file?
            name = blk_file_name(read_pos.file_number + 1);
            let open_result = File::open(name.clone());

            match open_result {
                Ok(f) => {
                    println!("Processing file {}", name);
                    file = f
                },
                Err(_) => {

                    // no? then we're done initial syncing
                    return read_pos;

                }
            };
            // next file
            read_pos.file_number = read_pos.file_number + 1;
            read_pos.file_position  = 0;
            rdr = BufReader::new(file);


        } else {
            bitcrust_lib::add_block(store, &blk.unwrap());
            read_pos.file_position = rdr.seek(std::io::SeekFrom::Current(0)).unwrap();

            blocks += 1;
            println!("Processing block {}", blocks);
        }
    }


}