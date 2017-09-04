use std::hash::{Hash, Hasher};
use std::io;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::time::{SystemTime, UNIX_EPOCH};

use byteorder::{BigEndian, WriteBytesExt};

use Encode;
use services::Services;

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use Encode;
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

        let mut encoded = vec![];
        addr.encode(&mut encoded).unwrap();
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

impl Encode for NetAddr {
    fn encode(&self, mut v: &mut Vec<u8>) -> Result<(), io::Error> {
        // write time
        self.time.encode(&mut v)?;
        // write services
        self.services.encode(&mut v)?;
        // write IP
        self.ip.encode(&mut v)?;
        // write port
        v.write_u16::<BigEndian>(self.port)?;
        Ok(())
    }
}

impl Hash for NetAddr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.services.hash(state);
        self.ip.hash(state);
        self.port.hash(state);
    }
}
