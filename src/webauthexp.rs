use actix_web::{middleware::Logger, App, HttpServer};
use anyhow::Result;
use app::config::AppArgs;
use env_logger::Env;

mod app;
use crate::app::handlers::{self};

#[actix_rt::main]
async fn main() -> Result<()> {
    let args = AppArgs::new();
    let config = args.load_config().await?;

    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %t \"%r\" %s %b \"%{Referer}i\" \"%{User-Agent}i\" %T"))
            .service(handlers::github::request_authorization)
    });
    server.bind(config.bind_address())?.run().await?;

    Ok(())
}
