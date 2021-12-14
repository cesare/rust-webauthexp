use actix_session::Session;
use actix_web::{HttpResponse, ResponseError, Result, Scope, web::{Data, Query, get, scope}};
use actix_web::body::BoxBody;
use serde::Serialize;

use crate::app::config::SpotifyConfig;
use crate::app::models::spotify::{AuthResponse, RequestAttributes, SpotifyAuthorization, SpotifySignin, SpotifySigninError};

#[derive(Debug, Serialize)]
struct ErrorMessage {
    message: String,
}

impl ResponseError for SpotifySigninError {
    fn error_response(&self) -> HttpResponse {
        let message = ErrorMessage {
            message: self.to_string(),
        };
        HttpResponse::InternalServerError().json(message)
    }
}

pub fn create_scope(config: &SpotifyConfig) -> Scope {
    scope("/spotify")
        .app_data(config.clone())
        .route("", get().to(index))
        .route("/", get().to(index))
        .route("/callback", get().to(callback))
}

async fn index(config: Data<SpotifyConfig>, session: Session) -> Result<HttpResponse<BoxBody>> {
    let request = SpotifyAuthorization::new(&config).start().unwrap();
    session.insert("spotify-oauth", &request.attributes)?;

    let response = HttpResponse::Found()
        .insert_header(("Location", request.request_uri))
        .finish();
    Ok(response)
}

async fn callback(config: Data<SpotifyConfig>, session: Session, Query(response): Query<AuthResponse>) -> Result<HttpResponse<BoxBody>> {
    let key = "spotify-oauth";
    let attributes = session.get::<RequestAttributes>(key)?;
    let _ = session.remove(key);

    let result = SpotifySignin::new(&config, &response, &attributes).execute().await?;
    let response = HttpResponse::Ok().json(result);
    Ok(response)
}
