use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SpotifyAuthorizationResponse {
    pub state: String,
    pub code: String,
    pub scope: String,
}
