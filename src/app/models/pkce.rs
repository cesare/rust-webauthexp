use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::app::models::random::{RandomString, RandomStringGenerator};

#[derive(Debug, Deserialize, Serialize)]
pub struct Pkce {
    pub code_verifier: String,
    pub code_challenge: String,
}

pub struct PkceGenerator<const N: usize> {
    rsg: Box<dyn RandomStringGenerator>,
}

impl<const N: usize> Default for PkceGenerator<N> {
    fn default() -> Self {
        Self {
            rsg: Box::new(RandomString::new()),
        }
    }
}

impl<const N: usize> PkceGenerator<N> {
    pub fn generate(&self) -> Pkce {
        let verifier = self.generate_verifier();
        let challenge = self.generate_challenge(&verifier);

        Pkce {
            code_verifier: verifier,
            code_challenge: challenge,
        }
    }

    fn generate_verifier(&self) -> String {
        self.rsg.generate(32)
    }

    fn generate_challenge(&self, verifier: &str) -> String {
        let digest = Sha256::digest(verifier.as_bytes());
        base64::encode_config(digest, base64::URL_SAFE_NO_PAD)
    }
}
