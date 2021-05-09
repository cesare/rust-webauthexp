use actix_http::body::Body;
use actix_session::Session;
use actix_web::{HttpResponse, Result, Scope, web::{Data, Query, get, scope}};

use crate::app::config::SpotifyConfig;
use crate::app::models::spotify::{SpotifyAuthorizationResponse};

pub fn create_scope(config: &SpotifyConfig) -> Scope {
    scope("/spotify")
        .data(config.clone())
        .route("", get().to(index))
        .route("/", get().to(index))
        .route("/callback", get().to(callback))
}

async fn index(_config: Data<SpotifyConfig>, _session: Session) -> Result<HttpResponse<Body>> {
    todo!()
}

async fn callback(_config: Data<SpotifyConfig>, _session: Session, Query(_response): Query<SpotifyAuthorizationResponse>) -> Result<HttpResponse<Body>> {
    todo!()
}
