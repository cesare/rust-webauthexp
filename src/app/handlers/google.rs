use actix_http::body::Body;
use actix_session::Session;
use actix_web::{HttpResponse, Result, Scope, web::{Data, Query, get, scope}};

use crate::app::config::{AppConfig, GoogleConfig};
use crate::app::models::google::{GoogleAutorization, GoogleAuthorizationResponse};

pub fn create_scope(config: &AppConfig) -> Scope {
    scope("/google")
        .data(config.google.clone())
        .route("", get().to(index))
        .route("/", get().to(index))
        .route("/callback", get().to(callback))
}

async fn index(config: Data<GoogleConfig>, session: Session) -> Result<HttpResponse<Body>> {
    let (request_uri, state, nonce) = GoogleAutorization::new(&config).start().unwrap();
    session.insert("google-oidc-state", &state)?;
    session.insert("google-oidc-nonce", &nonce)?;

    let response = HttpResponse::Found()
        .insert_header(("Location", request_uri))
        .finish();
    Ok(response)
}

async fn callback(_config: Data<GoogleConfig>, _session: Session, Query(_response): Query<GoogleAuthorizationResponse>) -> Result<HttpResponse<Body>> {
    let response = HttpResponse::Ok().finish();
    Ok(response)
}
