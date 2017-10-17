
#[macro_use]
extern crate encode_derive;
extern crate bitcrust_net;

use bitcrust_net::{Encode, VarInt};

#[derive(Encode)]
struct TestStructWithCount {
    name: String,
    #[count]
    data: Vec<u8>
}

#[derive(Encode)]
struct TestStruct {
    name: String,
    data: [u8; 32]
}

#[test]
fn it_encodes() {
    let t = TestStruct {
        name: "TestStruct".into(),
        data: [0x00; 32],
    };
    let mut encoded = vec![];
    let _ = t.encode(&mut encoded);
    assert_eq!(encoded, vec![
        // name
        84, 101, 115, 116, 83, 116, 114, 117, 99, 116,
        // data
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
}

#[test]
fn it_encodes_with_count() {
    let t = TestStructWithCount {
        name: "TestStruct".into(),
        data: vec![0x00],
    };
    let mut encoded = vec![];
    let _ = t.encode(&mut encoded);
    assert_eq!(encoded, vec![
        // name
        84, 101, 115, 116, 83, 116, 114, 117, 99, 116,
        // len
        1,
        // data
        0]);
}