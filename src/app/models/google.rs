use std::time::{SystemTime, SystemTimeError};

use anyhow::Result;
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use serde_derive::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

use crate::app::config::GoogleConfig;
use crate::app::models::oidc::discovery::{DiscoveryError, JsonWebKey, OpenIdConfiguration, OpenIdConfigurationDiscovery};
use crate::app::models::random::{RandomString, RandomStringGenerator};

pub struct GoogleAutorization<'a> {
    config: &'a GoogleConfig,
}

impl<'a> GoogleAutorization<'a> {
    pub fn new(config: &'a GoogleConfig) -> Self {
        Self {
            config,
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
            request_uri: url.into(),
            attributes: RequestAttributes {
                state,
                nonce,
            }
        };
        Ok(request)
    }

    fn generate_state(&self) -> String {
        RandomString::new().generate(32)
    }

    fn generate_nonce(&self) -> String {
        RandomString::new().generate(32)
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

#[derive(Debug, Error)]
pub enum GoogleSigninError {
    #[error("no saved request attributes")]
    RequestAttributesMissing,

    #[error("state mismatch")]
    StateMismatch,

    #[error("nonce mismatch")]
    NonceMismatch,

    #[error("invalid issuer on ID token")]
    InvalidIssuer,

    #[error("ID token already expired")]
    IdTokenExpired,

    #[error("Failed to get duration for current time")]
    InvalidCurrentTime(#[from] SystemTimeError),

    #[error("JWT error")]
    JwtError(#[from] jsonwebtoken::errors::Error),

    #[error("Discovery failed")]
    DiscoveryFailed(#[from] DiscoveryError),

    #[error(transparent)]
    OtherError(#[from] anyhow::Error),
}

pub struct GoogleSignin<'a> {
    config: &'a GoogleConfig,
    auth: &'a GoogleAuthorizationResponse,
    attributes: Option<RequestAttributes>,
}

impl<'a> GoogleSignin<'a> {
    pub fn new(config: &'a GoogleConfig, auth: &'a GoogleAuthorizationResponse, attributes: Option<RequestAttributes>) -> Self {
        Self {
            config,
            auth,
            attributes,
        }
    }

    pub async fn execute(&self) -> Result<GoogleId, GoogleSigninError> {
        let attrs = self.attributes.as_ref().ok_or(GoogleSigninError::RequestAttributesMissing)?;
        if attrs.state != self.auth.state {
            return Err(GoogleSigninError::StateMismatch)
        }

        let openid_config = OpenIdConfigurationDiscovery::new(&self.config.issuer()).execute().await?;
        let token_response = TokenRequest::new(self.config, &openid_config, &self.auth.code).execute().await?;

        let id_token = token_response.id_token;
        let header = jsonwebtoken::decode_header(&id_token)?;
        let jwk = self.find_jwk(&header.kid.unwrap(), &openid_config).await?;

        let decoding_key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e);
        let validation = Validation::new(Algorithm::RS256);
        let claims = jsonwebtoken::decode::<Claims>(&id_token, &decoding_key, &validation)?.claims;
        self.validate_claims(&claims, attrs)?;
        Ok(claims.into())
    }

    async fn find_jwk(&self, kid: &str, openid_config: &OpenIdConfiguration) -> Result<JsonWebKey> {
        let jwks = openid_config.find_jwks().await?;
        let jwk = jwks.find_by_kid(kid).unwrap();
        Ok(jwk.clone())
    }

    fn validate_claims(&self, claims: &Claims, attrs: &RequestAttributes) -> Result<(), GoogleSigninError> {
        if claims.nonce != attrs.nonce {
            return Err(GoogleSigninError::NonceMismatch)
        }
        if claims.iss != self.config.issuer() {
            return Err(GoogleSigninError::InvalidIssuer)
        }

        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).map(|d| d.as_secs())?;
        if claims.exp < now {
            return Err(GoogleSigninError::IdTokenExpired)
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Claims {
    pub aud: String,
    pub exp: u64,
    pub iss: String,
    pub sub: String,
    pub email: String,
    pub nonce: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GoogleId {
    pub sub: String,
    pub email: String,
}

impl From<Claims> for GoogleId {
    fn from(claims: Claims) -> Self {
        Self {
            sub: claims.sub,
            email: claims.email,
        }
    }
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
    id_token: String,
    scope: String,
    token_type: String,
}

struct TokenRequest<'a> {
    config: &'a GoogleConfig,
    openid_config: &'a OpenIdConfiguration,
    code: &'a str,
}

impl<'a> TokenRequest<'a> {
    fn new(config: &'a GoogleConfig, openid_config: &'a OpenIdConfiguration, code: &'a str) -> Self {
        Self {
            config,
            openid_config,
            code,
        }
    }

    async fn execute(&self) -> Result<TokenResponse> {
        let config = self.config;
        let client = reqwest::Client::new();
        let parameters = [
            ("grant_type", "authorization_code"),
            ("client_id", &config.client_id),
            ("client_secret", &config.client_secret),
            ("redirect_uri", &config.redirect_uri),
            ("code", self.code),
        ];
        let response = client.post(&self.openid_config.token_endpoint)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&parameters)
            .send()
            .await?;
        let result = response.json::<TokenResponse>().await?;
        Ok(result)
    }
}
