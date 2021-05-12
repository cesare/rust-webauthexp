use actix_http::body::Body;
use actix_session::Session;
use actix_web::{HttpResponse, Result, Scope, web::{Data, Query, get, scope}};

use crate::app::{config::SpotifyConfig, models::spotify::SpotifyAuthorization};
use crate::app::models::spotify::AuthResponse;

pub fn create_scope(config: &SpotifyConfig) -> Scope {
    scope("/spotify")
        .data(config.clone())
        .route("", get().to(index))
        .route("/", get().to(index))
        .route("/callback", get().to(callback))
}

async fn index(config: Data<SpotifyConfig>, session: Session) -> Result<HttpResponse<Body>> {
    let request = SpotifyAuthorization::new(&config).start().unwrap();
    session.insert("spotify-oauth", &request.attributes)?;

    let response = HttpResponse::Found()
        .insert_header(("Location", request.request_uri))
        .finish();
    Ok(response)
}

async fn callback(_config: Data<SpotifyConfig>, _session: Session, Query(_response): Query<AuthResponse>) -> Result<HttpResponse<Body>> {
    todo!()
}
