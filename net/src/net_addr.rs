use std::hash::{Hash, Hasher};
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::time::{SystemTime, UNIX_EPOCH};

use byteorder::{BigEndian, LittleEndian, NetworkEndian, WriteBytesExt};

use services::Services;

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use super::*;
    #[test]
    fn it_parses_a_net_address() {}

    #[test]
    fn it_encodes_a_net_address() {
        let addr = NetAddr {
            time: None,
            services: Services::from(1),
            ip: Ipv6Addr::from_str("::ffff:10.0.0.1").unwrap(),
            port: 8333,
        };

        let encoded = addr.encode();
        let expected = vec![
          0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // services
          0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0x0A, 0x00, 0x00, 0x01, // IP
          0x20, 0x8d // port
        ];
        assert_eq!(expected, encoded);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NetAddr {
    pub time: Option<u32>,
    pub services: Services,
    pub ip: Ipv6Addr,
    pub port: u16,
}

impl NetAddr {
    pub fn encode(&self) -> Vec<u8> {
        let mut v: Vec<u8> = Vec::with_capacity(30);
        // write time
        if let Some(t) = self.time {
            let _ = v.write_u32::<LittleEndian>(t);
        }
        // write services
        let _ = v.write_u64::<LittleEndian>(self.services.encode());
        // write IP
        for octet in self.ip.segments().iter() {
            let _ = v.write_u16::<NetworkEndian>(*octet);
        }
        // write port
        let _ = v.write_u16::<BigEndian>(self.port);
        v
    }

    pub fn from_socket_addr(addr: SocketAddr) -> NetAddr {
        let start = SystemTime::now();
        let now = start.duration_since(UNIX_EPOCH)
            .expect("Time went backwards").as_secs();
        let ip = match addr.ip() {
            IpAddr::V4(a) => a.to_ipv6_mapped(),
            IpAddr::V6(a) => a
        };
        NetAddr {
            time: Some(now as u32),
            services: Services::from(0),
            ip: ip,
            port: addr.port(),
        }
    }

    pub fn to_host(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}

impl Hash for NetAddr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.services.hash(state);
        self.ip.hash(state);
        self.port.hash(state);
    }
}
