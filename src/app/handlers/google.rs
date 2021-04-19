use actix_http::body::Body;
use actix_session::Session;
use actix_web::{HttpResponse, Result, Scope, web::{Data, get, scope}};

use crate::app::config::{AppConfig, GoogleConfig};

pub fn create_scope(config: &AppConfig) -> Scope {
    scope("/google")
        .data(config.google.clone())
        .route("", get().to(index))
        .route("/", get().to(index))
}

async fn index(_config: Data<GoogleConfig>, _session: Session) -> Result<HttpResponse<Body>> {
    let response = HttpResponse::Ok().finish();
    Ok(response)
}
