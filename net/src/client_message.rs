use net_addr::NetAddr;

#[derive(Clone, Debug)]
pub enum ClientMessage {
    Addrs(Vec<NetAddr>),
    /// Expects a hostname argument
    Closing(String),
}
