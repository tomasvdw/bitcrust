
use hash::*;

use serde_network;

/// Header represents the header of a block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {

    // db and network
    pub version:     u32,
    pub prev_hash:   Hash,
    pub merkle_root: Hash,
    pub time:        u32,
    pub bits:        u32,
    pub nonce:       u32,
}


impl Header {
    pub fn new(raw_header: &[u8]) -> Result<Header, serde_network::Error> {
        serde_network::deserialize(raw_header)
    }

}


