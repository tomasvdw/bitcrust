

/// Store for raw transactions and blockheaders
///
/// This structure does little more then wrap the underlying flatfileset

use config;

use store::fileptr::FilePtr;
use store::flatfileset::FlatFileSet;

const MB:                 u32 = 1024 * 1024;
const FILE_SIZE:          u32 = 1024 * MB as u32;
const MAX_CONTENT_SIZE:   u32 = FILE_SIZE - 10 * MB as u32 ;

const SUBPATH: &'static str   = "block_content";
const PREFIX:  &'static str   = "bc-";

pub struct BlockContent {

    fileset:    FlatFileSet,

}

impl BlockContent {
    pub fn new(cfg: &config::Config) -> BlockContent {

        let dir = &cfg.root.clone().join(SUBPATH);

        BlockContent {
            fileset: FlatFileSet::new(
                dir, PREFIX, FILE_SIZE, MAX_CONTENT_SIZE)
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

