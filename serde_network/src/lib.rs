extern crate byteorder;

extern crate serde;
use serde::{Serialize, Deserialize};

mod ser;
pub use ser::Serializer;
mod de;
pub use de::Deserializer;
mod error;
pub use error::{Error, Result};
pub use ser::encode_compact_size;
pub use de::decode_compact_size;


pub fn serialize<T>(out: &mut Vec<u8>, value: &T) -> Result<()>
    where T: Serialize
{
    let mut ser = Serializer::new(out);
    Serialize::serialize(value, &mut ser)
}

pub fn deserialize<'de, T>(bytes: &'de [u8]) -> Result<T>
    where T: Deserialize<'de>
{
    Deserializer::new(bytes).deserialize()
}
