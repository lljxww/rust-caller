use caller_context::CallerContext;
use reqwest::header;

pub mod caller_context;
pub mod models;
pub mod utils;

pub async fn get() -> Result<String, reqwest::Error> {
    let caller_context = CallerContext::build();

    let client = reqwest::Client::new();

    let url = format!(
        "{}{}",
        caller_context.config.service_items[0].base_url,
        caller_context.config.service_items[0].api_items[0].url
    );

    let result = client
        .get(url)
        .header(header::USER_AGENT, "rust-caller 0.1")
        .header(header::CONTENT_TYPE, "application/json")
        .send()
        .await?
        .text()
        .await?;

    Ok(result)
}
