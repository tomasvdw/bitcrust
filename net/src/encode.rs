pub trait Encode {
    fn encode(&Self) -> Vec<u8>;
}