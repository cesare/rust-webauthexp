use actix_http::{ResponseError, body::Body};
use actix_session::Session;
use actix_web::{HttpResponse, Result, Scope, web::{Data, Query, get, scope}};
use serde::Serialize;

use crate::app::config::GoogleConfig;
use crate::app::models::google::{GoogleAutorization, GoogleAuthorizationResponse, GoogleSignin, GoogleSigninError, RequestAttributes};

#[derive(Debug, Serialize)]
struct ErrorMessage {
    message: String,
}

impl ResponseError for GoogleSigninError {
    fn error_response(&self) -> HttpResponse {
        let message = ErrorMessage {
            message: self.to_string(),
        };
        HttpResponse::InternalServerError().json(message)
    }
}

pub fn create_scope(config: &GoogleConfig) -> Scope {
    scope("/google")
        .data(config.clone())
        .route("", get().to(index))
        .route("/", get().to(index))
        .route("/callback", get().to(callback))
}

async fn index(config: Data<GoogleConfig>, session: Session) -> Result<HttpResponse<Body>> {
    let request = GoogleAutorization::new(&config).start().unwrap();
    session.insert("google-oidc", &request.attributes)?;

    let response = HttpResponse::Found()
        .insert_header(("Location", request.request_uri))
        .finish();
    Ok(response)
}

async fn callback(config: Data<GoogleConfig>, session: Session, Query(response): Query<GoogleAuthorizationResponse>) -> Result<HttpResponse<Body>> {
    let key = "google-oidc";
    let attributes = session.get::<RequestAttributes>(key)?;
    let _ = session.remove(key);

    let google_id = GoogleSignin::new(&config, &response, attributes).execute().await?;
    let response = HttpResponse::Ok().json(google_id);
    Ok(response)
}
