use anyhow::Result;
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use rand::{RngCore, SeedableRng, rngs::StdRng};
use serde_derive::{Deserialize, Serialize};
use url::Url;

use crate::app::config::GoogleConfig;
use crate::app::models::oidc::discovery::{JsonWebKey, OpenIdConfiguration, OpenIdConfigurationDiscovery};

struct RandomString<const N: usize> {}

impl<const N: usize> RandomString<N> {
    fn generate() -> String {
        let mut rng = StdRng::from_entropy();
        let mut rs: [u8; N] = [0; N];
        rng.fill_bytes(&mut rs);
        base64::encode_config(rs, base64::URL_SAFE)
    }
}

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
        RandomString::<32>::generate()
    }

    fn generate_nonce(&self) -> String {
        RandomString::<32>::generate()
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

pub struct GoogleSignin<'a> {
    config: &'a GoogleConfig,
}

impl<'a> GoogleSignin<'a> {
    pub fn new(config: &'a GoogleConfig) -> Self {
        Self {
            config: config,
        }
    }

    pub async fn execute(&self, auth: &GoogleAuthorizationResponse, attributes: Option<RequestAttributes>) -> Result<GoogleId> {
        let openid_config = OpenIdConfigurationDiscovery::new("https://accounts.google.com").execute().await?;
        let token_response = TokenRequest::new(&self.config, &auth.code).execute().await?;

        let id_token = token_response.id_token;
        let header = jsonwebtoken::decode_header(&id_token)?;
        let jwk = self.find_jwk(&header.kid.unwrap(), &openid_config).await?;

        let decoding_key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e);
        let validation = Validation::new(Algorithm::RS256);
        let google_id = jsonwebtoken::decode::<GoogleId>(&id_token, &decoding_key, &validation)?;
        Ok(google_id.claims)
    }

    async fn find_jwk(&self, kid: &str, openid_config: &OpenIdConfiguration) -> Result<JsonWebKey> {
        let jwks = openid_config.find_jwks().await?;
        let jwk = jwks.find_by_kid(&kid).unwrap();
        Ok(jwk.clone())
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GoogleId {
    pub sub: String,
    pub email: Option<String>,
    pub name: Option<String>,
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
    code: &'a str,
}

impl<'a> TokenRequest<'a> {
    fn new(config: &'a GoogleConfig, code: &'a str) -> Self {
        Self {
            config: config,
            code: code,
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
        let response = client.post("https://oauth2.googleapis.com/token") // TODO: use discovery
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&parameters)
            .send()
            .await?;
        let result = response.json::<TokenResponse>().await?;
        Ok(result)
    }
}
