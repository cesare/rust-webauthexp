use actix_http::body::Body;
use actix_session::Session;
use actix_web::{get, HttpResponse, Result};

#[get("/github")]
pub async fn request_authorization(_session: Session) -> Result<HttpResponse<Body>> {
    let response = HttpResponse::Ok().body("ok");
    Ok(response)
}
