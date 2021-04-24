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
