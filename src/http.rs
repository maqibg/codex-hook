use serde_json::Value;
use std::time::Duration;

const MAX_RESPONSE_BYTES: usize = 1024 * 1024;

#[derive(Clone, Copy, Debug)]
pub struct RequestError {
    pub code: &'static str,
}

impl RequestError {
    pub const fn new(code: &'static str) -> Self {
        Self { code }
    }
}

pub fn build_client(proxy_url: &str, timeout_ms: u64) -> Result<reqwest::Client, RequestError> {
    let mut builder = reqwest::Client::builder()
        .timeout(Duration::from_millis(timeout_ms))
        .pool_max_idle_per_host(0);
    if !proxy_url.is_empty() {
        let proxy =
            reqwest::Proxy::all(proxy_url).map_err(|_| RequestError::new("invalid-proxy"))?;
        builder = builder.proxy(proxy);
    }
    builder
        .build()
        .map_err(|_| RequestError::new("client-build-failed"))
}

pub async fn response_json(response: reqwest::Response) -> Result<Value, RequestError> {
    if !response.status().is_success() {
        return Err(RequestError::new("http-status"));
    }
    if response
        .content_length()
        .is_some_and(|length| length > MAX_RESPONSE_BYTES as u64)
    {
        return Err(RequestError::new("response-too-large"));
    }
    let bytes = response
        .bytes()
        .await
        .map_err(|_| RequestError::new("response-read-failed"))?;
    if bytes.len() > MAX_RESPONSE_BYTES {
        return Err(RequestError::new("response-too-large"));
    }
    serde_json::from_slice(&bytes).map_err(|_| RequestError::new("invalid-json-response"))
}
