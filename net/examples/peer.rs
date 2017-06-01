extern crate bitcrust_net;
extern crate env_logger;
extern crate multiqueue;

use multiqueue::{BroadcastReceiver, BroadcastSender, broadcast_queue};

use bitcrust_net::peer::Peer;

fn main() {
    env_logger::init().unwrap();
    let (sender, receiver) = broadcast_queue(200);

    let mut peer = Peer::new("seed.bitcoinstats.com:8333", &sender, &receiver)
    // let mut peer = Peer::new("seed.btc.petertodd.org:8333")
        .expect("Could not connect to peer to initialize network connection");
    // let mut peer = Peer::new("127.0.0.1:8333")
    //     .expect("You need to be running bitcoin-core to run tests");
    let thread_handle = peer.run();
    thread_handle.join();
}
