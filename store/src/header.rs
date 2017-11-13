
use network_encoding::*;
use hash::*;

/// Header represents the header of a block
#[derive(Debug, Clone)]
pub struct Header {

    // db and network
    pub version:     u32,
    pub prev_hash:   Hash,
    pub merkle_root: Hash,
    pub time:        u32,
    pub bits:        u32,
    pub nonce:       u32,
}




impl<'a> NetworkEncoding<'a> for Header {

    /// Parses the block-header
    fn decode(buffer: &mut Buffer) -> Result<Header, EndOfBufferError> {

        Ok(Header {
            version:     u32::decode(buffer)?,
            prev_hash:   Hash::decode(buffer)?,
            merkle_root: Hash::decode(buffer)?,
            time:        u32::decode(buffer)?,
            bits:        u32::decode(buffer)?,
            nonce:       u32::decode(buffer)?,
        })
    }

    fn encode(&self, buffer: &mut Vec<u8>) {
        self.version.encode(buffer);
        self.prev_hash.encode(buffer);
        self.merkle_root.encode(buffer);
        self.time.encode(buffer);

    }


}

impl Header {
    pub fn new(raw_header: &[u8]) -> Result<Header, EndOfBufferError> {
        let mut b = Buffer::new(raw_header);
        Header::decode(&mut b)
    }
}
