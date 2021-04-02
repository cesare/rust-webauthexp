use actix_web::{App, HttpServer};
use anyhow::Result;

mod app;
use crate::app::config::AppConfig;

#[actix_rt::main]
async fn main() -> Result<()> {
    let config = AppConfig::new();

    let server = HttpServer::new(move || {
        App::new()
    });
    server.bind(config.bind_address())?.run().await?;

    Ok(())
}
