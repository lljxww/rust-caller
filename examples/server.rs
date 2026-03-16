//! API Documentation Server Example
//!
//! Run with: cargo run --example server --features server

use caller::{init_config, start_server, ServerConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize configuration from caller.json
    init_config()?;

    // Configure and start server
    let config = ServerConfig::new()
        .addr("127.0.0.1:8080")?
        .title("Caller API")
        .version("1.0.0");

    start_server(config).await?;

    Ok(())
}
