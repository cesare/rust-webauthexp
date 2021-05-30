use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::app::models::random::{RandomString, RandomStringGenerator};

#[derive(Debug, Deserialize, Serialize)]
pub struct Pkce {
    pub code_verifier: String,
    pub code_challenge: String,
}

pub struct PkceGenerator {
    rsg: Box<dyn RandomStringGenerator>,
}

impl Default for PkceGenerator {
    fn default() -> Self {
        Self {
            rsg: Box::new(RandomString::new()),
        }
    }
}

impl PkceGenerator {
    pub fn generate(&self, size: usize) -> Pkce {
        let verifier = self.generate_verifier(size);
        let challenge = self.generate_challenge(&verifier);

        Pkce {
            code_verifier: verifier,
            code_challenge: challenge,
        }
    }

    fn generate_verifier(&self, size: usize) -> String {
        self.rsg.generate(size)
    }

    fn generate_challenge(&self, verifier: &str) -> String {
        let digest = Sha256::digest(verifier.as_bytes());
        base64::encode_config(digest, base64::URL_SAFE_NO_PAD)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::models::random::RandomStringGenerator;

    struct DummyRSG {}
    impl RandomStringGenerator for DummyRSG {
        fn generate(&self, size: usize) -> String {
            let bytes: Vec<u8> = (0..size).map(|n| n as u8).collect();
            base64::encode_config(bytes, base64::URL_SAFE_NO_PAD)
        }
    }

    #[test]
    fn test_pkce_generator() {
        let generator = PkceGenerator {
            rsg: Box::new(DummyRSG {}),
        };
        let pkce = generator.generate(16);
        assert_eq!("AAECAwQFBgcICQoLDA0ODw", pkce.code_verifier);
        assert_eq!("XSYR1aTdK0Cyd9_bXs9bSYGRUNfVxi9O75YMsMJNsgw", pkce.code_challenge);
    }
}
