use actix_http::body::Body;
use actix_session::Session;
use actix_web::{HttpResponse, Resource, Result, web::{get, resource}};

use crate::app::config::AppConfig;

pub fn create_resource() -> Resource {
    resource("/github").route(get().to(index))
}

async fn index(_data: actix_web::web::Data<AppConfig>, _session: Session) -> Result<HttpResponse<Body>> {
    let response = HttpResponse::Ok().body("ok");
    Ok(response)
}
