use std::collections::HashSet;
use std::fmt::{self, Debug};
use std::io::Error;
use std::thread;
use std::time::{Duration, Instant};
use std::net::{Ipv6Addr, TcpStream};
use std::str::FromStr;
use std::time::{UNIX_EPOCH, SystemTime};

use multiqueue::{BroadcastReceiver, BroadcastSender};
use slog;

use bitcrust_net::{BitcoinNetworkConnection, BitcoinNetworkError, NetAddr, Message, AddrMessage, Services,
                   VersionMessage};

use client_message::ClientMessage;
use config::Config;
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
    config: Config,
    host: String,
    network_connection: BitcoinNetworkConnection,
    send_compact: bool,
    send_headers: bool,
    version_sent: bool,
    acked: bool,
    addrs: HashSet<NetAddr>,
    version: Option<VersionMessage>,
    sender: BroadcastSender<ClientMessage>,
    receiver: BroadcastReceiver<ClientMessage>,
    inbound_messages: usize,
    bad_messages: usize,
    peers_connected: u64,
    closed: bool,
    last_read: Instant,
    thread_speed: Duration,
    logger: slog::Logger,
}

impl Debug for Peer {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {

        write!(f,
               r"Peer {{
    host: {},
    send_compact: {},
    send_headers: {},
    version _sent: {},
    acked: {},
    addrs: Vec<NetAddr>,
    inbound_messages: {},
    bad_messages: {},
    peers_connected: {},
    closed: {},
    version: {:?}}}",
               self.host,
               self.send_compact,
               self.send_headers,
               self.version_sent,
               self.acked,
               self.inbound_messages,
               self.bad_messages,
               self.peers_connected,
               self.closed,
               self.version)
    }
}

impl Peer {
    pub fn new<T: Into<String>>(host: T,
                                sender: &BroadcastSender<ClientMessage>,
                                receiver: &BroadcastReceiver<ClientMessage>,
                                config: &Config, logger: &slog::Logger)
                                -> Result<Peer, Error> {
        Peer::new_with_addrs(host, HashSet::with_capacity(1000), sender, receiver, config, logger)
    }

    pub fn with_stream<T: Into<String>>(host: T,
                                        socket: TcpStream,
                                        sender: &BroadcastSender<ClientMessage>,
                                        receiver: &BroadcastReceiver<ClientMessage>,
                                        config: &Config, logger: &slog::Logger) -> Result<Peer, Error> {
        let host = host.into();
        let logger = logger.new(o!("host" => host.clone()));
        debug!(logger, "Initialized incoming peer with host: {}", host);
        let connection = BitcoinNetworkConnection::with_stream(host.clone(), socket, &logger)?;
        Ok(Peer {
            config: config.clone(),
            host: host,
            network_connection: connection,
            send_compact: false,
            send_headers: false,
            version_sent: false,
            acked: false,
            addrs: HashSet::with_capacity(1000),
            version: None,
            sender: sender.clone(),
            receiver: receiver.add_stream(),
            inbound_messages: 0,
            bad_messages: 0,
            peers_connected: 0,
            last_read: Instant::now(),
            closed: false,
            thread_speed: Duration::from_millis(250),
            logger: logger,
        })
    }

    pub fn new_with_addrs<T: Into<String>>(host: T,
                                           addrs: HashSet<NetAddr>,
                                           sender: &BroadcastSender<ClientMessage>,
                                           receiver: &BroadcastReceiver<ClientMessage>,
                                           config: &Config, logger: &slog::Logger)
                                           -> Result<Peer, Error> {
        let host = host.into();
        let logger = logger.new(o!("host" => host.clone()));
        let connection = BitcoinNetworkConnection::new(host.clone(), &logger)?;
        Ok(Peer {
            config: config.clone(),
            host: host,
            network_connection: connection,
            send_compact: false,
            send_headers: false,
            version_sent: false,
            acked: false,
            addrs: addrs,
            version: None,
            sender: sender.clone(),
            receiver: receiver.add_stream(),
            inbound_messages: 0,
            bad_messages: 0,
            peers_connected: 0,
            last_read: Instant::now(),
            closed: false,
            thread_speed: Duration::from_millis(250),
            logger: logger,
        })
    }

    pub fn connected_peers(&mut self, peers: u64) {
        self.peers_connected = peers;
    }

    fn handle_message(&mut self, message: Message) {
        self.inbound_messages += 1;
        if self.version.is_none() {
            match message {
                Message::Version(_) => {}
                _ => {
                    debug!(self.logger, "Received {:?} prior to VERSION", message);
                    return;
                }
            }
        }
        match message {
            Message::Version(v) => {
                match self.version {
                    None => {
                        self.version = Some(v);
                        if self.version_sent {
                            let _ = self.send(Message::Verack);
                            if self.addrs.len() < 100 {
                                let _ = self.send(Message::GetAddr);
                            }
                        } else {
                            let _ = self.send(Peer::version());
                        }
                    }
                    Some(_) => {
                        let _ = self.send(Peer::version());
                    }
                }
            }
            Message::Verack => {
                self.acked = true;
            }
            Message::FeeFilter(_fee) => {}
            Message::Ping(nonce) => {
                debug!(self.logger, "[{}] Ping", self.host);
                let _ = self.send(Message::Pong(nonce));
            }
            Message::Pong(_nonce) => {}
            Message::SendCompact(msg) => {
                self.send_compact = msg.send_compact;
            }
            Message::Addr(addrs) => {
                let _ = self.sender.try_send(ClientMessage::Addrs(addrs.addrs.clone()));
                // Ensure that we don't realocate repeatedly in here
                self.addrs.reserve(addrs.addrs.len());
                for addr in addrs.addrs {
                    self.addrs.insert(addr);
                }
            }
            Message::GetAddr => {
                let msg = AddrMessage { addrs: self.addrs.iter().cloned().collect() };
                let _ = self.send(Message::Addr(msg));
            }
            Message::SendHeaders => {
                self.send_headers = true;
            }
            Message::GetHeaders(_msg) => {}
            Message::Header(_header) => {}
            Message::Inv(_inv) => {}
            Message::Tx(_transaction) => {}
            Message::GetBlocks(_get_blocks) => {}
            Message::GetData(_data) => {}
            Message::Block(_block) => {}
            Message::NotFound(_not_found) => {}
            Message::Unparsed(name, message) => {
                // Support for alert messages has been removed from bitcoin core in March 2016.
                // Read more at https://github.com/bitcoin/bitcoin/pull/7692
                if name != "alert" {
                    debug!(self.logger, "{} : Not handling {} yet ({:?})",
                           self.host,
                           name,
                           to_hex_string(&message))
                }
            }
            // Bitcrust Specific Messages
            Message::BitcrustPeerCountRequest(msg) => {
                if msg.valid(&self.config.key()) {
                    let count = self.peers_connected;
                    let _ = self.send(Message::BitcrustPeerCount(count));
                } else {
                    warn!(self.logger, "Message: {:?}", msg);
                    warn!(self.logger, "Invalid authenticated request!");
                    self.closed = true;
                }
            }
            Message::BitcrustPeerCount(_count) => {}
            _ => {
                debug!(self.logger, "Not handling {:?} yet", message);
            }
        };
    }

    pub fn run(mut self, send_version: bool) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let mut last_cleanup = Instant::now();
            if send_version {
                let _ = self.send(Peer::version());
                self.version_sent = true;
            }
            
            loop {
                // Handle messages coming in from the network
                self.handle_remote_peer_messages();
                // Handle messages from other local peer connections
                self.handle_local_peer_message();
                // check if this is a bad peer
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
                            warn!(self.logger, "{} sent us {} requests, and {} bad ones",
                                  self.host,
                                  self.inbound_messages,
                                  self.bad_messages);
                            break;
                        }
                    }
                }
                if self.closed {
                    break;
                }
            }
            let _ = self.sender.try_send(ClientMessage::Closing(self.host));
        })

    }

    fn handle_remote_peer_messages(&mut self) {
        if self.last_read.elapsed() < self.thread_speed {
            let time = self.thread_speed - self.last_read.elapsed();
            thread::sleep(time);
        }
        self.last_read = Instant::now();
        while let Some(msg) = self.network_connection.try_recv() {
            match msg {
                Ok(msg) => self.handle_message(msg),
                Err(e) => {
                    match e {
                        BitcoinNetworkError::BadBytes => {
                            self.bad_messages += 1;
                        }
                        BitcoinNetworkError::Closed => self.closed = true,
                        BitcoinNetworkError::ReadTimeout => {}
                    }
                    return
                },
            }

        }
    }

    fn handle_local_peer_message(&mut self) {
        while let Ok(msg) = self.receiver.try_recv() {
            match msg {
                ClientMessage::Addrs(addrs) => {
                    // We only want to send `Addr`s that we don't think the remote
                    // peer knows about already
                    let mut addrs_to_send = Vec::with_capacity(addrs.len());
                    for addr in addrs {
                        if !self.addrs.insert(addr.clone()) {
                            addrs_to_send.push(addr);
                        }
                    }
                    let _ = self.send(Message::Addr(AddrMessage { addrs: addrs_to_send }));
                }
                ClientMessage::PeersConnected(count) => {
                    self.peers_connected = count;
                }
                ClientMessage::Closing(_) => {}
                // _ => info!("Ignoring msg: {:?}", msg),
            }
        }
    }

    fn send(&mut self, message: Message) -> Result<(), Error> {
        self.network_connection.try_send(message)
    }

    pub fn version() -> Message {
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
