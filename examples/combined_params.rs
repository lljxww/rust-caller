use caller::{call, init_config};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_config()?;

    let update_params = HashMap::from([
        ("post_id".to_string(), "1".to_string()),
        ("title".to_string(), "Updated Title".to_string()),
        ("body".to_string(), "Updated body content".to_string()),
        ("userId".to_string(), "1".to_string()),
    ]);

    let result = call("JP.update", Some(update_params)).await?;
    println!("Status: {}", result.status_code);
    println!("Updated title: {}", result.str_at("title").unwrap_or("N/A"));

    let patch_params = HashMap::from([
        ("post_id".to_string(), "1".to_string()),
        ("title".to_string(), "Patched Title".to_string()),
    ]);

    let result = call("JP.patch", Some(patch_params)).await?;
    println!("Patch status: {}", result.status_code);
    println!("Patched title: {}", result.str_at("title").unwrap_or("N/A"));

    Ok(())
}
