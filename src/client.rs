use std::{env, time::Duration};

use anyhow::{Context, Result};
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use serde::de::DeserializeOwned;

use crate::types::{ErrorResponse, SearchRequest, SearchResponse, UsageResponse};

const BASE_URL: &str = "https://api.cerul.ai";
const CLIENT_SOURCE: &str = "cli";

pub struct CerulClient {
    http: reqwest::Client,
    api_key: String,
}

impl CerulClient {
    pub fn from_env() -> Result<Self> {
        let api_key = env::var("CERUL_API_KEY").context(
            "CERUL_API_KEY is not set.\n\nGet your API key at https://cerul.ai/dashboard\nThen run: export CERUL_API_KEY=cerul_sk_...",
        )?;

        let http = reqwest::Client::builder()
            .user_agent(format!("cerul-cli/{}", env!("CARGO_PKG_VERSION")))
            .timeout(Duration::from_secs(30))
            .build()
            .context("Failed to build Cerul HTTP client")?;

        Ok(Self { http, api_key })
    }

    pub async fn search(&self, request: &SearchRequest) -> Result<SearchResponse> {
        let request = self
            .http
            .post(format!("{BASE_URL}/v1/search"))
            .bearer_auth(&self.api_key)
            .header(ACCEPT, "application/json")
            .header(CONTENT_TYPE, "application/json")
            .header("X-Cerul-Client-Source", CLIENT_SOURCE)
            .json(request);

        self.send(request).await
    }

    pub async fn usage(&self) -> Result<UsageResponse> {
        let request = self
            .http
            .get(format!("{BASE_URL}/v1/usage"))
            .bearer_auth(&self.api_key)
            .header(ACCEPT, "application/json")
            .header("X-Cerul-Client-Source", CLIENT_SOURCE);

        self.send(request).await
    }

    async fn send<T>(&self, request: reqwest::RequestBuilder) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let response = request
            .send()
            .await
            .context("Failed to send request to Cerul API")?;

        let status = response.status();
        let request_id = response
            .headers()
            .get("x-request-id")
            .and_then(|value| value.to_str().ok())
            .map(ToOwned::to_owned);
        let body = response
            .bytes()
            .await
            .context("Failed to read response body from Cerul API")?;

        if !status.is_success() {
            return Err(build_api_error(
                status.as_u16(),
                request_id.as_deref(),
                &body,
            ));
        }

        serde_json::from_slice::<T>(&body).context("Failed to parse Cerul API response")
    }
}

fn build_api_error(status: u16, request_id: Option<&str>, body: &[u8]) -> anyhow::Error {
    if let Ok(payload) = serde_json::from_slice::<ErrorResponse>(body) {
        let mut message = format!(
            "[{status}] {}: {}",
            payload.error.code, payload.error.message
        );
        if let Some(request_id) = request_id {
            message.push_str(&format!(" (request_id: {request_id})"));
        }
        return anyhow::anyhow!(message);
    }

    let fallback = String::from_utf8_lossy(body).trim().to_string();
    if fallback.is_empty() {
        if let Some(request_id) = request_id {
            return anyhow::anyhow!(
                "[{status}] api_error: Request failed (request_id: {request_id})"
            );
        }
        return anyhow::anyhow!("[{status}] api_error: Request failed");
    }

    if let Some(request_id) = request_id {
        return anyhow::anyhow!("[{status}] api_error: {fallback} (request_id: {request_id})");
    }

    anyhow::anyhow!("[{status}] api_error: {fallback}")
}
