use std::io::{Error, Read, Write};
use std::net::TcpStream;
use std::time::Duration;
use std::net::Ipv6Addr;
use std::str::FromStr;
use std::time::{UNIX_EPOCH, SystemTime};

use nom::IResult;

use net_addr::NetAddr;
use message::Message;
use message::VersionMessage;
use parser::message;

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
    messages: Vec<Message>,
    buffer: Vec<u8>,
}

impl Peer {
    pub fn new(host: &str) -> Result<Peer, Error> {
        match TcpStream::connect(host) {
            Ok(mut socket) => {
                socket.set_read_timeout(Some(Duration::from_secs(2)))
                    .expect("set_read_timeout call failed");
                socket.set_write_timeout(Some(Duration::from_secs(2)))
                    .expect("set_read_timeout call failed");
                Ok(Peer {
                    socket: socket,
                    messages: Vec::with_capacity(10),
                    buffer: Vec::with_capacity(4096),
                })
            }
            Err(e) => Err(e),
        }
    }

    fn handle_message(&mut self, message: Message) {
        println!("Handling {:?}", message);
    }

    pub fn run(&mut self) {
        let _ = self.send(Peer::version()).unwrap();
        if let Some(message) = self.recv() {
            let _ = self.send(Message::Verack).unwrap();
            let _ = self.send(Message::GetAddr);
            loop {
                if let Some(msg) = self.recv() {
                    self.handle_message(msg);
                } else {
                    println!("[{}] Trying to recieve again", self.buffer.len());
                }
                // sending messages to peers

            }
        } else {
            println!("Failed to understand VERSION packet from remote peer");
        }
    }

    pub fn addrs(&mut self) {
        let message = self.recv().unwrap();
        println!("Message: {:?}", message);
        let _ = self.send(Message::GetAddr).unwrap();
    }

    fn recv(&mut self) -> Option<Message> {
        let mut buffer: Vec<u8> = Vec::with_capacity(8192 + self.buffer.len());
        println!("appending buffer of len: {}", self.buffer.len());
        buffer.append(&mut self.buffer);
        println!("Buffer len: {}", buffer.len());
        let mut buff = [0; 8192];
        let mut read = match self.socket.read(&mut buff) {
            Ok(r) => r,
            Err(_) => {
                self.buffer = buffer;
                return None;
            }
        };
        println!("[{}] Read: {}", buffer.len(), read);
        buffer.extend((buff[0..read]).iter().cloned());
        while read == 8192 {
            if let Ok(r) = self.socket.read(&mut buff) {
                read = r;
                buffer.extend((buff[0..read]).iter().cloned());
            } else {
                break;
            }
            println!("[{}] Read: {}", buffer.len(), read);
        }
        // println!("Read: {}", read);
        // println!("Buff: {:?}", to_hex_string(&messages));
        if buffer.len() == 0 {
            return None;
        } else {
            {
                let message = {
                    message(&buffer)
                };
                match message {
                    IResult::Done(remaining, msg) => {
                        println!("Remaining: {:?}", remaining);
                        self.buffer = remaining.into();
                        println!("Got back {:?}", msg);
                        return Some(msg);
                    }
                    _ => {
                        println!("Problem parsing: {:?}, have: {:?}", buffer, message);
                    }
                };
            }
            self.buffer = buffer;
        };
        None
    }

    fn send(&mut self, message: Message) -> Result<(), Error> {
        println!("About to write: {:?}", message);
        let written = self.socket.write(&message.encode())?;
        println!("Written: {:}", written);
        Ok(())
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
