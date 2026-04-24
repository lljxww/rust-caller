use caller::{call, init_config};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_config()?;

    let result = call("JP.list", None).await?;
    println!("Status: {}", result.status_code);

    if let Some(title) = result.str_at("0.title") {
        println!("First post: {}", title);
    }

    let params = HashMap::from([("post_id".to_string(), "1".to_string())]);
    let result = call("JP.get", Some(params)).await?;
    println!("Post 1 title: {}", result.str_at("title").unwrap_or("N/A"));

    Ok(())
}
