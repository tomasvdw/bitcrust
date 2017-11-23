

extern crate serde_network;

#[macro_use]
extern crate serde_derive;

extern crate serde;


#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Test {
    pub primitive: u32,
    pub fixed_size: [u32;2],

    pub var_size: Vec<u32>,
    pub t2: Test2
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Test2 {
    pub primitive: u32
}

#[test]
fn test_encode_decode() {

    let val = Test {
        primitive: 17,
        fixed_size: [15,16],
        var_size: vec![21,22,23],
        t2: Test2 { primitive: 31 }
    };

    let buf = &mut Vec::new();
    serde_network::serialize(buf, &val).unwrap();
    assert_eq!(buf[0], 17u8);
    assert_eq!(buf[1], 0);

    assert_eq!(buf[4], 15);
    assert_eq!(buf[5], 0);
    assert_eq!(buf[8], 16);
    assert_eq!(buf[9], 0);

    assert_eq!(buf[12], 3);
    assert_eq!(buf[13], 21);
    assert_eq!(buf[14], 0);
    assert_eq!(buf[17], 22);
    assert_eq!(buf[21], 23);
    assert_eq!(buf[25], 31);


    let back = serde_network::deserialize(buf).unwrap();
    assert_eq!(val, back);
}

#[test]
fn test_deserialize() {

    // test deserialization by parts
    let x = vec![12u8,13u8];

    let mut de = serde_network::Deserializer::new(&x);

    let u1: u8 = de.deserialize().unwrap() ;
    let u2: u8 = de.deserialize().unwrap() ;
    assert_eq!(12u8, u1);
    assert_eq!(13u8, u2);

}
