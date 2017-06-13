use std::net::Ipv6Addr;

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

#[derive(Clone, Debug, PartialEq)]
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
}
