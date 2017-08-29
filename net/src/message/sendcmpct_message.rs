use Encode;

#[derive(Debug, Encode, PartialEq)]
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
