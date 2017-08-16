use std::clone::Clone;
use std::env::home_dir;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

use clap::ArgMatches;
use log::LogLevel;
use ring::{digest, rand, hmac};
use toml;

#[derive(Deserialize, Serialize, Debug)]
pub struct ConfigFile {
    key: Vec<u8>
}

pub struct Config {
    pub log_level: LogLevel,
    raw_key: [u8; 32],
    signing_key: hmac::SigningKey
}

impl Config {
    pub fn from_args(matches: &ArgMatches) -> Config {
        let mut path = home_dir().expect("Can't figure out where your $HOME is");
        path.push(".bitcrust.toml");
        let log_level = match matches.occurrences_of("debug") {
            0 => LogLevel::Warn,
            1 => LogLevel::Info,
            2 => LogLevel::Debug,
            3 | _ => LogLevel::Trace,
        };
        let config_file_path: PathBuf = matches.value_of("config").map(|p| PathBuf::from(&p)).unwrap_or(path);
        let config_from_file: ConfigFile = if config_file_path.exists() {
            let mut f = File::open(config_file_path.clone()).unwrap();
            let mut s = String::new();
            f.read_to_string(&mut s);
            toml::from_str(&s).unwrap_or_else(|_| Config::create_default(config_file_path))
        } else {
            Config::create_default(config_file_path)
        };

        let key = hmac::SigningKey::new(&digest::SHA256, &config_from_file.key);

        let mut a: [u8; 32] = [0; 32];
        a.copy_from_slice(&config_from_file.key);

        Config {
            log_level: log_level,
            raw_key: a,
            signing_key: key,
        }


    }

    pub fn create_default(path: PathBuf) -> ConfigFile {
        let rng = rand::SystemRandom::new();
        let mut key = [0; 32];
        rng.fill(&mut key).unwrap();
        let c = ConfigFile {
            key: key.to_vec()
        };
        let s = toml::to_string(&c).unwrap();
        println!("Making a new config file with: {}", s);
        let mut f = File::create(path).unwrap();
        f.write_all(&s.as_bytes());//.to_string());
        c
    }

    pub fn key(&self) -> &hmac::SigningKey {
        &self.signing_key
    }
}

impl Clone for Config {
    fn clone(&self) -> Config {
        Config {
            log_level: self.log_level,
            raw_key: self.raw_key,
            signing_key: hmac::SigningKey::new(&digest::SHA256, &self.raw_key)
        }
    }
}