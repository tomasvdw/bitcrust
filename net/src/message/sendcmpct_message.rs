use std::io;

use Encode;

#[derive(Debug, PartialEq)]
pub struct SendCmpctMessage {
    pub send_compact: bool,
    pub version: u64,
}

impl SendCmpctMessage {
    #[inline]
    pub fn len(&self) -> usize {
        9
    }

    #[inline]
    pub fn name(&self) -> &'static str {
        "sendcmpct"
    }
}

impl Encode for SendCmpctMessage {
    fn encode(&self, mut buff: &mut Vec<u8>) -> Result<(), io::Error> {
        // let mut v = Vec::with_capacity(self.len());
        if self.send_compact {
            buff.push(1);
        } else {
            buff.push(0);
        }
        let _ = self.version.encode(&mut buff);
        Ok(())
    }
}