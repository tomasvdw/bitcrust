use std::io::{Error, Read, Write};
use std::net::TcpStream;
use std::time::Duration;
use std::net::Ipv6Addr;
use std::str::FromStr;
use std::time::{UNIX_EPOCH, SystemTime};

use net_addr::NetAddr;
use message::Message;
use message::VersionMessage;

#[cfg(test)]
mod tests {
    use super::*;

}

/// Peer bootstrapping follows the following sequence:
/// New client == -->
/// Remote peer == <--
///
/// --> Version
/// <-- Version
/// --> Verack
///
/// After the handshake, other communication can occur
pub struct Peer {
    socket: TcpStream,
}

impl Peer {
    pub fn new(host: &str) -> Result<Peer, Error> {
        match TcpStream::connect(host) {
            Ok(mut socket) => {
                socket.set_read_timeout(Some(Duration::from_secs(2)))
                    .expect("set_read_timeout call failed");
                socket.set_write_timeout(Some(Duration::from_secs(2)))
                    .expect("set_read_timeout call failed");
                Ok(Peer { socket: socket })
            }
            Err(e) => Err(e),
        }
    }

    pub fn connect(&mut self) {
        let written = self.socket.write(&Peer::version().encode()).unwrap();
        println!("Written: {:}", written);
        let mut buff = [0; 1024];
        let read = self.socket.read(&mut buff).unwrap();
        println!("Read: {}", read);
        println!("Buff: {:?}", to_hex_string(&buff[0..read]));
        let message = Message::try_parse(&buff[0..read]).unwrap();
        println!("Message: {:?}", message);
        let written = self.socket.write(&Message::Verack.encode()).unwrap();

        println!("Written: {:}", written);
    }

    pub fn addrs(&mut self) -> Vec<String> {
        unimplemented!()
    }

    fn version() -> Message {
        Message::Version(VersionMessage {
            version: 70015,
            services: 1,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64,
            addr_recv: NetAddr {
                time: None,
                services: 1,
                ip: Ipv6Addr::from_str("::ffff:127.0.0.1").unwrap(),
                port: 8333,
            },
            addr_send: NetAddr {
                time: None,
                services: 1,
                ip: Ipv6Addr::from_str("::ffff:127.0.0.1").unwrap(),
                port: 8333,
            },
            nonce: 1,
            user_agent: "bitcrust".into(),
            start_height: 0,
            relay: false,
        })
    }
}

fn to_hex_string(bytes: &[u8]) -> String {
    let strs: Vec<String> = bytes.iter()
        .map(|b| format!("{:02X}", b))
        .collect();
    strs.join(" ")
}
