use std::clone::Clone;
use std::env::home_dir;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

use clap::{App, Arg, ArgMatches, SubCommand};
use slog::Level;
use ring::{digest, rand, hmac};
use ring::rand::SecureRandom;
use slog;
use slog_term;
use slog::DrainExt;
use toml;

#[cfg(test)]
mod tests {
    extern crate tempfile;
    use self::tempfile::NamedTempFile;
    use std::fs::File;

    use super::*;

    fn temp_file() -> (File, PathBuf) {
        let f = NamedTempFile::new().expect("failed to create temporary file");
        let path = f.path();
        (f.try_clone().unwrap(), path.to_path_buf())
    }

    #[test]
    fn it_generates_a_new_key() {
        let (_f, path) = temp_file();
        let config_file = Config::create_default(path.clone());
        let args = Config::matches().get_matches_from(vec!["bitcrustd", &format!("--config={}", path.to_string_lossy())[..], "stats", "peers"]);
        let config = Config::from_args(&args);
        assert_eq!(config_file.key, config.raw_key.to_vec());
    }

    #[test]
    fn it_can_be_cloned() {
        let (_f, path) = temp_file();
        let _= Config::create_default(path.clone());
        let args = Config::matches().get_matches_from(vec!["bitcrustd", &format!("--config={}", path.to_string_lossy())[..], "stats", "peers"]);
        let config = Config::from_args(&args);
        assert_eq!(config.clone().raw_key, config.raw_key);
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ConfigFile {
    key: Vec<u8>
}

pub struct Config {
    pub logger: slog::Logger,
    pub log_level: Level,
    raw_key: [u8; 32],
    signing_key: hmac::SigningKey
}

impl<'a, 'b> Config {
    pub fn from_args(matches: &ArgMatches) -> Config {
        let log_level = match matches.occurrences_of("debug") {
            0 => Level::Warning,
            1 => Level::Info,
            2 => Level::Debug,
            3 | _ => Level::Trace,
        };
        let config_file_path: PathBuf = matches.value_of("config").map(|p| PathBuf::from(&p)).unwrap_or_else(|| {
            let mut path = home_dir().expect("Can't figure out where your $HOME is");
            path.push(".bitcrust.toml");
            path
        });
        let config_from_file: ConfigFile = if config_file_path.exists() {
            let mut f = File::open(config_file_path.clone()).unwrap();
            let mut s = String::new();
            let _ = f.read_to_string(&mut s);
            toml::from_str(&s).unwrap_or_else(|_| Config::create_default(config_file_path))
        } else {
            Config::create_default(config_file_path)
        };

        let key = hmac::SigningKey::new(&digest::SHA256, &config_from_file.key);

        let mut a: [u8; 32] = [0; 32];
        a.copy_from_slice(&config_from_file.key);

        Config {
            logger:  slog::Logger::root(slog_term::streamer().compact().build().fuse(), o!()),
            log_level: log_level,
            raw_key: a,
            signing_key: key,
        }


    }

    pub fn matches() -> App<'a, 'b> {
        App::new("bitcrustd")
            .version(crate_version!())
            .author("Chris M., Tomas W.")
            .arg(Arg::with_name("config")
                .short("c")
                .long("config")
                .takes_value(true)
                .help("Location of the Bitcrust Config File, default: $HOME/.bitcrust.toml"))
            .arg(Arg::with_name("debug")
                .short("d")
                .long("debug")
                .multiple(true)
                .help("Turn debugging information on"))
            .subcommand(SubCommand::with_name("node").about("Bitcrust peer node"))
            .subcommand(SubCommand::with_name("stats")
                .about("Get stats from a running Bitcrust node")
                .arg(Arg::with_name("host")
                    .short("h")
                    .long("host")
                    .takes_value(true))
                .subcommand(SubCommand::with_name("peers"))
            )
            .subcommand(SubCommand::with_name("balance")
                .about("Get balance for address")
                .arg(Arg::with_name("address")
                    .short("a")
                    .long("address")
                    .help("Address to get balance for")
                    .takes_value(true)
                    .required(true))
            )
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
        let _ = f.write_all(&s.as_bytes());//.to_string());
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
            signing_key: hmac::SigningKey::new(&digest::SHA256, &self.raw_key),
            logger: self.logger.clone(),
        }
    }
}