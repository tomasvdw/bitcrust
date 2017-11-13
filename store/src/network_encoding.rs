
//! A buffer is used for decoding raw bytes
//!
//! It is represented by a copyable slice
//!
//! This is normally imported as buffer::* such that the
//! pub's can be considered to be in the global namespace

use std::mem;

#[derive(Debug)]
pub struct EndOfBufferError;

#[derive(Clone,Copy,Debug)]
pub struct Buffer<'a> {
    pub inner: &'a[u8]
}


/// Trait implemented for types that can read and itself from a Buffer
/// and write themselves to a vec in network encoding format
///
/// Should be implemented with zero-copying
pub trait NetworkEncoding<'a>
    where Self: Sized
{
    fn decode(buf: &mut Buffer<'a>) -> Result<Self, EndOfBufferError>;
    fn encode(&self, &mut Vec<u8>);
}




impl<'a> Buffer<'a> {

    /// Constructs a buffer from a slice
    /// The buffer makes the slice Copyable
    pub fn new(slice: &'a[u8]) -> Self {
        Buffer {
            inner: slice
        }
    }

    pub fn len(self) -> usize {
        self.inner.len()
    }

    /// Returns the buffer of what is contained in `original` that is no longer
    /// in self. That is: returns whatever is consumed since original
    pub fn consumed_since(self, original: Buffer) -> Buffer {
        let len = original.inner.len() - self.inner.len();
        Buffer {
            inner: &original.inner[..len]
        }
    }


    /// Parse a compact size
    /// This can be 1-8 bytes; see bitcoin-spec for details
    pub fn decode_compact_size(&mut self) -> Result<usize, EndOfBufferError> {
        let byte1 = { try!(u8::decode(self)) };
        Ok(match byte1 {
            0xff => { try!(u64::decode(self)) as usize },
            0xfe => { try!(u32::decode(self)) as usize },
            0xfd => { try!(u16::decode(self)) as usize },
            _ => byte1 as usize
        })
    }

    /// Parses given amount of bytes
    pub fn decode_bytes(&mut self, count: usize) -> Result<&'a[u8], EndOfBufferError> {
        if self.inner.len() < count {
            return Err(EndOfBufferError);
        }

        // split in result, and remaining
        let result = &self.inner[..count];
        self.inner = &self.inner[count..];

        Ok(result)
    }

    pub fn decode_compact_size_bytes(&mut self) -> Result<&'a[u8], EndOfBufferError> {
        let count = try!(self.decode_compact_size());

        self.decode_bytes(count)
    }

}

pub fn encode_compact_size(buffer: &mut Vec<u8>, sz: usize) {
    if sz < 0xFD {
        buffer.push(sz as u8);
    }
    else if sz < 0xFFFF {
        buffer.push(0xFD);
        (sz as u16).encode(buffer);
    }
    else if sz <0xFFFF_FFFF {
        buffer.push(0xFE);
        (sz as u32).encode(buffer);
    }
    else {
        buffer.push(0xFF);
        (sz as u64).encode(buffer);
    }
}

impl<'a> NetworkEncoding<'a> for &'a [u8] {
    fn decode(buffer: &mut Buffer<'a>) -> Result<&'a [u8], EndOfBufferError> {
        let count = buffer.decode_compact_size()?;
        if buffer.inner.len() < count {
            return Err(EndOfBufferError);
        }

        let result = &buffer.inner[..count];
        buffer.inner = &buffer.inner[count..];
        Ok(result)
    }

    fn encode(&self, buffer: &mut Vec<u8>) {
        encode_compact_size(buffer, self.len());
        buffer.extend_from_slice(self);
    }
}



impl<'a, T : NetworkEncoding<'a>> NetworkEncoding<'a> for Vec<T> {

    /// Parses a compact-size prefix vector of parsable stuff
    fn decode(buffer: &mut Buffer<'a>) -> Result<Vec<T>, EndOfBufferError> {

        let count = try!(buffer.decode_compact_size());
        let mut result: Vec<T> = Vec::with_capacity(count);
        for _ in 0..count {
            result.push(try!(T::decode(buffer)));
        }
        Ok(result)
    }

    fn encode(&self, buffer: &mut Vec<u8>) {
        encode_compact_size(buffer, self.len());
        for itm in self.into_iter() {
            itm.encode(buffer);
        }
    }
}


macro_rules! impl_decode_primitive {
    ($prim_type: ty) =>

    (
        impl<'a> NetworkEncoding<'a> for $prim_type {
            fn decode(buffer: &mut Buffer<'a>) -> Result<$prim_type, EndOfBufferError> {
                let sz = mem::size_of::<$prim_type>();
                if buffer.inner.len() < sz {
                    return Err(EndOfBufferError);
                }

                // Shift-n-fold
                let result = (0..sz)
                    .map(|n| (buffer.inner[n] as $prim_type) << (8* n) )
                    .fold(0, |a,b| a | b);

                buffer.inner = &buffer.inner[sz..];

                Ok(result)
            }

            fn encode(&self, buffer: &mut Vec<u8>) {
                let sz = mem::size_of::<$prim_type>();
                for n in 0..sz {
                    buffer.push((self >> (n*8)) as u8);
                }
            }
        }
    )
}

impl_decode_primitive!(u32);
impl_decode_primitive!(i64);
impl_decode_primitive!(i32);
impl_decode_primitive!(u8);
impl_decode_primitive!(u16);
impl_decode_primitive!(u64);



#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_primitive() {
        let x = &[0xff_u8, 0x00_u8, 0x00_u8, 0x00_u8];
        let mut buf = Buffer { inner: x };
        let org_buf = buf;

        assert_eq!(u32::decode(&mut buf).unwrap(), 0xff_u32);

        assert_eq!(buf.len(), 0);
        assert_eq!(buf.consumed_since(org_buf).len(), 4);
    }
}


