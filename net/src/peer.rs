use std::fmt::{self, Debug};
use std::io::{Error, Read, Write};
use std::net::TcpStream;
use std::thread;
use std::time::{Duration, Instant};
use std::net::Ipv6Addr;
use std::str::FromStr;
use std::time::{UNIX_EPOCH, SystemTime};

use circular::Buffer;
use multiqueue::{BroadcastReceiver, BroadcastSender};
use nom::{ErrorKind, Err, IResult, Needed};

use client_message::ClientMessage;
use net_addr::NetAddr;
use message::Message;
use message::{AddrMessage, VersionMessage};
use parser::message;
use services::Services;

#[cfg(test)]
mod tests {}

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
    host: String,
    buffer: Buffer,
    socket: TcpStream,
    /// Bytes that we need to parse the next message
    needed: usize,
    send_compact: bool,
    send_headers: bool,
    acked: bool,
    addrs: Vec<NetAddr>,
    version: Option<VersionMessage>,
    sender: BroadcastSender<ClientMessage>,
    receiver: BroadcastReceiver<ClientMessage>,
    inbound_messages: usize,
    bad_messages: usize,
    last_read: Instant,
    thread_speed: Duration,
}

impl Debug for Peer {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {

        write!(f,
               r"Peer {{
    host: {},
    send_compact: {},
    send_headers: {},
    acked: {},
    addrs: Vec<NetAddr>,
    version: {:?}}}",
               self.host,
               self.send_compact,
               self.send_headers,
               self.acked,
               self.version)
    }
}

impl Peer {
    pub fn new(host: &str,
               sender: &BroadcastSender<ClientMessage>,
               receiver: &BroadcastReceiver<ClientMessage>)
               -> Result<Peer, Error> {
        Peer::new_with_addrs(host, Vec::with_capacity(1000), sender, receiver)
    }

    pub fn new_with_addrs(host: &str,
                          addrs: Vec<NetAddr>,
                          sender: &BroadcastSender<ClientMessage>,
                          receiver: &BroadcastReceiver<ClientMessage>)
                          -> Result<Peer, Error> {
        info!("Trying to initialize connection to {}", host);
        let socket = TcpStream::connect(host)?;
        info!("Connected to {}", host);
        socket.set_read_timeout(Some(Duration::from_secs(2)))?;
        // .expect("set_read_timeout call failed");
        socket.set_write_timeout(Some(Duration::from_secs(2)))?;
        // .expect("set_read_timeout call failed");
        Ok(Peer {
            host: host.into(),
            socket: socket,
            // Allocate a buffer with 128k of capacity
            buffer: Buffer::with_capacity(1024 * 128),
            needed: 0,
            send_compact: false,
            send_headers: false,
            acked: false,
            addrs: addrs,
            version: None,
            sender: sender.clone(),
            receiver: receiver.clone(),
            inbound_messages: 0,
            bad_messages: 0,
            last_read: Instant::now(),
            thread_speed: Duration::from_millis(100),
        })
    }

    fn handle_message(&mut self, message: Message) {
        self.inbound_messages += 1;
        match message {
            Message::Version(v) => {
                self.version = Some(v);
                let _ = self.send(Peer::version());
            }
            Message::Ping(nonce) => {
                debug!("Ping");
                let _ = self.send(Message::Pong(nonce));
            }
            Message::SendCompact(msg) => {
                self.send_compact = msg.send_compact;
            }
            Message::Addr(mut addrs) => {
                debug!("Found {} addrs", addrs.addrs.len());
                let _ = self.sender.try_send(ClientMessage::Addrs(addrs.addrs.clone()));
                self.addrs.append(&mut addrs.addrs);
            }
            Message::GetAddr => {
                let msg = AddrMessage { addrs: self.addrs.clone() };
                let _ = self.send(Message::Addr(msg));
            }
            Message::SendHeaders => {
                self.send_headers = true;
            }
            Message::GetHeaders(msg) => {}
            Message::Verack => {
                self.acked = true;
            }
            Message::Inv(inv) => {}
            Message::Unparsed(name, message) => {
                // Support for alert messages has been removed from bitcoin core in March 2016.
                // Read more at https://github.com/bitcoin/bitcoin/pull/7692
                if name != "alert" {
                    debug!("{} : Not handling {} yet ({:?})",
                           self.host,
                           name,
                           to_hex_string(&message))
                }
            }
            _ => {
                debug!("Not handling {:?} yet", message);
            }
        };
    }

    pub fn run(mut self) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let mut last_cleanup = Instant::now();
            let _ = self.send(Peer::version()).unwrap();
            loop {
                match self.recv() {
                    Some(Message::Version(message)) => {
                        self.acked = true;
                        debug!("Connected to a peer running: {}", message.user_agent);
                        self.version = Some(message);
                        let _ = self.send(Message::Verack).unwrap();
                        if self.addrs.len() < 100 {
                            let _ = self.send(Message::GetAddr);
                        }
                        break;
                    }
                    Some(s) => debug!("Received {:?} prior to VERSION", s),
                    _ => {
                        trace!("{} - Haven't yet received VERSION packet from remote peer",
                               self.host)
                    }
                }
            }
            loop {
                if let Some(msg) = self.recv() {
                    self.handle_message(msg);
                } else {
                    trace!("{} :: [{}] Trying to recieve again",
                           self.host,
                           self.buffer.available_data());
                }
                if let Ok(msg) = self.receiver.try_recv() {
                    match msg {
                        ClientMessage::Addrs(addrs) => {
                            // Need some more logic around when to send the AddrMessage
                            // to the peer
                            // let _ = self.send(Message::Addr(AddrMessage { addrs: addrs }));
                        }
                        ClientMessage::Closing(_) => {}
                        // _ => info!("Ignoring msg: {:?}", msg),
                    }
                }
                // sending messages to peers
                // check if this is bad
                if last_cleanup.elapsed() > self.thread_speed * 5 {
                    last_cleanup = Instant::now();
                    self.inbound_messages = match self.inbound_messages.checked_sub(1) {
                        Some(u) => u,
                        None => self.inbound_messages,
                    };
                    self.bad_messages = match self.bad_messages.checked_sub(2) {
                        Some(u) => u,
                        None => self.bad_messages,
                    };
                    if self.inbound_messages > 0 {
                        if self.bad_messages >= self.inbound_messages * 2 {
                            warn!("{} sent us {} requests, and {} bad ones",
                                  self.host,
                                  self.inbound_messages,
                                  self.bad_messages);
                            break;
                        }
                    }
                }
            }
            let _ = self.sender.try_send(ClientMessage::Closing(self.host));
        })

    }

    pub fn addrs(&mut self) {
        let message = self.recv().unwrap();
        debug!("Message: {:?}", message);
        let _ = self.send(Message::GetAddr).unwrap();
    }

    fn try_parse(&mut self) -> Option<Message> {
        let available_data = self.buffer.available_data();
        if available_data == 0 {
            return None;
        }
        let mut trim = false;
        let mut consume = 0;
        let parsed = match message(&self.buffer.data(), &self.host) {
            IResult::Done(remaining, msg) => Some((msg, remaining.len())),
            IResult::Incomplete(len) => {
                if let Needed::Size(s) = len {
                    self.needed = s;
                }
                None
            }
            IResult::Error(e) => {
                match e {
                    Err::Code(ErrorKind::Custom(i)) => {
                        warn!("{} Gave us bad data!", self.host);
                        consume = i;
                        trim = true;
                    }
                    _ => {
                        // debug!("{} - Failed to parse remaining buffer", self.host, e);
                        consume = 1;
                        trim = true;
                    }
                }
                None
            }
        };
        if let Some((message, remaining_len)) = parsed {
            self.buffer.consume(available_data - remaining_len);
            self.needed = 0;
            return Some(message);
        }

        self.buffer.consume(consume as usize);
        if trim {
            self.bad_messages += 1;
        }
        None
    }

    fn recv(&mut self) -> Option<Message> {
        trace!("Buffer len: {}", self.buffer.available_data());
        if let Some(message) = self.try_parse() {
            return Some(message);
        }
        if self.last_read.elapsed() < self.thread_speed {
            let time = self.thread_speed - self.last_read.elapsed();
            trace!("{} - Sleeping for {:?}", self.host, time);
            thread::sleep(time);
        }
        self.last_read = Instant::now();
        let len = self.buffer.available_data();

        self.read();
        // If we haven't received any more data
        if self.buffer.available_data() < self.needed || self.buffer.available_data() == 0 ||
           self.buffer.available_data() == len {
            return None;
        }

        // let _ = self.sender.try_send(ClientMessage::Alive(self.id));

        if let Some(message) = self.try_parse() {
            return Some(message);
        }
        None
    }

    fn read(&mut self) {
        let mut buff = [0; 8192];
        let read = match self.socket.read(&mut buff) {
            Ok(r) => r,
            Err(e) => {
                trace!("Socket read error? {:?}", e);
                return;
            }
        };
        if read == 0 {
            return;
        }
        trace!("{} [{} / {}] Read: {}, Need: {}",
               self.host,
               self.buffer.available_data(),
               self.buffer.capacity(),
               read,
               self.needed);
        self.buffer.grow(read);
        let _ = self.buffer.write(&buff[0..read]);
        if self.needed >= read {
            self.needed -= read;
        } else {
            self.needed = 0;
            return;
        }
    }

    fn send(&mut self, message: Message) -> Result<(), Error> {
        trace!("{} About to write: {:?}", self.host, message);
        let written = self.socket.write(&message.encode())?;
        trace!("{} Written: {:}", self.host, written);
        Ok(())
    }

    fn version() -> Message {
        Message::Version(VersionMessage {
            version: 70015,
            services: Services::from(1),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64,
            addr_recv: Peer::addr(Ipv6Addr::from_str("::ffff:127.0.0.1").unwrap(), 8333, None),
            addr_send: Peer::addr(Ipv6Addr::from_str("::ffff:127.0.0.1").unwrap(), 8333, None),
            nonce: 1,
            user_agent: "/bitcrust:0.1.0/".into(),
            start_height: 0,
            relay: false,
        })
    }

    fn addr(ip: Ipv6Addr, port: u16, time: Option<u32>) -> NetAddr {
        NetAddr {
            time: time,
            services: Services::from(1),
            ip: ip,
            port: port,
        }
    }
}

fn to_hex_string(bytes: &[u8]) -> String {
    let strs: Vec<String> = bytes.iter()
        .map(|b| format!("{:02X}", b))
        .collect();
    strs.join(" ")
}
