//! Stub configutation
//! Meant to wrap toml-config files

use std::fs;
use std::env;
use std::path::PathBuf;


// Overrides the store directory to use
pub const ENV_BITCRUST_STORE: &'static str = "BITCRUST_STORE";

// Set to "1" will prevent the data-folder to be cleared
pub const ENV_BITCRUST_NOCLEAR: &'static str = "BITCRUST_NOCLEAR";



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

    /// Creates and empties a store
    /// Uses the given name except when overriden by env-var BITCRUST_STORE
    /// The store is cleared unless BITCRUST_NOCLEAR=1
    /// This is used from tests
    pub fn new_empty<T : Into<String>>(name: T) -> Config {

        let path = env::var(ENV_BITCRUST_STORE)

            .map(|s| PathBuf::from(s))
            .unwrap_or_else(|_| {

                let mut path = PathBuf::from("tmp");
                let name: String = name.into()
                    .replace("bitcrust-lib/","")
                    .replace("/", "-");
                path.push(name);
                path
            }
        );
        println!("Using store {:?}", path);

        if env::var(ENV_BITCRUST_NOCLEAR).unwrap_or("0".to_string()) !=  "1" {
            let _ =  fs::remove_dir_all(path.clone());
        }
        Config { root: path }
    }


    pub fn new_persist() -> Config {

        let path = PathBuf::from("prs");
        Config { root: path }

    }
}

