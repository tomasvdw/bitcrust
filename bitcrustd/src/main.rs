#[macro_use]
extern crate clap;
#[macro_use]
extern crate slog;
extern crate slog_term;
extern crate bitcrust_net;
extern crate simple_logger;
extern crate multiqueue;
extern crate ring;
extern crate rusqlite;
#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate toml;

use std::thread;
use std::time::Duration;

use clap::ArgMatches;

use bitcrust_net::{BitcoinNetworkConnection, BitcoinNetworkError, Message, AuthenticatedBitcrustMessage};

mod client_message;
mod config;
mod peer_manager;
mod peer;

use config::Config;
use peer_manager::PeerManager;

fn main() {
    let matches = Config::matches().get_matches();

    let config = Config::from_args(&matches);

    match matches.subcommand() {
        ("node", Some(node_matches)) => {
            // simple_logger::init_with_level(config.log_level).expect("Couldn't initialize logger");
            node(node_matches, &config);
        }
        ("balance", Some(balance_matches)) => {
            balance(balance_matches, &config);
        }
        ("stats", Some(stats_matches)) => {
            stats(stats_matches, &config);
        }
        ("", None) => println!("No subcommand was used"), // If no subcommand was usd it'll match the tuple ("", None)
        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachabe!()
    }
}

fn node(_matches: &ArgMatches, config: &Config) {
    let mut client = PeerManager::new(config);
    client.execute();
}

fn balance(matches: &ArgMatches, _config: &Config) {
    // This unwrap is safe because we require it above
    let address = matches.value_of("address").unwrap();
    println!("I'd love to get your balance on '{}' but it's not yet implemented!", address);
}

fn stats(matches: &ArgMatches, config: &Config) {
    let host = matches.value_of("host").unwrap_or("127.0.0.1:8333").to_string();
    match matches.subcommand() {
        ("peers", Some(peer_matches)) => {
            connected_peers(peer_matches, config, host);
        }
        ("", None) => println!("No subcommand was used"), // If no subcommand was usd it'll match the tuple ("", None)
        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachabe!()
    }
}

fn connected_peers(_matches: &ArgMatches, config: &Config, host: String) {
    let connection = BitcoinNetworkConnection::new(host.clone())
        .expect(&format!("Couldn't connect to a node running on {}", host));
    let _ = connection.try_send(peer::Peer::version());
    loop {
        if let Some(msg) = connection.try_recv() {
            match msg {
                Ok(msg) => match msg {
                    Message::Version(_version) => {
                        let _ = connection.try_send(Message::Verack);
                        break;
                    }
                    _ => {}
                },
                Err(BitcoinNetworkError::ReadTimeout) => thread::sleep(Duration::from_millis(200)),
                Err(BitcoinNetworkError::Closed) => {
                    println!("Remote server closed the connection");
                    return
                }
                Err(BitcoinNetworkError::BadBytes) => return
            }
        }
    }
    let auth = AuthenticatedBitcrustMessage::create(config.key());
    match connection.try_send(Message::BitcrustPeerCountRequest(auth)) {
        Ok(_) => {},
        Err(e) => warn!(config.logger, "Error sending request: {:?}", e),
    }
    loop {
        if let Some(msg) = connection.try_recv() {
            match msg {
                Ok(msg) => match msg {
                    Message::BitcrustPeerCount(count) => {
                        println!("There are {} peers currently connected to {}", count, host);
                        break;
                    }
                    _ => {}
                },
                Err(BitcoinNetworkError::ReadTimeout) => thread::sleep(Duration::from_millis(200)),
                Err(BitcoinNetworkError::Closed) => {
                    println!("Remote server closed the connection");
                    return
                }
                Err(BitcoinNetworkError::BadBytes) => return
            }
        }
    }
}