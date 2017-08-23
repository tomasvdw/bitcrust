use rand::{self, Rng};
use ring::hmac;

use byteorder::{LittleEndian, WriteBytesExt};

#[derive(Debug, PartialEq)]
pub struct AuthenticatedBitcrustMessage {
    signature: [u8; 32],
    nonce: [u8; 8],
}

impl AuthenticatedBitcrustMessage {
    pub fn create(key: &hmac::SigningKey) ->
     AuthenticatedBitcrustMessage {
        let mut rng = rand::thread_rng();

        let nonce: u64 = rng.gen();
        
        let mut nonce_vec = Vec::with_capacity(8);
        let _ = nonce_vec.write_u64::<LittleEndian>(nonce);
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
        hmac::verify_with_own_key(key, &self.nonce, &self.signature).is_ok()
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut v = Vec::with_capacity(40);

        for byte in &self.nonce {
            let _ = v.write_u8(*byte);
        }
        for byte in &self.signature {
            let _ = v.write_u8(*byte);
        }
        v
    }
}