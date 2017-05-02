extern crate bitcrust_net;

use bitcrust_net::peer::Peer;

fn main() {
    let mut peer = Peer::new("dashjr.org:8333")
        .expect("You need to be running bitcoin-core to run tests");
    peer.connect();
    // let addrs = peer.addrs();
}
