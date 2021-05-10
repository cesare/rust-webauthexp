use rand::{RngCore, SeedableRng, rngs::StdRng};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

struct RandomString<const N: usize> {}

impl<const N: usize> RandomString<N> {
    fn generate() -> String {
        let mut rng = StdRng::from_entropy();
        let mut rs: [u8; N] = [0; N];
        rng.fill_bytes(&mut rs);
        base64::encode_config(rs, base64::URL_SAFE_NO_PAD)
    }
}

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
        RandomString::<32>::generate()
    }

    fn generate_challenge(verifier: &str) -> String {
        let digest = Sha256::digest(verifier.as_bytes());
        base64::encode_config(digest, base64::URL_SAFE_NO_PAD)
    }
}
