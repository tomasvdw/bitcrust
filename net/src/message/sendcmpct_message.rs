use Encode;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_implements_types_required_for_protocol() {
        let m =  SendCmpctMessage::default();
        assert_eq!(m.name(), "sendcmpct");
        assert_eq!(m.len(), 9);
    }
}
///
/// https://github.com/bitcoin/bips/blob/master/bip-0152.mediawiki
///
/// Setting the send_compact field to 1 enables the high-bandwidth
/// mode specified in the above bip.

#[derive(Debug, Default, Encode, PartialEq)]
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
