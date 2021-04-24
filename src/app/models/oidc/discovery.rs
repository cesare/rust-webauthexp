use anyhow::Result;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct OpenIdConfiguration {
    issuer: String,
    authorization_endpoint: String,
    token_endpoint: String,
    userinfo_endpoint: Option<String>,
    jwks_uri: String,
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

#[derive(Debug, Deserialize, Serialize)]
pub struct JsonWebKey {
    alg: String,
    #[serde(rename = "use")]
    key_use: String,
    kid: String,
    e: String,
    kty: String,
    n: String,
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
