use actix_http::body::Body;
use actix_session::Session;
use actix_web::{HttpResponse, Result, web::{get, resource, ServiceConfig}};

use crate::app::config::AppConfig;

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        resource("/github")
            .route(get().to(index))
    );
}

pub async fn index(_data: actix_web::web::Data<AppConfig>, _session: Session) -> Result<HttpResponse<Body>> {
    let response = HttpResponse::Ok().body("ok");
    Ok(response)
}
