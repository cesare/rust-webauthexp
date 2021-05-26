use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::app::models::random::RandomString;

#[derive(Debug, Deserialize, Serialize)]
pub struct Pkce {
    pub code_verifier: String,
    pub code_challenge: String,
}

impl Pkce {
    pub fn new() -> Self {
        let verifier = Self::generate_verifier();
        let challenge = Self::generate_challenge(&verifier);

        Self {
            code_verifier: verifier,
            code_challenge: challenge,
        }
    }

    fn generate_verifier() -> String {
        RandomString::<32>::new().generate()
    }

    fn generate_challenge(verifier: &str) -> String {
        let digest = Sha256::digest(verifier.as_bytes());
        base64::encode_config(digest, base64::URL_SAFE_NO_PAD)
    }
}
