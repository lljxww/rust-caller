use caller::{Caller, NoAuth};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let caller = Caller::from_path("caller.json")?;

    // Instance-local auth registration.
    caller.register_auth("demo", NoAuth)?;

    let result = caller.call("JP.list", None).await?;
    println!("Status: {}", result.status_code);
    println!("First title: {}", result.str_at("0.title").unwrap_or("N/A"));

    Ok(())
}
