use core::panic;
use std::collections::HashMap;

use crate::models::caller_config::CallerConfig;
use caller_context::CallerContext;
use http_method::HttpMethod;
use reqwest::header;

pub mod caller_const;
pub mod caller_context;
pub mod http_method;
pub mod models;
pub mod utils;

async fn call(method: String, param: Option<HashMap<String, String>>) {}

fn split_method(method: String) -> Vec<String> {
    if !method.contains(".") {
        panic!("method is not valid");
    }

    let splited_method: Vec<String> = method
        .split('.')
        .into_iter()
        .map(|s| s.to_owned())
        .collect();

    if splited_method.len() != 2 {
        panic!("method is not valid");
    }

    splited_method
}

fn get_final_url(method: String, config: &CallerConfig) -> String {
    let splited_method = split_method(method);

    let service_item = config
        .service_items
        .iter()
        .find(|s| s.api_name == splited_method[0])
        .unwrap();

    let api_item = service_item
        .api_items
        .iter()
        .find(|a| a.method == splited_method[1])
        .unwrap();

    format!("{}{}", service_item.base_url, api_item.url)
}

fn get_http_method(http_method: String) -> HttpMethod {
    match http_method.to_lowercase().as_str() {
        "get" => HttpMethod::Get,
        "post" => HttpMethod::Post,
        "put" => HttpMethod::Put,
        "delete" => HttpMethod::Delete,
        "options" => HttpMethod::Options,
        _ => panic!("http method is not valid"),
    }
}

pub async fn get(method: &str) -> Result<String, reqwest::Error> {
    let caller_context = CallerContext::build();

    let client = reqwest::Client::new();

    let url = get_final_url(method.to_string(), &caller_context.config);

    let result = client
        .get(url)
        .header(header::USER_AGENT, caller_const::UA)
        .header(header::CONTENT_TYPE, caller_const::DEFAULT_CONTENT_TYPE)
        .send()
        .await?
        .text()
        .await?;

    Ok(result)
}
