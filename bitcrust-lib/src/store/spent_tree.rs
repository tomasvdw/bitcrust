

/// The spent tree stores the location of transactions in the block-tree
///
/// It is tracks the tree of blocks and is used to verify whether a block can be inserted at a
/// certain location in the tree
///
///



use config;

use store::fileptr::FilePtr;
use store::flatfileset::FlatFileSet;

const MB:                 u32 = 1024 * 1024;
const FILE_SIZE:          u32 = 1024 * MB as u32;
const MAX_CONTENT_SIZE:   u32 = FILE_SIZE - 10 * MB as u32 ;

const SUBPATH: &'static str   = "spent_tree";
const PREFIX:  &'static str   = "st-";



struct Record {
    ptr:   FilePtr,
    skips: [u16;4]
}

impl Record {

    fn previous(&self) -> Option<Record> {
        None
    }

    fn new(ptr: FilePtr) -> Self {
        Record {
            ptr: ptr,
            skips: [0,0,0,0]
        }
    }
}


pub struct SpentTree {

    fileset:    FlatFileSet,

}

impl SpentTree {
    pub fn new(cfg: &config::Config) -> SpentTree {

        let dir = &cfg.root.clone().join(SUBPATH);

        SpentTree {
            fileset: FlatFileSet::new(
                dir, PREFIX, FILE_SIZE, MAX_CONTENT_SIZE)
        }
    }


    pub fn store_block(&mut self, file_ptrs: Vec<FilePtr>) {

        let target: &[Record] = self.fileset.alloc_slice(file_ptrs.len());

        for (idx, ptr) in file_ptrs.iter().enumerate() {


            // we scan back through target
            // we need to find

        }

    }


    pub fn read(&mut self, pos: FilePtr) -> &[u8] {
        self.fileset.read(pos)
    }

    pub fn write(&mut self, buffer: &[u8]) -> FilePtr {
        self.fileset.write(buffer)
    }

    pub fn read_fixed<T>(&mut self, pos: FilePtr) -> &'static T {
        self.fileset.read_fixed(pos)
    }
}

