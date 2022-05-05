use std::cmp::PartialEq;

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
            return Self::DecodingFailed { uri, message }
        }
        Self::HttpRequestFailed { uri, message }
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

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
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

#[cfg(test)]
mod tests {
    use httpmock::MockServer;
    use httpmock::Method::GET;
    use serde_json::json;
    use super::*;

    #[actix_rt::test]
    async fn test_oidc_configuration_discovery() {
        let server = MockServer::start();

        let issuer = server.base_url();
        let authorization_endpoint = format!("{}/authorize", issuer);
        let token_endpoint = format!("{}/token", issuer);
        let userinfo_endpoint = format!("{}/userinfo", issuer);
        let jwks_endpoint = format!("{}/jwks", issuer);

        let _mock = server.mock(|when, then| {
            when.method(GET).path("/.well-known/openid-configuration");
            then.status(200).json_body(json!({
                "issuer": issuer,
                "authorization_endpoint": authorization_endpoint,
                "token_endpoint": token_endpoint,
                "userinfo_endpoint": userinfo_endpoint,
                "jwks_uri": jwks_endpoint,
            }));
        });
        let discovery = OpenIdConfigurationDiscovery::new(server.base_url());
        let result = discovery.execute().await.unwrap();

        assert_eq!(issuer, result.issuer);
        assert_eq!(authorization_endpoint, result.authorization_endpoint);
        assert_eq!(token_endpoint, result.token_endpoint);
        assert_eq!(Some(userinfo_endpoint), result.userinfo_endpoint);
        assert_eq!(jwks_endpoint, result.jwks_uri);
    }

    #[actix_rt::test]
    async fn test_find_jwks() {
        let server = MockServer::start();

        let issuer = server.base_url();
        let authorization_endpoint = format!("{}/authorize", issuer);
        let token_endpoint = format!("{}/token", issuer);
        let userinfo_endpoint = format!("{}/userinfo", issuer);
        let jwks_endpoint = format!("{}/jwks", issuer);

        let oidc_config = OpenIdConfiguration {
            issuer: issuer,
            authorization_endpoint: authorization_endpoint,
            token_endpoint: token_endpoint,
            userinfo_endpoint: Some(userinfo_endpoint),
            jwks_uri: jwks_endpoint,
        };
        let _mock = server.mock(|when, then| {
            when.method(GET).path("/jwks");
            then.status(200).json_body(json!({
                "keys": [
                    {
                        "alg": "RS256",
                        "kty": "RSA",
                        "kid": "test-key-id",
                        "use": "sig",
                        "n": "xxxxxx",
                        "e": "yyyyyyy",
                    }
                ]
            }));
        });

        let results = oidc_config.find_jwks().await.unwrap();
        assert_eq!(1, results.keys.len());

        let jwk = results.keys.first().unwrap();
        assert_eq!("RS256", jwk.alg);
        assert_eq!("RSA", jwk.kty);
        assert_eq!("test-key-id", jwk.kid);
        assert_eq!("sig", jwk.key_use);
        assert_eq!("xxxxxx", jwk.n);
        assert_eq!("yyyyyyy", jwk.e);
    }

    #[test]
    fn test_find_by_kid() {
        let jwk1 = JsonWebKey {
            alg: "RS256".to_owned(),
            key_use: "sig".to_owned(),
            kid: "key-01".to_owned(),
            e: "xxxxxx".to_owned(),
            kty: "RSA".to_owned(),
            n: "yyyyyy".to_owned(),
        };
        let jwk2 = JsonWebKey {
            alg: "RS256".to_owned(),
            key_use: "sig".to_owned(),
            kid: "key-02".to_owned(),
            e: "aaaaa".to_owned(),
            kty: "RSA".to_owned(),
            n: "bbbbb".to_owned(),
        };
        let jwks = Jwks {
            keys: vec![
                jwk1.to_owned(),
                jwk2.to_owned()
            ],
        };

        let result = jwks.find_by_kid("key-02");
        assert_eq!(Some(&jwk2), result);

        let result = jwks.find_by_kid("no-such-key");
        assert_eq!(None, result);
    }
}
