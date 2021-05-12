use serde_derive::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

use crate::app::{config::SpotifyConfig, models::pkce::Pkce};

use super::random::RandomString;

#[derive(Debug, Error)]
pub enum SpotifySigninError {
}

type Result<T> = std::result::Result<T, SpotifySigninError>;

pub struct SpotifyAuthorization<'a> {
    config: &'a SpotifyConfig,
}

impl<'a> SpotifyAuthorization<'a> {
    pub fn new(config: &'a SpotifyConfig) -> Self {
        Self {
            config: config,
        }
    }

    pub fn start(&self) -> Result<AuthRequest> {
        let config = self.config;

        let pkce = Pkce::new();
        let state = self.generate_state();

        let base = "https://accounts.spotify.com/authorize";
        let parameters = vec![
            ("response_type", "code"),
            ("client_id", &config.client_id),
            ("redirect_uri", &config.redirect_uri),
            ("scope", &config.scope),
            ("state", &state),
            ("code_challenge_method", "S256"),
            ("code_challenge", &pkce.code_challenge),
        ];
        let uri = Url::parse_with_params(base, &parameters).unwrap();

        let request = AuthRequest {
            request_uri: uri.into(),
            attributes: RequestAttributes {
                state: state,
                code_verifier: pkce.code_verifier,
            },
        };
        Ok(request)
    }

    fn generate_state(&self) -> String {
        RandomString::<32>::generate()
    }
}

pub struct AuthRequest {
    pub request_uri: String,
    pub attributes: RequestAttributes,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RequestAttributes {
    pub state: String,
    pub code_verifier: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthResponse {
    pub state: String,
    pub code: String,
}
