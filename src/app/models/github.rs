use anyhow::{anyhow, Result};
use rand::{RngCore, SeedableRng, rngs::StdRng};
use serde_derive::Deserialize;
use url::Url;

use crate::app::config::GithubConfig;

pub struct GithubAutorizationRequest {
    client_id: String,
    redirect_uri: String,
    scope: String,
    pub state: String,
}

impl GithubAutorizationRequest {
    pub fn new(config: &GithubConfig) -> Self {
        let mut rng = StdRng::from_entropy();
        let mut rs: [u8; 32] = [0; 32];
        rng.fill_bytes(&mut rs);
        let state = base64::encode_config(rs, base64::URL_SAFE);

        Self {
            client_id: config.client_id.to_owned(),
            redirect_uri: config.redirect_uri.to_owned(),
            scope: config.scope.to_owned(),
            state: state,
        }
    }

    pub fn request_uri(&self) -> Result<String> {
        let base = "https://github.com/login/oauth/authorize";
        let parameters = vec![
            ("client_id", &self.client_id),
            ("redirect_uri", &self.redirect_uri),
            ("scope", &self.scope),
            ("state", &self.state),
        ];
        let url = Url::parse_with_params(base, &parameters)?;
        Ok(url.into_string())
    }
}

#[derive(Debug, Deserialize)]
pub struct GithubAuthorizationResponse {
    pub code: String,
    pub state: String,
}

pub struct GithubSignin {
    config: GithubConfig,
}

impl GithubSignin {
    pub fn new(config: &GithubConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }
}

struct AccessTokenRequest {
}

#[derive(Deserialize)]
struct AccessTokenResponse {
    pub access_token: String,
    pub scope: String,
    pub token_type: String,
}

impl AccessTokenRequest {
    fn execute(&self, _config: &GithubConfig, _code: &str, _state: &str) -> Result<AccessTokenResponse> {
        Err(anyhow!("not implemented yet"))
    }
}

#[derive(Debug, Deserialize)]
pub struct GithubUser {
    pub id: String,
    pub login: String,
    pub name: String,
}
