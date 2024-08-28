use super::caller_const;
use super::config_loader::ConfigLoader;
use crate::models::caller_config::CallerConfig;
use crate::models::service_item::ServiceItem;
use crate::models::{api_item::ApiItem, api_result::ApiResult};
use reqwest::{header, Method};
use std::collections::HashMap;

pub struct CallerContext {
    pub service_name: String,
    pub api_name: String,
    pub service_item: ServiceItem,
    pub api_item: ApiItem,
    pub http_method: Method,
    pub url: String,
    pub params: Option<HashMap<String, String>>,
}

impl CallerContext {
    fn build(method: &str, params: Option<HashMap<String, String>>) -> CallerContext {
        let splited_method = split_method(method);

        if splited_method.len() != 2 {
            panic!("method is not valid");
        }

        let service_name = splited_method[0].to_owned();
        let api_name = splited_method[1].to_owned();

        let (service_item, api_item) = ConfigLoader::get_config(&service_name, &api_name);

        let mut url = get_final_url(method, &ConfigLoader::get_config_ref());

        if api_item.param_type.to_lowercase() == "path" {
            if !params.is_none() {
                for param in params.clone().unwrap() {
                    url = url.replace(format!("{{{}}}", param.0).as_str(), param.1.as_str());
                }
            }
        }

        let mut caller_context = CallerContext {
            service_name,
            api_name,
            service_item,
            api_item,
            http_method: Method::default(),
            url,
            params,
        };

        caller_context.http_method = get_http_method(&caller_context.api_item.http_method);

        caller_context
    }

    pub async fn call(
        method: &str,
        params: Option<HashMap<String, String>>,
    ) -> Result<ApiResult, reqwest::Error> {
        let context = CallerContext::build(method, params);
        let client = reqwest::Client::new();

        let mut rb = client
            .request(context.http_method, context.url)
            .header(header::USER_AGENT, caller_const::UA)
            .header(header::CONTENT_TYPE, caller_const::DEFAULT_CONTENT_TYPE);

        if !context.params.is_none() {
            match context.api_item.param_type.to_lowercase().as_str() {
                "query" => rb = rb.query(&context.params.unwrap()),
                "json" => rb = rb.json(&context.params.unwrap()),
                "form" => rb = rb.form(&context.params.unwrap()),
                _ => (),
            }
        }

        let response = rb.send().await?;

        let status_code = response.status();
        let result = response.text().await?;

        Ok(ApiResult::build(result, status_code))
    }
}

fn split_method(method: &str) -> Vec<String> {
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

fn get_final_url(method: &str, config: &CallerConfig) -> String {
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

fn get_http_method(http_method: &str) -> Method {
    match http_method.to_lowercase().as_str() {
        "get" => Method::GET,
        "post" => Method::POST,
        "put" => Method::PUT,
        "delete" => Method::DELETE,
        "patch" => Method::PATCH,
        _ => panic!("http method is not valid"),
    }
}
