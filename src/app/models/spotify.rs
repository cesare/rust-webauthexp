use serde_derive::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

use crate::app::{config::SpotifyConfig, models::pkce::Pkce};

use super::random::RandomString;

#[derive(Debug, Error)]
pub enum SpotifySigninError {
    #[error("missing request attributes")]
    RequestAttributesMissing,

    #[error("state mismatch")]
    StateMismatch,

    #[error("token request failed")]
    TokenRequestFailed(#[from] reqwest::Error),

    #[error("invalid url")]
    InvalidUrl(#[from] url::ParseError),
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
        let uri = Url::parse_with_params(base, &parameters)?;

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

pub struct SpotifySignin<'a> {
    config: &'a SpotifyConfig,
    response: &'a AuthResponse,
    attributes: &'a Option<RequestAttributes>,
}

impl<'a> SpotifySignin<'a> {
    pub fn new(config: &'a SpotifyConfig, response: &'a AuthResponse, attributes: &'a Option<RequestAttributes>) -> Self {
        Self {
            config: config,
            response: response,
            attributes: attributes,
        }
    }

    pub async fn execute(&self) -> Result<SigninResult> {
        let attrs = self.attributes.as_ref().ok_or(SpotifySigninError::RequestAttributesMissing)?;
        if self.response.state != attrs.state {
            return Err(SpotifySigninError::StateMismatch)
        }

        let token = TokenRequest::new(self.config, &self.response.code, &attrs.code_verifier).execute().await?;
        let user = UserRequest::new(&token.access_token).execute().await?;
        let result = SigninResult {
            id: user.id,
            display_name: user.display_name,
            email: user.email,
            access_token: token.access_token,
        };
        Ok(result)
    }
}

#[derive(Debug, Serialize)]
pub struct SigninResult {
    id: String,
    display_name: String,
    email: String,
    access_token: String,
}

struct TokenRequest<'a> {
    config: &'a SpotifyConfig,
    code: &'a str,
    code_verifier: &'a str,
}

impl<'a> TokenRequest<'a> {
    fn new(config: &'a SpotifyConfig, code: &'a str, code_verifier: &'a str) -> Self {
        Self {
            config: config,
            code: code,
            code_verifier: code_verifier,
        }
    }

    async fn execute(&self) -> Result<AccessToken> {
        let config = self.config;

        let client = reqwest::Client::new();
        let parameters = [
            ("grant_type", "authorization_code"),
            ("client_id", &config.client_id),
            ("code", self.code),
            ("redirect_uri", &config.redirect_uri),
            ("code_verifier", self.code_verifier),
        ];
        let result = client.post("https://accounts.spotify.com/api/token")
            .header("Accept", "application/json")
            .form(&parameters)
            .send()
            .await?
            .json::<AccessToken>().await?;

        Ok(result)
    }
}

#[derive(Debug, Deserialize)]
struct AccessToken {
    access_token: String,
    token_type: String,
    scope: String,
    expires_in: u64,
    refresh_token: String,
}

struct UserRequest<'a> {
    access_token: &'a str,
}

impl<'a> UserRequest<'a> {
    fn new(access_token: &'a str) -> Self {
        Self {
            access_token: access_token,
        }
    }

    async fn execute(&self) -> Result<User> {
        let client = reqwest::Client::new();
        let result = client.get("https://api.spotify.com/v1/me")
            .header("Accept", "application/json")
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await?
            .json::<User>().await?;

        Ok(result)
    }
}

#[derive(Debug, Deserialize)]
struct User {
    id: String,
    display_name: String,
    email: String,
}
