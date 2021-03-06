use actix_web::{HttpResponse, ResponseError};
use anyhow::Result;
use rand::{RngCore, SeedableRng, rngs::StdRng};
use serde_derive::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

use crate::app::config::GithubConfig;

pub struct GithubAutorizationRequest<'a> {
    config: &'a GithubConfig,
}

impl<'a> GithubAutorizationRequest<'a> {
    pub fn new(config: &'a GithubConfig) -> Self {
        Self {
            config,
        }
    }

    pub fn create(&self) -> Result<(String, String)> {
        let config = self.config;
        let state = self.generate_state();
        let base = "https://github.com/login/oauth/authorize";
        let parameters = vec![
            ("client_id", &config.client_id),
            ("redirect_uri", &config.redirect_uri),
            ("scope", &config.scope),
            ("state", &state),
        ];
        let url = Url::parse_with_params(base, &parameters)?;
        Ok((url.into(), state))
    }

    fn generate_state(&self) -> String {
        let mut rng = StdRng::from_entropy();
        let mut rs: [u8; 32] = [0; 32];
        rng.fill_bytes(&mut rs);
        base64::encode_config(rs, base64::URL_SAFE)
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

impl ResponseError for GithubSigninError {
    fn status_code(&self) -> reqwest::StatusCode {
        reqwest::StatusCode::INTERNAL_SERVER_ERROR
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::InternalServerError().finish()
    }
}

pub struct GithubSignin<'a> {
    config: &'a GithubConfig,
}

impl<'a> GithubSignin<'a> {
    pub fn new(config: &'a GithubConfig) -> Self {
        Self {
            config,
        }
    }

    pub async fn execute(&self, auth: &GithubAuthorizationResponse, saved_state: Option<String>) -> Result<GithubUser, GithubSigninError> {
        let state = saved_state.ok_or(GithubSigninError::StateNotFound)?;
        if state != auth.state {
            return Err(GithubSigninError::StateMismatch)
        }

        let access_token = AccessTokenRequest::new(self.config)
            .execute(&auth.code, &state)
            .await?;

        UserRequest::new()
            .execute(&access_token.token)
            .await
    }
}

struct AccessTokenRequest<'a> {
    config: &'a GithubConfig,
}

#[derive(Deserialize)]
struct AccessToken {
    #[serde(rename(deserialize = "access_token"))]
    token: String,
}

impl<'a> AccessTokenRequest<'a> {
    fn new(config: &'a GithubConfig) -> Self {
        Self {
            config,
        }
    }

    async fn execute(&self, code: &String, state: &String) -> Result<AccessToken, GithubSigninError> {
        let config = self.config;
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
            .json::<AccessToken>().await?;

        Ok(result)
    }
}

struct UserRequest {
}

impl UserRequest {
    fn new() -> Self {
        Self {}
    }

    async fn execute(&self, access_token: &str) -> Result<GithubUser, GithubSigninError> {
        let client = reqwest::Client::new();
        let response = client.get("https://api.github.com/user")
            .header("Accept", "application/vnd.github.v3+json")
            .header("Authorization", format!("token {}", access_token))
            .header("User-Agent", "Webauthexp")
            .send()
            .await?;

        if response.status().is_success() {
            let result = response.json::<GithubUser>()
                .await?;
            Ok(result)
        } else {
            let _body = response.text().await?;
            Err(GithubSigninError::NotImplemented)
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GithubUser {
    pub id: u64,
    pub login: String,
    pub name: String,
}
