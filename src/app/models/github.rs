use anyhow::Result;
use rand::{RngCore, SeedableRng, rngs::StdRng};
use serde_derive::Deserialize;
use thiserror::Error;
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

#[derive(Debug, Error)]
pub enum GithubSigninError {
    #[error("no saved state found")]
    StateNotFound,

    #[error("states do not match")]
    StateMismatch,

    #[error(transparent)]
    RequestFailed(#[from] reqwest::Error),

    #[error("not implemented yet")]
    NotImplemented,
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

    pub async fn execute(&self, auth: &GithubAuthorizationResponse, saved_state: Option<String>) -> Result<GithubUser, GithubSigninError> {
        let state = saved_state.ok_or(GithubSigninError::StateNotFound)?;
        if state != auth.state {
            return Err(GithubSigninError::StateMismatch)
        }

        let token_request = AccessTokenRequest {};
        let token_response = token_request.execute(&self.config, &auth.code, &state).await?;

        let user_request = GithubUserRequest {};
        user_request.execute(&self.config, &token_response.access_token)
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
    async fn execute(&self, config: &GithubConfig, code: &String, state: &String) -> Result<AccessTokenResponse, GithubSigninError> {
        let client = reqwest::Client::new();
        let parameters = [
            ("client_id", &config.client_id),
            ("client_secret", &config.client_secret),
            ("code", code),
            ("redirect_uri", &config.redirect_uri),
            ("state", state),
        ];
        let result = client.post("https://github.com/login/oauth/access_token")
            .header("Accept", "application/json")
            .form(&parameters)
            .send()
            .await?
            .json::<AccessTokenResponse>().await?;

        Ok(result)
    }
}

struct GithubUserRequest {
}

impl GithubUserRequest {
    fn execute(&self, _config: &GithubConfig, _access_token: &str) -> Result<GithubUser, GithubSigninError> {
        Err(GithubSigninError::NotImplemented)
    }
}

#[derive(Debug, Deserialize)]
pub struct GithubUser {
    pub id: String,
    pub login: String,
    pub name: String,
}
