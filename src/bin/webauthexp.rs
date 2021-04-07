use actix_session::CookieSession;
use actix_web::{middleware::Logger, App, HttpServer};
use anyhow::Result;
use env_logger::Env;

use webauthexp::app::config::AppArgs;
use webauthexp::app::handlers::{self};

#[actix_rt::main]
async fn main() -> Result<()> {
    let args = AppArgs::new();
    let config = args.load_config().await?;

    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let bind_address = config.bind_address();
    let server = HttpServer::new(move || {
        App::new()
            .data(config.clone())
            .wrap(Logger::default())
            .wrap(Logger::new("%a %t \"%r\" %s %b \"%{Referer}i\" \"%{User-Agent}i\" %T"))
            .wrap(CookieSession::signed(&[0; 32]).secure(false))
            .service(handlers::github::request_authorization)
    });
    server.bind(bind_address)?.run().await?;

    Ok(())
}
