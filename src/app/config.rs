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

#[derive(Clone, Debug, Deserialize)]
struct ServerConfig {
    bind: String,
    port: u16,
}

#[derive(Clone, Debug, Deserialize)]
pub struct GithubConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub scope: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct GoogleConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub scope: String,
}

impl GoogleConfig {
    pub fn issuer(&self) -> String {
        String::from("https://accounts.google.com")
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct AppConfig {
    server: ServerConfig,
    pub github: GithubConfig,
    pub google: GoogleConfig,
}

impl AppConfig {
    pub fn bind_address(&self) -> (String, u16) {
        (self.server.bind.to_owned(), self.server.port)
    }
}
