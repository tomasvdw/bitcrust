

use std::path::PathBuf;


pub struct Config {
    pub root: PathBuf
}


impl Config {

    pub fn root() -> PathBuf {
        unimplemented!();
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    impl Config {
        pub fn new_test() -> Config {
            Config {
                root: PathBuf::from("tmp")
            }
        }
    }
}