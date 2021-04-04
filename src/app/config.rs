use std::path::PathBuf;

use anyhow::Result;
use serde_derive::Deserialize;
use structopt::StructOpt;
use tokio::{fs::File, io::AsyncReadExt};

#[derive(StructOpt)]
#[structopt(name = "webauthexp")]
pub struct AppArgs {
    #[structopt(short, long, parse(from_os_str))]
    config: PathBuf,
}

impl AppArgs {
    pub fn new() -> Self {
        Self::from_args()
    }

    pub async fn load_config(&self) -> Result<AppConfig> {
        let mut file = File::open(&self.config).await?;
        let mut content = String::new();
        file.read_to_string(&mut content).await?;
        let config: AppConfig = toml::from_str(&content)?;
        Ok(config)
    }
}

#[derive(Debug, Deserialize)]
struct ServerConfig {
    bind: String,
    port: i32,
}

#[derive(Debug, Deserialize)]
struct GithubConfig {
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    scope: String,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    server: ServerConfig,
    github: GithubConfig,
}

impl AppConfig {
    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.server.bind, self.server.port)
    }
}
