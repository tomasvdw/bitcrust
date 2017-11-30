
use clap::ArgMatches;
use config::Config;
use util::*;
use store;

use serde_json;

pub fn db_query(matches: &ArgMatches, config: &Config) {

    let db = &mut store::init(&config.data_dir).unwrap();


    match matches.subcommand() {
        ("get-transaction", Some(txhash)) => {
            let txhash = txhash.value_of("tx-hash").unwrap();
            let tx_db = store::transaction_get(db, &hash_from_hex(txhash))
                .unwrap()
                .expect("Not found");

            let tx = tx_db.as_tx().unwrap();

            println!("{}", serde_json::to_string_pretty(&tx).unwrap());


        },
        ("get-block", Some(block_hash)) => {
            let block_hash = block_hash.value_of("block-hash").unwrap();
            let hdr_db = store::header_get(db, &hash_from_hex(block_hash))
                .unwrap()
                .expect("Not found");

            println!("{}", serde_json::to_string_pretty(&hdr_db).unwrap());


        },
        ("", None) => println!("No subcommand was used"), // If no subcommand was usd it'll match the tuple ("", None)
        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachabe!()
    }
}
