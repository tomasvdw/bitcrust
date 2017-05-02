use std::io::{BufReader, Read, Error};
use std::net::Ipv6Addr;

use byteorder::{BigEndian, LittleEndian, NetworkEndian, ReadBytesExt, WriteBytesExt};

use parser::{net_addr, version_net_addr};

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
            services: 1,
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

#[derive(Debug, PartialEq)]
pub struct NetAddr {
    pub time: Option<u32>,
    pub services: u64,
    pub ip: Ipv6Addr,
    pub port: u16,
}

impl NetAddr {
    pub fn try_parse(input: &[u8], version_packet: bool) -> Result<NetAddr, Error> {
        let mut buf = BufReader::new(input);
        let time = if version_packet {
            None
        } else {
            Some(buf.read_u32::<LittleEndian>()?)
        };
        Ok(NetAddr {
            time: time,
            services: buf.read_u64::<LittleEndian>()?,
            ip: Ipv6Addr::new(buf.read_u16::<NetworkEndian>()?,
                              buf.read_u16::<NetworkEndian>()?,
                              buf.read_u16::<NetworkEndian>()?,
                              buf.read_u16::<NetworkEndian>()?,
                              buf.read_u16::<NetworkEndian>()?,
                              buf.read_u16::<NetworkEndian>()?,
                              buf.read_u16::<NetworkEndian>()?,
                              buf.read_u16::<NetworkEndian>()?),
            port: buf.read_u16::<BigEndian>()?,
        })
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut v: Vec<u8> = Vec::with_capacity(30);
        // write time
        if let Some(t) = self.time {
            let _ = v.write_u32::<LittleEndian>(t);
        }
        // write services
        let _ = v.write_u64::<LittleEndian>(self.services);
        // write IP
        for octet in self.ip.segments().iter() {
            let _ = v.write_u16::<NetworkEndian>(*octet);
        }
        // write port
        let _ = v.write_u16::<BigEndian>(self.port);
        v
    }
}
