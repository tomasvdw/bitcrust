extern crate bitcrust_net;
extern crate env_logger;

use bitcrust_net::client::Client;

fn main() {
    env_logger::init().unwrap();
    let mut client = Client::new();
    client.execute();
    // let mut peer = Peer::new("127.0.0.1:8333")
    // .expect("Could not connect to bitcoinstats peer to initialize network connection");

}
