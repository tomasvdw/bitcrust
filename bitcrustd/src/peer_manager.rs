use std::collections::HashSet;
use std::env::home_dir;
use std::net::{TcpListener, Ipv6Addr};
use std::sync::mpsc::{channel, TryRecvError};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::str::FromStr;
use std::thread;

use multiqueue::{BroadcastReceiver, BroadcastSender, broadcast_queue};
use rusqlite::{Error, Connection};

use bitcrust_net::{NetAddr, Services};
use client_message::ClientMessage;
use peer::Peer;
use config::Config;

pub struct PeerManager {
    config: Config,
    addrs: HashSet<NetAddr>,
    database: Connection,
    sender: BroadcastSender<ClientMessage>,
    receiver: BroadcastReceiver<ClientMessage>,
    peers: Vec<(String, thread::JoinHandle<()>)>,
}

//pub time: Option<u32>,
// pub services: u64,
// pub ip: Ipv6Addr,
// pub port: u16,

impl PeerManager {
    pub fn new(config: &Config) -> PeerManager {
        let mut path = home_dir().expect("Can't figure out where your $HOME is");
        path.push(".bitcrust.dat");
        debug!("Connecting to DB at {:?}", path);
        let db = Connection::open(path).expect("Couldn't open SQLite connection");
        debug!("Have a database connection");
        db.execute("
             CREATE TABLE IF NOT EXISTS peers (
                 time \
                      INTEGER,
                 services INTEGER,
                 ip TEXT,
                 \
                      port INTEGER
             );",
                     &[])
            .unwrap();

        let addrs: HashSet<NetAddr> = {
            let mut stmt =
                db.prepare("SELECT time, services, ip, port from peers ORDER BY time DESC \
                              limit 1000")
                    .unwrap();

            let addrs: HashSet<NetAddr> = stmt.query_map(&[], |row| {
                    NetAddr {
                        time: Some(row.get(0)),
                        services: Services::from(row.get::<_, i64>(1) as u64),
                        ip: Ipv6Addr::from_str(&row.get::<_, String>(2))
                            .unwrap_or(Ipv6Addr::from_str("::").unwrap()),
                        port: row.get(3),
                    }
                })
                .unwrap()
                .filter_map(|l| l.ok())
                .collect();
            addrs
        };
        info!("Pulled {} addrs from the database", addrs.len());

        let (sender, receiver) = broadcast_queue(200);

        debug!("Setup peers");

        PeerManager {
            config: config.clone(),
            database: db,
            addrs: addrs,
            receiver: receiver,
            sender: sender,
            peers: Vec::with_capacity(100),
        }
    }

    pub fn execute(&mut self) -> ! {
        debug!("Executing!");
        let (sender, receiver) = channel();

        let peer_sender = self.sender.clone();
        let peer_receiver = self.receiver.add_stream();
        let peer_config = self.config.clone();
        let sleep_duration = Duration::from_millis(200);
        let connected_peers = Arc::new(Mutex::new(0));
        let listener_peers = connected_peers.clone();
        thread::spawn(move || {
            let sender = sender.clone();
            info!("Spawning listener");
            let listener = TcpListener::bind("0.0.0.0:8333").unwrap();

            // accept connections and process them serially

            loop {
                match listener.accept() {
                    Ok((socket, addr)) => {
                        let host = format!("{}", addr);
                        let addr = NetAddr::from_socket_addr(addr);
                        match Peer::with_stream(host, socket, &peer_sender, &peer_receiver, &peer_config, ) {
                            Ok(mut peer) => {
                                debug!("new client: {:?}", peer);
                                if let Ok(peers) = listener_peers.try_lock() {
                                    peer.connected_peers(*peers);
                                }
                                let _ = sender.send((addr, peer.run(false)));
                            }
                            Err(e) => {
                                debug!("Some error happened while creating the Peer: {:?}", e);
                            }
                        }
                    }
                    Err(e) => debug!("couldn't get client: {:?}", e),
                }
            }
        });
        // self.initialize_peers();
        let mut recieved = false;
        loop {
            trace!("Currently connected to {}/{} peers",
                   self.peers.len(),
                   self.addrs.len());

            match self.receiver.try_recv() {
                Ok(s) => {

                    recieved = true;
                    match s {
                        ClientMessage::Addrs(addrs) => self.addr_message(addrs),
                        ClientMessage::PeersConnected(_count) => {
                            recieved = false;
                        }
                        ClientMessage::Closing(hostname) => {
                            self.peers
                                .retain(|ref peer| peer.0 != hostname);
                        }
                    }
                }
                Err(e) => {
                    match e {
                        TryRecvError::Empty => {
                            // thread::sleep_ms(200);
                        }
                        TryRecvError::Disconnected => trace!("Remote end has disconnected?"),

                    }

                    // match e {
                    //     Empty => {}
                    //     _ => trace!("Some error with thread communication, {:?}", e),
                    // }
                }
            }
            match receiver.try_recv() {
                Ok((addr, peer_handle)) => {
                    let _ = self.update_time(&addr);
                    self.peers.push((addr.to_host(), peer_handle));
                    debug!("Connected to a new inbound peer");
                    recieved = true;
                }
                Err(e) => {
                    match e {
                        TryRecvError::Empty => {}
                        TryRecvError::Disconnected => trace!("Listener has disconnected?"),
                    }
                }
            }
            // debug!("Receiver: {:?}", self.receiver.len());
            if recieved == false {
                thread::sleep(sleep_duration);
            }
            recieved = false;
            self.initialize_peers(&connected_peers);
            if let Ok(peers) = connected_peers.try_lock() {
                if *peers as usize != self.peers.len() {
                    let _ = self.sender.try_send(ClientMessage::PeersConnected(*peers));
                }
            }
        }
    }

    fn addr_message(&mut self, addrs: Vec<NetAddr>) {
        info!("Peer sent us {} new addrs", addrs.len());
        for addr in addrs.into_iter() {
            match self.add_addr(addr) {
                Ok(_) => {}
                Err(e) => warn!("Error adding addr: {:?}", e),
            }
        }
    }

    fn initialize_peers(&mut self, connected_peers: &Arc<Mutex<u64>>) {
        
        if self.peers.len() >= 10 {
            return;
        }

        if self.addrs.len() > 0 {
            let connected_addrs: Vec<String> = self.peers.iter().map(|t| t.0.clone()).collect();
            for (addr, host) in self.addrs
                .iter()
                .map(|addr| {
                    let s = format!("{}:{}", addr.ip, addr.port);
                    (addr, s)
                })
                .filter(|a| !connected_addrs.contains(&a.1)) {
                if self.peers.len() >= 10 {
                    break;
                }
                match Peer::new(&host[..], &self.sender, &self.receiver, &self.config) {
                    Ok(peer) => {
                        let _ = self.update_time(addr);
                        self.peers.push((host.to_string(), peer.run(true)));
                        if let Ok(mut peers) = connected_peers.try_lock() {
                            *peers += 1;
                        }
                        debug!("Self.peers.len(): {}", self.peers.len());
                    }
                    Err(e) => {
                        debug!("Failed to connect to peer at {} :: {:?}", addr.ip, e);
                        let r = self.remove_addr(&addr);
                        debug!("Removing addr res: {:?}", r);
                    }
                };
            }
        }

        if self.peers.len() == 0 {
            for hostname in ["seed.bitcoinabc.org:8333",
                             "btccash-seeder.bitcoinunlimited.info:8333",
                             ]
                .iter() {
                // info!("Trying to connect")
                match Peer::new(*hostname, &self.sender, &self.receiver, &self.config) {
                    Ok(peer) => {
                        self.peers.push((hostname.to_string(), peer.run(true)));
                    }
                    Err(e) => warn!("Error connecting to {}: {:?}", hostname, e),
                }

            }
        }
    }

    fn add_addr(&mut self, addr: NetAddr) -> Result<(), Error> {
        if self.update_time(&addr).is_some() {
            let updating: Vec<NetAddr> = self.addrs
                .iter()
                .filter(|a| a.ip == addr.ip && a.port == addr.port)
                .map(|a| a.clone())
                .collect();
            for mut a in updating {
                self.addrs.remove(&a);
                a.time = addr.time;
                self.addrs.insert(a);
            }
            return Ok::<(), Error>(());
        }
        let mut stmt = self.database
            .prepare("INSERT INTO peers (time, services, ip, port) VALUES (?, ?, ?, ?)")?;
        stmt.execute(&[&format!("{}", addr.time.unwrap_or(0)),
                       &format!("{}", addr.services.as_i64()),
                       &format!("{}", addr.ip),
                       &format!("{}", addr.port)])?;
        self.addrs.insert(addr.clone());
        Ok(())
    }

    fn remove_addr(&self, addr: &NetAddr) -> Result<(), Error> {
        // self.addrs.retain(|&ref a| !(a.ip == addr.ip && a.port == addr.port));
        let mut stmt = self.database
            .prepare("DELETE FROM peers WHERE ip = ? AND port = ?")?;
        stmt.execute(&[&format!("{}", addr.ip), &format!("{}", addr.port)])?;
        Ok(())
    }

    fn update_time(&self, addr: &NetAddr) -> Option<()> {
        let mut update = match self.database
            .prepare("UPDATE peers SET time = ? WHERE ip = ? AND port = ?") {
            Ok(s) => s,
            Err(e) => {
                warn!("Couldn't prepare peer: {:?}", e);
                return None;
            }
        };
        let count = match update.execute(&[&format!("{}", addr.time.unwrap_or(0)),
                                           &format!("{}", addr.ip),
                                           &format!("{}", addr.port)]) {
            Ok(s) => s,
            Err(e) => {
                warn!("Couldn't update addr's time: {:?}", e);
                return None;
            }
        };
        if count > 0 {
            return Some(());
        }
        None
    }
}
