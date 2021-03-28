use rand::{self, Rng};
use ring::hmac;

use Encode;

#[cfg(test)]
mod tests {
    use ring::digest;
    use super::*;

    #[test]
    fn it_implements_types_required_for_protocol() {
        let m =  AuthenticatedBitcrustMessage::default();
        assert_eq!(m.name(), "bcr_pcr");
        assert_eq!(m.len(), 40);
    }

    #[test]
    fn it_creates_and_validates() {
        let key = hmac::SigningKey::new(&digest::SHA256, &[0x00; 32]);
        let m =  AuthenticatedBitcrustMessage::create(&key);
        assert!(m.valid(&key));
    }
}

#[derive(Debug, Default, Encode, PartialEq)]
pub struct AuthenticatedBitcrustMessage {
    nonce: [u8; 8],
    signature: [u8; 32],
}

impl AuthenticatedBitcrustMessage {
    pub fn create(key: &hmac::SigningKey) ->
     AuthenticatedBitcrustMessage {
        let mut rng = rand::thread_rng();

        let nonce: u64 = rng.gen();
        
        let mut nonce_vec = Vec::with_capacity(8);
        let _ = nonce.encode(&mut nonce_vec);
        let signature = hmac::sign(&key, &nonce_vec);
        AuthenticatedBitcrustMessage::with_signature(signature.as_ref(), &nonce_vec)
    }

    pub fn with_signature(input: &[u8], nonce: &[u8]) -> AuthenticatedBitcrustMessage{
        let mut a: [u8; 32] = [0; 32];
        a.copy_from_slice(&input);
        let mut b: [u8; 8] = [0; 8];
        b.copy_from_slice(&nonce);
        AuthenticatedBitcrustMessage {
            nonce: b,
            signature: a
        }
    }
    pub fn valid(&self, key: &hmac::SigningKey) -> bool {
        hmac::verify(key, &self.nonce, &self.signature).is_ok()
    }

    pub fn len(&self) -> usize {
        40
    }

    #[inline]
    pub fn name(&self) -> &'static str {
        "bcr_pcr"
    }
}
