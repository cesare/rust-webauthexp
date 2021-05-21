use serde_derive::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DiscoveryError {
    #[error("Decoding response body for {uri} failed: {message}")]
    DecodingFailed { uri: String, message: String },

    #[error("HTTP request for {uri} failed: {message}")]
    HttpRequestFailed { uri: String, message: String }
}

impl From<reqwest::Error> for DiscoveryError {
    fn from(error: reqwest::Error) -> Self {
        let message = error.to_string();
        let uri = error.url()
            .map(|url| url.as_str())
            .unwrap_or("-")
            .to_owned();

        if error.is_decode() {
            return Self::DecodingFailed { uri: uri, message: message }
        }
        Self::HttpRequestFailed { uri: uri, message: message }
    }
}

type Result<T> = std::result::Result<T, DiscoveryError>;

#[derive(Debug, Deserialize, Serialize)]
pub struct OpenIdConfiguration {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub userinfo_endpoint: Option<String>,
    pub jwks_uri: String,
}

impl OpenIdConfiguration {
    pub async fn find_jwks(&self) -> Result<Jwks> {
        JwksDiscovery::new(&self.jwks_uri).execute().await
    }
}

pub struct OpenIdConfigurationDiscovery {
    issuer: String,
}

impl OpenIdConfigurationDiscovery {
    pub fn new(issuer: impl Into<String>) -> Self {
        OpenIdConfigurationDiscovery {
            issuer: issuer.into(),
        }
    }

    pub async fn execute(&self) -> Result<OpenIdConfiguration> {
        let client = reqwest::Client::new();
        let endpoint = format!("{}/.well-known/openid-configuration", self.issuer);
        let response = client.get(endpoint).send().await?;
        let result = response.json::<OpenIdConfiguration>().await?;
        Ok(result)
    }
}


#[derive(Debug, Deserialize, Serialize)]
pub struct Jwks {
    keys: Vec<JsonWebKey>,
}

impl Jwks {
    pub fn find_by_kid(&self, kid: &str) -> Option<&JsonWebKey> {
        self.keys.iter().find(|jwk| jwk.kid == kid)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JsonWebKey {
    pub alg: String,
    #[serde(rename = "use")]
    pub key_use: String,
    pub kid: String,
    pub e: String,
    pub kty: String,
    pub n: String,
}

pub struct JwksDiscovery {
    uri: String,
}

impl JwksDiscovery {
    pub fn new(uri: impl Into<String>) -> Self {
        Self {
            uri: uri.into(),
        }
    }

    pub async fn execute(&self) -> Result<Jwks> {
        let client = reqwest::Client::new();
        let response = client.get(&self.uri).send().await?;
        let result = response.json::<Jwks>().await?;
        Ok(result)
    }
}
