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
    send_compact: bool,
    send_headers: bool,
    acked: bool,
    addrs: Vec<NetAddr>,
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
                    send_compact: false,
                    send_headers: false,
                    acked: false,
                    addrs: Vec::with_capacity(1000),
                })
            }
            Err(e) => Err(e),
        }
    }

    fn handle_message(&mut self, message: Message) {
        debug!("Handling {:?}", message);
        match message {
            Message::Version(v) => {
                self.send(Peer::version());
            }
            Message::Ping(nonce) => {
                info!("Ping");
                self.send(Message::Pong(nonce));
            }
            Message::SendCompact(msg) => {
                self.send_compact = msg.send_compact;
            }
            Message::Addr(addrs) => {
                info!("Found {} addrs", addrs.addrs.len());
                self.addrs = addrs.addrs;
            }
            Message::SendHeaders => {
                self.send_headers = true;
            }
            Message::Verack => {
                self.acked = true;
            }
            _ => info!("Not handling {:?} yet", message),
        };
    }

    pub fn run(&mut self) {
        let _ = self.send(Peer::version()).unwrap();
        loop {

            if let Some(message) = self.recv() {
                let _ = self.send(Message::Verack).unwrap();
                if self.addrs.len() < 1000 {
                    let _ = self.send(Message::GetAddr);
                }
                break;
            } else {
                debug!("Failed to understand VERSION packet from remote peer");
            }
        }
        loop {
            if let Some(msg) = self.recv() {
                self.handle_message(msg);
            } else {
                debug!("[{}] Trying to recieve again", self.buffer.len());
            }
            // sending messages to peers
            // check if this is bad
        }
    }

    pub fn addrs(&mut self) {
        let message = self.recv().unwrap();
        debug!("Message: {:?}", message);
        let _ = self.send(Message::GetAddr).unwrap();
    }

    fn recv(&mut self) -> Option<Message> {
        let mut buffer: Vec<u8> = Vec::with_capacity(8192 + self.buffer.len());
        // debug!("appending buffer of len: {}", self.buffer.len());
        buffer.append(&mut self.buffer);
        debug!("Buffer len: {}", buffer.len());
        let starting_len = buffer.len();
        if starting_len > 0 {
            match message(&buffer) {
                IResult::Done(remaining, msg) => {
                    self.buffer = remaining.into();
                    // info!("Got back {:?}", msg);
                    return Some(msg);
                }
                _ => {
                    trace!("Problem parsing: {:?}", buffer);
                    trace!("Failed to parse remaining buffer");
                }
            };
        }
        let mut buff = [0; 8192];
        let mut read = match self.socket.read(&mut buff) {
            Ok(r) => r,
            Err(_) => {
                self.buffer = buffer;
                return None;
            }
        };
        debug!("[{}] Read: {}", buffer.len(), read);
        buffer.extend((buff[0..read]).iter().cloned());

        if buffer.len() == 0 || buffer.len() == starting_len {
            return None;
        } else {
            {
                let message = {
                    message(&buffer)
                };
                match message {
                    IResult::Done(remaining, msg) => {
                        debug!("Remaining: {:?}", remaining);
                        self.buffer = remaining.into();
                        // info!("Got back {:?}", msg);
                        return Some(msg);
                    }
                    IResult::Incomplete(len) => {
                        info!("Still need {:?} more bytes", len);
                    }
                    _ => {
                        debug!("Problem parsing: {:?}, have: {:?}", buffer, message);
                        trace!("Problem parsing final buffer");
                    }
                };
            }
            self.buffer = buffer;
        };
        None
    }

    fn send(&mut self, message: Message) -> Result<(), Error> {
        info!("About to write: {:?}", message);
        let written = self.socket.write(&message.encode())?;
        debug!("Written: {:}", written);
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
            user_agent: "/bitcrust:0.1.0/".into(),
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
