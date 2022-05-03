use std::env::args;
use anyhow::Result;

use webauthexp::app::models::oidc::discovery::OpenIdConfigurationDiscovery;

#[tokio::main]
async fn main() -> Result<()> {
    if let Some(issuer) = args().nth(1) {
        let discovery = OpenIdConfigurationDiscovery::new(&issuer);
        let configuration = discovery.execute().await?;
        print!("{:?}", configuration);
    } else {
        eprintln!("Usage: discover_openid_configuration issuer-uri");
    }
    Ok(())
}
