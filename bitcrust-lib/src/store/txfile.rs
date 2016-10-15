

use std::fs;
use std::io;
use std::path::{Path,PathBuf};

pub struct TxFile {
    path:       PathBuf,
    first_file: i16,
    last_file:  i16,
    handles:    Vec<Option<fs::File>>
}

impl TxFile {

    pub fn new(path: &Path) -> TxFile {

        let mut first_file = 0_i16;
        let mut last_file = 0_i16;

        let dir = path
            .read_dir()
            .unwrap()
            .map(|direntry| direntry.unwrap().path() )
            ;

        TxFile {
            path: path.to_path_buf(),
            first_file: first_file,
            last_file: last_file,
            handles: Vec::new()

        }

    }


}


#[cfg(test)]
mod tests {

    #[test]
    fn test_memmap() {

    }
}