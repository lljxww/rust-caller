//! API Documentation Server Example
//!
//! Run with: cargo run --example server --features server

use caller::{Caller, ServerConfig, start_server_with_caller};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build an instance-based caller from caller.json
    let caller = Caller::from_path("caller.json")?;

    // Configure and start server
    let config = ServerConfig::new()
        .addr("127.0.0.1:8080")?
        .title("Caller API")
        .version("1.0.0");

    start_server_with_caller(config, caller).await?;

    Ok(())
}
