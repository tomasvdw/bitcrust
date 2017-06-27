use std::collections::HashSet;
use std::env::home_dir;
use std::net::Ipv6Addr;
use std::str::FromStr;
use std::thread;

use multiqueue::{BroadcastReceiver, BroadcastSender, broadcast_queue};
use rusqlite::{Error, Connection};

use client_message::ClientMessage;
use net_addr::NetAddr;
use peer::Peer;
use services::Services;

pub struct Client {
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

impl Client {
    pub fn new() -> Client {
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

        Client {
            database: db,
            addrs: addrs,
            receiver: receiver,
            sender: sender,
            peers: Vec::with_capacity(100),
        }
    }

    pub fn execute(&mut self) {
        debug!("Executing!");
        self.initialize_peers();
        // self.peers.append(&mut peers);
        // println!("About to make a scope for a peer");
        // debug!("Made a crossbeam scope!");

        // scope.spawn(|| {
        // debug!("Spawning peer thread");
        // });
        // });
        loop {
            info!("Currently connected to {} peers", self.peers.len());
            match self.receiver.recv() {
                Ok(s) => {
                    match s {
                        ClientMessage::Addrs(addrs) => self.addr_message(addrs),
                        ClientMessage::Closing(hostname) => {
                            self.peers
                                .retain(|ref peer| peer.0 != hostname);
                        }
                    }
                }
                Err(e) => {
                    trace!("Some error with thread communication, {:?}", e)
                    // match e {
                    //     Empty => {}
                    //     _ => trace!("Some error with thread communication, {:?}", e),
                    // }
                }
            }
            self.initialize_peers();
        }
    }

    fn addr_message(&mut self, addrs: Vec<NetAddr>) {
        info!("Peer sent us {} new addrs", addrs.len());
        let current_addrs = self.addrs.clone();
        for addr in addrs.into_iter().filter(|addr| current_addrs.contains(addr)) {
            if self.peers.len() < 100 {
                let host = format!("{}:{}", addr.ip, addr.port);
                match Peer::new(&host[..], &self.sender, &self.receiver) {
                    Ok(peer) => {
                        let _ = self.update_time(&addr);
                        self.peers.push((host, peer.run()));
                    }
                    Err(e) => {
                        debug!("Failed to connect to peer at {} :: {:?}", addr.ip, e);
                    }
                };
            }
            match self.add_addr(addr) {
                Ok(_) => {}
                Err(e) => warn!("Error adding addr: {:?}", e),
            }
        }
    }

    fn initialize_peers(&mut self) {
        if self.peers.len() >= 100 {
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
                if self.peers.len() >= 100 {
                    break;
                }
                match Peer::new(&host[..], &self.sender, &self.receiver) {
                    Ok(peer) => {
                        let _ = self.update_time(addr);
                        self.peers.push((host.to_string(), peer.run()));
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
            for hostname in ["seed.bitcoin.sipa.be:8333",
                             "dnsseed.bluematt.me:8333",
                             "dnsseed.bitcoin.dashjr.org:8333",
                             "seed.bitcoinstats.com:8333",
                             // "seed.bitcoin.jonasschnelli.ch:8333",
                             // "seed.btc.petertodd.org:8333"
                             ]
                .iter() {
                // info!("Trying to connect")
                match Peer::new(*hostname, &self.sender, &self.receiver) {
                    Ok(peer) => {
                        self.peers.push((hostname.to_string(), peer.run()));
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
                       &format!("{}", addr.services.encode() as i64),
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
