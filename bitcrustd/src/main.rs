#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
extern crate bitcrust_net;
extern crate simple_logger;
extern crate multiqueue;
extern crate rusqlite;

use clap::{App, Arg, ArgMatches, SubCommand};
use log::LogLevel;

mod peer_manager;
mod peer;

use peer_manager::PeerManager;

fn main() {
    let matches = App::new("bitcrustd")
        .version(crate_version!())
        .author("Chris M., Tomas W.")
        .arg(Arg::with_name("debug")
            .short("d")
            .multiple(true)
            .help("Turn debugging information on"))
        .subcommand(SubCommand::with_name("node").about("Bitcrust peer node"))
        .subcommand(SubCommand::with_name("balance")
            .about("Get balance for address")
            .arg(Arg::with_name("address")
                .short("a")
                .help("Address to get balance for")
                .takes_value(true)
                .required(true)))
        .get_matches();

    let log_level = match matches.occurrences_of("debug") {
        0 => LogLevel::Warn,
        1 => LogLevel::Info,
        2 => LogLevel::Debug,
        3 | _ => LogLevel::Trace,
    };
    simple_logger::init_with_level(log_level).expect("Couldn't initialize logger");

    match matches.subcommand() {
        ("node", Some(node_matches)) => {
            node(node_matches);
        }
        ("balance", Some(balance_matches)) => {
            balance(balance_matches);
        }
        ("", None) => println!("No subcommand was used"), // If no subcommand was usd it'll match the tuple ("", None)
        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachabe!()
    }
}

fn node(_matches: &ArgMatches) {
    let mut client = PeerManager::new();
    client.execute();
}

fn balance(matches: &ArgMatches) {
    // This unwrap is safe because we require it above
    let address = matches.value_of("address").unwrap();
    println!("I'd love to get your balance on '{}' but it's not yet implemented!", address);
}