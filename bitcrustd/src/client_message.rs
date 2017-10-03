use bitcrust_net::NetAddr;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_is_clonable_with_peers() {
        let input = ClientMessage::PeersConnected(12);
        let output = input.clone();

        assert_eq!(input, output);
    }

        #[test]
    fn it_is_clonable_with_addrs() {
        let input = ClientMessage::Addrs(vec![]);
        let output = input.clone();

        assert_eq!(input, output);
    }

        #[test]
    fn it_is_clonable_with_closing() {
        let input = ClientMessage::Closing("localhost:8192".into());
        let output = input.clone();

        assert_eq!(input, output);
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ClientMessage {
    Addrs(Vec<NetAddr>),
    PeersConnected(u64),
    /// Expects a hostname argument
    Closing(String),
}
