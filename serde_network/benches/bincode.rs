#![feature(test)]

extern crate byteorder;

#[macro_use]
extern crate serde_derive;

extern crate bincode;
extern crate serde;
extern crate serde_bench;
extern crate test;

use bincode::Infinite;
use byteorder::NetworkEndian;
use serde::{Serialize, Deserialize};
use test::Bencher;

#[derive(Serialize, Deserialize)]
struct Foo {
    bar: String,
    baz: u64,
    derp: bool,
}

impl Default for Foo {
    fn default() -> Self {
        Foo {
            bar: "hello".into(),
            baz: 1337u64,
            derp: true,
        }
    }
}

#[bench]
fn bincode_deserialize(b: &mut Bencher) {
    let foo = Foo::default();
    let mut bytes = Vec::with_capacity(128);
    type BincodeSerializer<W> = bincode::internal::Serializer<W, NetworkEndian>;
    foo.serialize(&mut BincodeSerializer::new(&mut bytes)).unwrap();

    b.iter(|| {
        type BincodeDeserializer<R, S> = bincode::internal::Deserializer<R, S, NetworkEndian>;
        let read = bincode::read_types::SliceReader::new(&bytes);
        let mut de = BincodeDeserializer::new(read, Infinite);
        Foo::deserialize(&mut de).unwrap()
    })
}

#[bench]
fn bincode_serialize(b: &mut Bencher) {
    let foo = Foo::default();

    b.iter(|| {
        let mut bytes = Vec::with_capacity(128);
        type BincodeSerializer<W> = bincode::internal::Serializer<W, NetworkEndian>;
        foo.serialize(&mut BincodeSerializer::new(&mut bytes)).unwrap()
    })
}

#[bench]
fn serde_deserialize(b: &mut Bencher) {
    let foo = Foo::default();
    let mut bytes = Vec::new();
    serde_bench::serialize(&mut bytes, &foo).unwrap();

    b.iter(|| {
        serde_bench::deserialize::<Foo>(&bytes).unwrap()
    })
}

#[bench]
fn serde_serialize(b: &mut Bencher) {
    let foo = Foo::default();

    b.iter(|| {
        let mut bytes = Vec::with_capacity(128);
        serde_bench::serialize(&mut bytes, &foo).unwrap()
    })
}
