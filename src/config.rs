//! Stub configutation
//! Meant to wrap toml-config files

use std::fs;
use std::path::PathBuf;



#[derive(Clone)]
pub struct Config {
    pub root: PathBuf
}


impl Config {

    pub fn root() -> PathBuf {
        unimplemented!()
    }
}
impl Config {

    pub fn new(path: &str) -> Config {

        let path = PathBuf::from(path);
        Config { root: path }

    }

    pub fn new_empty<T : Into<String>>(name: T) -> Config {
        let mut path = PathBuf::from("tmp");
        let name: String = name.into()
            .replace("bitcrust-lib/","")
            .replace("/", "-");
        path.push(name);
        let _ =  fs::remove_dir_all(path.clone());
        Config { root: path }
    }


    pub fn new_persist() -> Config {

        let path = PathBuf::from("prs");
        Config { root: path }

    }
}

