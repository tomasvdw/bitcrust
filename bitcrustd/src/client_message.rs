use bitcrust_net::NetAddr;

#[derive(Clone, Debug)]
pub enum ClientMessage {
    Addrs(Vec<NetAddr>),
    PeersConnected(u64),
    /// Expects a hostname argument
    Closing(String),
}
