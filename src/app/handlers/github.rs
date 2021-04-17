use actix_http::body::Body;
use actix_session::Session;
use actix_web::{HttpResponse, Result, Scope, web::{Data, Query, get, scope}};

use crate::app::config::{AppConfig, GithubConfig};
use crate::app::models::github::{GithubAutorizationRequest, GithubAuthorizationResponse, GithubSignin};

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

async fn callback(data: Data<GithubConfig>, session: Session, Query(response): Query<GithubAuthorizationResponse>) -> Result<HttpResponse<Body>> {
    let signin = GithubSignin::new(&data);
    let saved_state: Option<String> = session.get("github-oauth-state")?;
    let user = signin.execute(&response, saved_state).await?;
    let response = HttpResponse::Ok().json(user);
    Ok(response)
}
