use byteorder::{LittleEndian, WriteBytesExt};

/// sendcmpct message
/// The sendcmpct message is defined as a message containing a 1-byte integer followed by a 8-byte integer where pchCommand == "sendcmpct".
/// The first integer SHALL be interpreted as a boolean (and MUST have a value of either 1 or 0)
/// The second integer SHALL be interpreted as a little-endian version number. Nodes sending a sendcmpct message MUST currently set this value to 1.
/// Upon receipt of a "sendcmpct" message with the first and second integers set to 1, the node SHOULD announce new blocks by sending a cmpctblock message.
/// Upon receipt of a "sendcmpct" message with the first integer set to 0, the node SHOULD NOT announce new blocks by sending a cmpctblock message, but SHOULD announce new blocks by sending invs or headers, as defined by BIP130.
/// Upon receipt of a "sendcmpct" message with the second integer set to something other than 1, nodes MUST treat the peer as if they had not received the message (as it indicates the peer will provide an unexpected encoding in
/// cmpctblock, and/or other, messages). This allows future versions to send duplicate sendcmpct messages with different versions as a part of a version handshake for future versions.
/// Nodes SHOULD check for a protocol version of >= 70014 before sending sendcmpct messages.
/// Nodes MUST NOT send a request for a MSG_CMPCT_BLOCK object to a peer before having received a sendcmpct message from that peer.
/// This message is only supported by protocol version >= 70014
#[derive(Debug, PartialEq)]
pub struct SendCmpctMessage {
    pub send_compact: bool,
    pub version: u64,
}

impl SendCmpctMessage {
    pub fn encode(&self) -> Vec<u8> {
        let mut v = Vec::with_capacity(8);
        if self.send_compact {
            v.push(1);
        } else {
            v.push(0);
        }
        v.write_u64::<LittleEndian>(self.version);
        v
    }
}
