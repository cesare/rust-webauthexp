use actix_http::body::Body;
use actix_session::Session;
use actix_web::{HttpResponse, Resource, Result, web::{get, resource, Data}};

use crate::app::config::{AppConfig, GithubConfig};

pub fn create_resource(config: &AppConfig) -> Resource {
    resource("/github")
        .data(config.github.clone())
        .route(get().to(index))
}

async fn index(_data: Data<GithubConfig>, _session: Session) -> Result<HttpResponse<Body>> {
    let response = HttpResponse::Ok().body("ok");
    Ok(response)
}
