use actix_web::{App, HttpServer, middleware::Logger};
use anyhow::Result;
use env_logger::Env;

mod app;
use crate::app::config::AppConfig;

#[actix_rt::main]
async fn main() -> Result<()> {
    let config = AppConfig::new();

    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let server = HttpServer::new(move || {
        App::new()
        .wrap(Logger::default())
        .wrap(Logger::new("%a %t \"%r\" %s %b \"%{Referer}i\" \"%{User-Agent}i\" %T"))
    });
    server.bind(config.bind_address())?.run().await?;

    Ok(())
}
