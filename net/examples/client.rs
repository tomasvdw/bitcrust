extern crate bitcrust_net;
extern crate env_logger;

use bitcrust_net::peer::Peer;

fn main() {
    env_logger::init().unwrap();
    let mut peer = Peer::new("seed.bitcoinstats.com:8333")
    // let mut peer = Peer::new("127.0.0.1:8333")
      .expect("Could not connect to bitcoinstats peer to initialize network connection");
    peer.run();
}
