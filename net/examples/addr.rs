extern crate bitcrust_net;

use bitcrust_net::peer::Peer;

fn main() {
    // let mut peer = Peer::new("dashjr.org:8333")
    let mut peer = Peer::new("127.0.0.1:8333")
        .expect("You need to be running bitcoin-core to run tests");
    peer.run();
    // let addrs = peer.addrs();
}
