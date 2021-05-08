use actix_http::body::Body;
use actix_session::Session;
use actix_web::{HttpResponse, Result, Scope, web::{Data, Query, get, scope}};

use crate::app::config::GithubConfig;
use crate::app::models::github::{GithubAutorizationRequest, GithubAuthorizationResponse, GithubSignin};

pub fn create_scope(config: &GithubConfig) -> Scope {
    scope("/github")
        .data(config.clone())
        .route("", get().to(index))
        .route("/", get().to(index))
        .route("/callback", get().to(callback))
}

async fn index(config: Data<GithubConfig>, session: Session) -> Result<HttpResponse<Body>> {
    let request = GithubAutorizationRequest::new(&config);
    let (request_uri, state) = request.create().unwrap();
    session.insert("github-oauth-state", &state)?;

    let response = HttpResponse::Found()
        .insert_header(("Location", request_uri))
        .finish();
    Ok(response)
}

async fn callback(config: Data<GithubConfig>, session: Session, Query(response): Query<GithubAuthorizationResponse>) -> Result<HttpResponse<Body>> {
    let key = "github-oauth-state";
    let saved_state: Option<String> = session.get(key)?;
    let _ = session.remove(key);

    let user = GithubSignin::new(&config)
        .execute(&response, saved_state)
        .await?;
    let response = HttpResponse::Ok().json(user);
    Ok(response)
}
