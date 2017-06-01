use std::env::home_dir;
use std::net::Ipv6Addr;
use std::str::FromStr;
use std::thread;

use multiqueue::{BroadcastReceiver, BroadcastSender, broadcast_queue};
use rusqlite::{Error, Connection};

use client_message::ClientMessage;
use net_addr::NetAddr;
use peer::Peer;

pub struct Client {
    addrs: Vec<NetAddr>,
    database: Connection,
    sender: BroadcastSender<ClientMessage>,
    receiver: BroadcastReceiver<ClientMessage>,
    peers: Vec<thread::JoinHandle<()>>,
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

        let addrs: Vec<NetAddr> = {
            let mut stmt =
                db.prepare("SELECT time, services, ip, port from peers ORDER BY time DESC \
                              limit 1000")
                    .unwrap();

            let addrs: Vec<NetAddr> = stmt.query_map(&[], |row| {
                    NetAddr {
                        time: Some(row.get(0)),
                        services: row.get::<_, i64>(1) as u64,
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
        let mut peers = self.initialize_peers();
        self.peers.append(&mut peers);
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
                        ClientMessage::Addrs(addrs) => {
                            info!("Peer sent us {} new addrs", addrs.len());
                            for addr in addrs {
                                if self.peers.len() < 100 {
                                    match Peer::new_with_addrs(&format!("{}:{}",
                                                                        addr.ip,
                                                                        addr.port),
                                                               self.addrs.clone(),
                                                               &self.sender,
                                                               &self.receiver) {
                                        Ok(peer) => {
                                            let _ = self.update_time(&addr);
                                            self.peers.push(peer.run())
                                        }
                                        Err(e) => {
                                            debug!("Failed to connect to peer at {} :: {:?}",
                                                   addr.ip,
                                                   e);
                                        }
                                    };
                                }
                                match self.add_addr(addr) {
                                    Ok(_) => {}
                                    Err(e) => warn!("Error adding addr: {:?}", e),
                                }
                            }
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
        }
    }

    fn initialize_peers(&mut self) -> Vec<thread::JoinHandle<()>> {
        let mut threads = vec![];
        if self.addrs.len() > 0 {
            for addr in &self.addrs {
                if threads.len() >= 100 {
                    break;
                }
                match Peer::new_with_addrs(&format!("{}:{}", addr.ip, addr.port),
                                           self.addrs.clone(),
                                           &self.sender,
                                           &self.receiver) {
                    Ok(peer) => {
                        let _ = self.update_time(addr);
                        threads.push(peer.run())
                    }
                    Err(e) => {
                        debug!("Failed to connect to peer at {} :: {:?}", addr.ip, e);
                        let _ = self.remove_addr(&addr);
                    }
                };

            }
        }

        if self.peers.len() + threads.len() == 0 {
            for hostname in ["seed.bitcoin.sipa.be:8333",
                             "dnsseed.bluematt.me:8333",
                             "dnsseed.bitcoin.dashjr.org:8333",
                             "seed.bitcoinstats.com:8333",
                             // "seed.bitcoin.jonasschnelli.ch:8333",
                             // "seed.btc.petertodd.org:8333"
                             ]
                .iter() {
                // info!("Trying to connect")
                match Peer::new(&hostname, &self.sender, &self.receiver) {
                    Ok(peer) => threads.push(peer.run()),
                    Err(e) => warn!("Error connecting to {}: {:?}", hostname, e),
                }

            }
        }
        threads
    }

    fn add_addr(&mut self, addr: NetAddr) -> Result<(), Error> {
        if self.update_time(&addr).is_some() {
            for a in self.addrs
                .iter_mut()
                .filter(|a| a.ip == addr.ip && a.port == addr.port) {
                a.time = addr.time;
            }
            return Ok::<(), Error>(());
        }
        let mut stmt = self.database
            .prepare("INSERT INTO peers (time, services, ip, port) VALUES (?, ?, ?, ?)")?;
        stmt.execute(&[&format!("{}", addr.time.unwrap_or(0)),
                       &format!("{}", addr.services as i64),
                       &format!("{}", addr.ip),
                       &format!("{}", addr.port)])?;
        self.addrs.push(addr.clone());
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
