use actix_http::body::Body;
use actix_session::Session;
use actix_web::{HttpResponse, Result, Scope, web::{Data, Query, get, scope}};

use crate::app::config::GoogleConfig;
use crate::app::models::google::{GoogleAutorization, GoogleAuthorizationResponse, GoogleSignin, RequestAttributes};

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

    let result = GoogleSignin::new(&config).execute(&response, attributes).await;
    match result {
        Ok(google_id) => {
            let response = HttpResponse::Ok().json(google_id);
            Ok(response)
        },
        _ => {
            let response = HttpResponse::InternalServerError().finish();
            Ok(response)
        }
    }
}
