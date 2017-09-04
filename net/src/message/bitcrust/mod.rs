use std::io;

use rand::{self, Rng};
use ring::hmac;

use Encode;

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
        hmac::verify_with_own_key(key, &self.nonce, &self.signature).is_ok()
    }

    pub fn len(&self) -> usize {
        40
    }
}

impl Encode for AuthenticatedBitcrustMessage {
    fn encode(&self, mut buff: &mut Vec<u8>) -> Result<(), io::Error> {
        // let mut v = Vec::with_capacity(40);

        let _ = self.nonce.encode(&mut buff);
        let _ = self.signature.encode(&mut buff);
        Ok(())
    }
}