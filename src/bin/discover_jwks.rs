use std::env::args;
use anyhow::Result;

use webauthexp::app::models::oidc::discovery::JwksDiscovery;

#[tokio::main]
async fn main() -> Result<()> {
    if let Some(uri) = args().skip(1).next() {
        let discovery = JwksDiscovery::new(&uri);
        let jwks = discovery.execute().await?;
        print!("{:?}", jwks);
    } else {
        eprintln!("Usage: discover_jwks uri");
    }
    Ok(())
}
