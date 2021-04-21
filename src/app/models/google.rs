use anyhow::Result;
use rand::{RngCore, SeedableRng, rngs::StdRng};
use serde_derive::{Deserialize, Serialize};
use url::Url;

use crate::app::config::GoogleConfig;

pub struct GoogleAutorization<'a> {
    config: &'a GoogleConfig,
}

impl<'a> GoogleAutorization<'a> {
    pub fn new(config: &'a GoogleConfig) -> Self {
        Self {
            config: config,
        }
    }

    pub fn start(&self) -> Result<GoogleAuthRequest> {
        let config = self.config;
        let state = self.generate_state();
        let nonce = self.generate_nonce();
        let base = "https://accounts.google.com/o/oauth2/v2/auth";
        let parameters = vec![
            ("response_type", "code"),
            ("client_id", &config.client_id),
            ("redirect_uri", &config.redirect_uri),
            ("scope", &config.scope),
            ("state", &state),
            ("nonce", &nonce),
        ];
        let url = Url::parse_with_params(base, &parameters)?;
        let request = GoogleAuthRequest {
            request_uri: url.into_string(),
            attributes: RequestAttributes {
                state: state,
                nonce: nonce,
            }
        };
        Ok(request)
    }

    fn generate_state(&self) -> String {
        let mut rng = StdRng::from_entropy();
        let mut rs: [u8; 32] = [0; 32];
        rng.fill_bytes(&mut rs);
        base64::encode_config(rs, base64::URL_SAFE)
    }

    fn generate_nonce(&self) -> String {
        let mut rng = StdRng::from_entropy();
        let mut rs: [u8; 32] = [0; 32];
        rng.fill_bytes(&mut rs);
        base64::encode_config(rs, base64::URL_SAFE)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RequestAttributes {
    pub state: String,
    pub nonce: String,
}

pub struct GoogleAuthRequest {
    pub request_uri: String,
    pub attributes: RequestAttributes,
}

#[derive(Debug, Deserialize)]
pub struct GoogleAuthorizationResponse {
    pub state: String,
    pub code: String,
    pub scope: String,
}
