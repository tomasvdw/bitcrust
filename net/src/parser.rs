use std::net::Ipv6Addr;

use nom;
use nom::{le_u16, le_u32, le_u64, le_i32, le_i64, be_u16, be_u32};

use message::Message;
use message::VersionMessage;
use net_addr::NetAddr;

#[cfg(test)]
mod parse_tests {
    use std::str::FromStr;
    use super::*;

    #[test]
    fn it_parses_an_ipv6_address() {
        // [u8] for ::ffff:10.0.0.1
        let address = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF,
                       0x0A, 0x00, 0x00, 0x01, 0x20, 0x8D];
        let parsed = ipv6(&address).unwrap().1;
        assert_eq!(parsed, Ipv6Addr::from_str("::ffff:10.0.0.1").unwrap());
    }

    #[test]
    fn it_creates_a_net_addr() {
        // [u8] for a netaddr chunk
        let addr_input = [0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                          0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0x0A, 0x00, 0x00, 0x01,
                          0x20, 0x8D];
        let parsed = version_net_addr(&addr_input).unwrap().1;
        assert_eq!(parsed,
                   NetAddr {
                       time: None,
                       services: 1,
                       ip: Ipv6Addr::from_str("::ffff:10.0.0.1").unwrap(),
                       port: 8333,
                   });
    }

    #[test]
    fn it_parses_a_variable_str() {
        let input = [0x0F, 0x2F, 0x53, 0x61, 0x74, 0x6F, 0x73, 0x68, 0x69, 0x3A, 0x30, 0x2E, 0x37,
                     0x2E, 0x32, 0x2F];
        assert_eq!(variable_str(&input).unwrap().1, "/Satoshi:0.7.2/");
    }

    #[test]
    fn it_parses_a_version() {
        let input = [
          0x62, 0xEA, 0x00, 0x00,                                                                                                                                     //- 60002 (protocol version 60002)
          0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,                                                                                                             //- 1 (NODE_NETWORK services)
          0x11, 0xB2, 0xD0, 0x50, 0x00, 0x00, 0x00, 0x00,                                                                                                             //- Tue Dec 18 10:12:33 PST 2012
          0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0x0A, 0x00, 0x00, 0x01, 0x20, 0x8D, //- Recipient address info - see Network Address
          0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0x0A, 0x00, 0x00, 0x01, 0x20, 0x8D, //- Sender address info - see Network Address
          0x3B, 0x2E, 0xB3, 0x5D, 0x8C, 0xE6, 0x17, 0x65,                                                                                                             //- Nonce
          0x0F, 0x2F, 0x53, 0x61, 0x74, 0x6F, 0x73, 0x68, 0x69, 0x3A, 0x30, 0x2E, 0x37, 0x2E, 0x32, 0x2F,                                                             //- "/Satoshi:0.7.2/" sub-version string (string is 15 bytes long)
          0xC0, 0x3E, 0x03, 0x00                                                                                                                                      //- Last block sending node has is block #212672
        ];
        println!("Parsing len: {}", input.len());
        let expected = VersionMessage {
            version: 60002,
            services: 1,
            timestamp: 1355854353,
            addr_recv: NetAddr {
                time: None,
                services: 1,
                ip: Ipv6Addr::from_str("::ffff:10.0.0.1").unwrap(),
                port: 8333,
            },
            addr_send: NetAddr {
                time: None,
                services: 1,
                ip: Ipv6Addr::from_str("::ffff:10.0.0.1").unwrap(),
                port: 8333,
            },
            nonce: 7284544412836900411,
            user_agent: "/Satoshi:0.7.2/".into(),
            start_height: 212672,
            relay: false,
        };
        let actual = version(&input);
        println!("actual: {:?}", actual);
        assert_eq!(expected, actual.unwrap().1);
    }

    #[test]
    fn it_parses_a_version_message() {
        // taken from my Satoshi client's response on 25 April, 2017
        let input = [0xF9, 0xBE, 0xB4, 0xD9, 0x76, 0x65, 0x72, 0x73, 0x69, 0x6F, 0x6E, 0x00, 0x00,
                     0x00, 0x00, 0x00, 0x66, 0x00, 0x00, 0x00, 0x7F, 0xA7, 0xD3, 0xE8, 0x7F, 0x11,
                     0x01, 0x00, 0x0D, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xDA, 0x5E, 0xFF,
                     0x58, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                     0x00, 0x00, 0x00, 0x00, 0x00, 0x0D, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                     0x00, 0x00, 0x00, 0x00, 0x00, 0x2B, 0xA5, 0xBD, 0xC7, 0xD0, 0x38, 0x67, 0x6A,
                     0x10, 0x2F, 0x53, 0x61, 0x74, 0x6F, 0x73, 0x68, 0x69, 0x3A, 0x30, 0x2E, 0x31,
                     0x34, 0x2E, 0x31, 0x2F, 0x59, 0x12, 0x07, 0x00, 0x01, 0xF9, 0xBE, 0xB4, 0xD9,
                     0x76, 0x65, 0x72, 0x61, 0x63, 0x6B, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                     0x00, 0x00, 0x00, 0x5D, 0xF6, 0xE0, 0xE2, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

        let res = message(&input);
        println!("Message: {:?}", res);
        // assert!(res.is_ok())
    }

    #[test]
    fn it_parses_version_from_docs() {
        let input = [
          // Message Header:
          0xF9, 0xBE, 0xB4, 0xD9,                                                                                                                                    //- Main network magic bytes
          0x76, 0x65, 0x72, 0x73, 0x69, 0x6F, 0x6E, 0x00, 0x00, 0x00, 0x00, 0x00,                                                                                    //- "version" command
          0x64, 0x00, 0x00, 0x00,                                                                                                                                    //- Payload is 100 bytes long
          0x30, 0x42, 0x7C, 0xEB,                                                                                                                                    //- payload checksum

          // Version message:
          0x62, 0xEA, 0x00, 0x00,                                                                                                                                     //- 60002 (protocol version 60002)
          0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,                                                                                                             //- 1 (NODE_NETWORK services)
          0x11, 0xB2, 0xD0, 0x50, 0x00, 0x00, 0x00, 0x00,                                                                                                             //- Tue Dec 18 10:12:33 PST 2012
          0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0x0A, 0x00, 0x00, 0x01, 0x20, 0x8D, //- Recipient address info - see Network Address
          0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0x0A, 0x00, 0x00, 0x01, 0x20, 0x8D, //- Sender address info - see Network Address
          0x3B, 0x2E, 0xB3, 0x5D, 0x8C, 0xE6, 0x17, 0x65,                                                                                                             //- Nonce
          0x0F, 0x2F, 0x53, 0x61, 0x74, 0x6F, 0x73, 0x68, 0x69, 0x3A, 0x30, 0x2E, 0x37, 0x2E, 0x32, 0x2F,                                                             //- "/Satoshi:0.7.2/" sub-version string (string is 15 bytes long)
          0xC0, 0x3E, 0x03, 0x00                                                                                                                                      //- Last block sending node has is block #212672
        ];
        let output = message(&input);
        println!("Output: {:?}", output);
    }
}

named!(pub message< Message >,
  do_parse!(
    magic: le_u32 >>
    message_type: take_str!(12) >>
    payload_len: le_u32 >>
    checksum: le_u32 >>
    version_message: cond!(message_type.trim_matches(0x00 as char) == "version", version) >>
    // mes: alt!(
    //   version => {|v| Message::Version(v)}
    // ) >>
    ({
      println!("Message type: {:?}", message_type);
       match message_type.trim_matches(0x00 as char) {
      "verack" => Message::Verack,
      "version" => Message::Version(version_message.unwrap()),
      _ => Message::None
      // _ => unreachable!()
    }})
));


named!(version <VersionMessage>, 
  do_parse!(
    version: le_i32 >>
    services: le_u64 >>
    timestamp: le_i64 >>
    addr_recv: version_net_addr >>
    addr_send: version_net_addr >>
    nonce: le_u64 >>
    user_agent: variable_str >>
    start: le_i32 >>
    // relay: opt!(take!(1)) >>
    (
      VersionMessage {
        version: version,
        services: services,
        timestamp: timestamp,
        addr_recv: addr_recv,
        addr_send: addr_send,
        nonce: nonce,
        user_agent: user_agent,
        start_height: start,
        relay: false, //relay.is_some() && relay.unwrap() == [1],
      }
    )
));

named!(variable_str <String>, 
do_parse!(
  len: compact_size >>
  data: take!(len) >>
  (String::from_utf8_lossy(data).into())
));

named!(compact_size<u64>,
    do_parse!(
      res: alt!(i9 | i5 | i3 | i) >>
      (res as u64)
    )
);

named!(i<u64>,
  do_parse!(
    i: take!(1) >>
    (i[0] as u64)
));

named!(i3<u64>,
  do_parse!(
    tag!([0xfd]) >>
    len: le_u16 >>
    (len as u64)
  )
);

named!(i5<u64>,
  do_parse!(
    tag!([0xfe]) >>
    len: le_u32 >>
    (len as u64)
  )
);

named!(i9<u64>,
  do_parse!(
    tag!([0xff]) >>
    len: le_u64 >>
    (len)
  )
);

named!(ipv6< Ipv6Addr >,
  do_parse!(
    a: be_u16 >>
    b: be_u16 >>
    c: be_u16 >>
    d: be_u16 >>
    e: be_u16 >>
    f: be_u16 >>
    g: be_u16 >>
    h: be_u16 >>
    (Ipv6Addr::new(a, b, c, d, e, f, g, h))
));

named!(pub version_net_addr< NetAddr >,
  do_parse!(
    services: le_u64 >>
    ip: ipv6 >>
    port: be_u16 >>

    (NetAddr {
      time: None,
      services: services,
      ip: ip,
      port: port
    })
));

named!(pub net_addr< NetAddr >,
  do_parse!(
    time: le_u32 >>
    services: le_u64 >>
    ip: ipv6 >>
    port: be_u16 >>

    (NetAddr {
      time: Some(time),
      services: services,
      ip: ip,
      port: port
    })
));
