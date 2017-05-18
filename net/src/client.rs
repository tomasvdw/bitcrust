use std::net::Ipv6Addr;
use std::str::FromStr;

use rusqlite::{Error, Connection};

use net_addr::NetAddr;
use peer::Peer;

pub struct Client {
    pub peers: Vec<Peer>,
    addrs: Vec<NetAddr>,
    database: Connection,
}

//pub time: Option<u32>,
// pub services: u64,
// pub ip: Ipv6Addr,
// pub port: u16,

impl Client {
    pub fn new() -> Client {
        let db = Connection::open_in_memory().unwrap();

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
                db.prepare("SELECT time, services, ip, port from peers ORDER BY time limit 1000")
                    .unwrap();

            let addrs: Vec<NetAddr> = stmt.query_map(&[], |row| {
                    NetAddr {
                        time: Some(row.get(0)),
                        services: row.get::<_, i64>(1) as u64,
                        ip: Ipv6Addr::from_str(&row.get::<_, String>(2))
                            .unwrap_or(Ipv6Addr::from_str("0.0.0.0").unwrap()),
                        port: row.get(3),
                    }
                })
                .unwrap()
                .filter_map(|l| l.ok())
                .collect();
            addrs
        };

        let mut peers = Vec::with_capacity(110);

        if addrs.len() > 0 {
            for i in 0..100 {
                match addrs.get(i) {
                    Some(addr) => {
                        match Peer::new_with_addrs(&format!("{}", addr.ip), addrs.clone()) {
                            Ok(peer) => peers.push(peer),
                            _ => {}
                        }
                    }
                    _ => {}
                }

            }
        } else {
            // Seeds pulled from https://github.com/bitcoin/bitcoin/blob/master/src/chainparams.cpp
            for hostname in ["seed.bitcoin.sipa.be",
                             "dnsseed.bluematt.me",
                             "dnsseed.bitcoin.dashjr.org",
                             "seed.bitcoinstats.com",
                             "seed.bitcoin.jonasschnelli.ch",
                             "seed.btc.petertodd.org"]
                .iter() {
                match Peer::new(&hostname) {
                    Ok(peer) => peers.push(peer),
                    _ => {}
                }

            }
        };

        Client {
            database: db,
            addrs: addrs,
            peers: peers,
        }
    }

    fn add_addr(&self, addr: NetAddr) -> Result<(), Error> {
        let mut stmt = self.database
            .prepare("INSERT INTO time, services, ip, port VALUES (?, ?, ?, ?)")?;
        stmt.execute(&[&format!("{}", addr.time.unwrap_or(0)),
                       &format!("{}", addr.services as i64),
                       &format!("{}", addr.ip),
                       &format!("{}", addr.port)])?;
        Ok(())
    }
}
