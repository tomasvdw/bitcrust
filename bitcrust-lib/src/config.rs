//! Stub configutation
//! Meant to wrap toml-config files

use std::fs;
use std::path::PathBuf;




pub struct Config {
    pub root: PathBuf
}


impl Config {

    pub fn root() -> PathBuf {
        unimplemented!();
    }
}
impl Config {


    pub fn new_test() -> Config {

        let path = PathBuf::from("tmp");
        let _ =  fs::remove_dir_all(path.clone());
        Config { root: path }

    }
}

