use actix_http::body::Body;
use actix_session::Session;
use actix_web::{HttpResponse, Result, Scope, web::{Data, Query, get, scope}};

use crate::app::{config::{AppConfig, GithubConfig}, models::github::GithubAuthorizationResponse};
use crate::app::models::github::GithubAutorizationRequest;

pub fn create_scope(config: &AppConfig) -> Scope {
    scope("/github")
        .data(config.github.clone())
        .route("", get().to(index))
        .route("/", get().to(index))
        .route("/callback", get().to(callback))
}

async fn index(data: Data<GithubConfig>, session: Session) -> Result<HttpResponse<Body>> {
    let request = GithubAutorizationRequest::new(&data);
    session.insert("github-oauth-state", &request.state)?;

    let request_uri = request.request_uri().unwrap();
    let response = HttpResponse::Found()
        .insert_header(("Location", request_uri))
        .finish();
    Ok(response)
}

async fn callback(_data: Data<GithubConfig>, _session: Session, response: Query<GithubAuthorizationResponse>) -> Result<HttpResponse<Body>> {
    let message = format!("ok: {:?}", response);
    let response = HttpResponse::Ok().body(message);
    Ok(response)
}
