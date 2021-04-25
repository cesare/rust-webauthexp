use serde_derive::Deserialize;
use thiserror::Error;

pub struct IdToken {
    pub header: Header,
    pub claims: Claims,
    pub signature: Vec<u8>,
}

#[derive(Debug, Deserialize)]
pub struct Header {
    pub alg: String,
    pub typ: String,
}

#[derive(Debug, Deserialize)]
pub struct Claims {
    pub aud: String,
    pub exp: u64,
    pub iat: u64,
    pub iss: String,
    pub sub: String,
}

#[derive(Debug, Error)]
pub enum IdTokenDecodingError {
    #[error("Invalid token format")]
    InvalidFormat,
}

impl From<base64::DecodeError> for IdTokenDecodingError {
    fn from(_error: base64::DecodeError) -> Self {
        IdTokenDecodingError::InvalidFormat
    }
}

impl From<serde_json::Error> for IdTokenDecodingError {
    fn from(_error: serde_json::Error) -> Self {
        IdTokenDecodingError::InvalidFormat
    }
}

pub struct IdTokenDecoder {
}

impl IdTokenDecoder {
    pub fn new() -> Self {
        Self {}
    }

    pub fn execute(&self, raw_token: &str) -> Result<IdToken, IdTokenDecodingError> {
        let parts: Vec<&str> = raw_token.splitn(3, ".").collect();
        if parts.len() != 3 {
            return Err(IdTokenDecodingError::InvalidFormat)
        }

        let binary_header = base64::decode(parts[0])?;
        let binary_payload = base64::decode(parts[1])?;
        let binary_signature = base64::decode(parts[2])?;

        let header = serde_json::from_slice::<Header>(binary_header.as_ref())?;
        let claims = serde_json::from_slice::<Claims>(binary_payload.as_ref())?;

        let id_token = IdToken {
            header: header,
            claims: claims,
            signature: binary_signature,
        };
        Ok(id_token)
    }
}
