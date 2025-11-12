use reqwest::{Client, header, Method};
use crate::shared::error::CallerError;

/// HTTP client for making requests
pub struct HttpClient {
    client: Client,
}

impl HttpClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn request(
        &self,
        method: &str,
        url: &str,
    ) -> Result<String, CallerError> {
        let http_method = Method::from_bytes(method.as_bytes())
            .map_err(|_| CallerError::http_method_not_supported(method.to_string()))?;

        let request = self.client
            .request(http_method, url)
            .header(header::USER_AGENT, "caller/0.1.0");

        let response = request.send().await?;
        let text = response.text().await?;

        Ok(text)
    }

    pub fn get_client(&self) -> &Client {
        &self.client
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}