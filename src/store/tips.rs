


//! These rules manage the set of tips
//!
//! * When genesis is added, it is stored as tip 0
//! * When a block is added to a block with a tip, the tip is updated
//! * When a block is added to a block without a tip, a tip is added
//! * Tips are eventually pruned TBD

//! Tips do not use flatfileset. Instead, each tip is a file on disk
//! as these cover the requirements better and nicely allow atomic replacement

//! A tip is referenced by the block hash and it contains
//! * The block hash
//! * The difficulty target
//! * Softfork info
//!
//! This is a draft implementation; the format is TBD



use std::path::{Path,PathBuf};
use std::fs;
use std::fs::File;
use std::io;
use std::io::prelude::*;

use util::*;
use hash::*;
use config;


pub struct Tips {

    path: PathBuf
}

pub fn add_tip(tips: &Tips, block_hash: Hash32Buf, difficulty: u64, height: u64) {


    let tip = Tip {
        block_hash: block_hash,
        difficulty: difficulty,
        height: height
    };
    let path = tips.path.join(format!("{}", tip.filename()));

    println!("ADD TIP: {:?}", path);

    let path = tips.path.join(format!("{}", tip.filename()));

    let mut file = fs::File::create(path)
        .expect("Cannot create files in store");

    tip.write(&mut file);
}


impl Tips {

    pub fn new(cfg: &config::Config) -> Tips {
        let path = &cfg.root.clone().join("tips");

        if !path.exists() {
            fs::create_dir_all(path)
                .expect(&format!("Could not create {:?}", path));
        }

        Tips {
            path: PathBuf::from(path)
        }
    }


    pub fn get_tips() -> Vec<Tip> {
        vec![]
    }

    pub fn get_most_work_tip() -> Tip {
        unimplemented!()
    }

    pub fn remove_tip(tip: Tip) {
        unimplemented!()
    }

    pub fn get_all() {
        
    }
}

pub struct Tip {

    block_hash: Hash32Buf,

    difficulty: u64,
    height: u64

    
    // softfork rules

}

impl Tip {


    pub fn new(block_hash: Hash32Buf, difficulty: u64, height: u64) -> Tip {

        Tip {
            block_hash: block_hash,
            difficulty: difficulty,
            height: height
        }
    }

    fn write<W: io::Write>(&self, writer: &mut W) {

        write!(writer,"{},{}", self.difficulty, self.height).unwrap();
    }

    fn filename(&self) -> String {

        self.block_hash
            .as_ref().0
            .iter()
            .rev()
            .map(|n| format!("{:02x}", n))
            .collect::<Vec<_>>()
            .concat()

    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use util::*;

    #[test]
    fn test_create_tip() {

        const HASH1: &'static str = "212300e77d897f2f059366ed03c8bf2757bc2b1dd30df15d34f6f1ee521e58e8";
        const HASH2: &'static str = "4feec9316077e49b59bc23173303e13be9e9f5f9fa0660a58112a04a65a84ef1";

        let tips = Tips::new(&test_cfg!());

        add_tip(&tips, Hash32Buf::from_slice(&from_hex_rev(HASH1)), 1, 2);
        add_tip(&tips, Hash32Buf::from_slice(&from_hex_rev(HASH2)), 3, 4);

    }
}