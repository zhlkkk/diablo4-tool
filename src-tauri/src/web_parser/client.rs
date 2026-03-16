use std::time::Duration;

use serde_json::Value;

use crate::types::BuildPlan;

use super::error::ParserError;
use super::extract::extract_build_id;
use super::parse::parse_build_response;

const TCB_ENDPOINT: &str = "https://diablocore-4gkv4qjs9c6a0b40.ap-shanghai.tcb-api.tencentcloudapi.com/web?env=diablocore-4gkv4qjs9c6a0b40";
const TCB_ENV: &str = "diablocore-4gkv4qjs9c6a0b40";
const TCB_SDK_VERSION: &str = "@cloudbase/js-sdk/1.0.0";

/// HTTP client for the d2core.com Tencent CloudBase API.
pub struct D2CoreClient {
    http: reqwest::Client,
    endpoint: String,
    env: String,
}

impl Default for D2CoreClient {
    fn default() -> Self {
        Self::new()
    }
}

impl D2CoreClient {
    /// Create a new client with hardcoded TCB constants.
    pub fn new() -> Self {
        let http = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30))
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(
                    "x-sdk-version",
                    reqwest::header::HeaderValue::from_static(TCB_SDK_VERSION),
                );
                headers.insert(
                    reqwest::header::CONTENT_TYPE,
                    reqwest::header::HeaderValue::from_static("application/json;charset=UTF-8"),
                );
                headers
            })
            .build()
            .expect("failed to build reqwest client");

        Self {
            http,
            endpoint: TCB_ENDPOINT.to_string(),
            env: TCB_ENV.to_string(),
        }
    }

    /// Fetch and parse a Diablo IV build from d2core.com.
    ///
    /// Accepts a full d2core.com URL (`https://d2core.com/d4/planner?bd=1QMw`)
    /// or a raw build ID (`1QMw`).
    pub async fn fetch_build(&self, url_or_id: &str) -> Result<BuildPlan, ParserError> {
        let build_id = extract_build_id(url_or_id)?;
        let raw_json = self
            .call_tcb(
                "function-planner-queryplan",
                serde_json::json!({"bd": build_id, "enableVariant": true}),
            )
            .await?;
        parse_build_response(raw_json, &build_id)
    }

    /// Send a POST request to the Tencent CloudBase endpoint invoking a cloud function.
    ///
    /// Note: `request_data` is a JSON-encoded string inside the JSON body (double-serialized),
    /// matching the TCB SDK wire format.
    async fn call_tcb(
        &self,
        func_name: &str,
        params: Value,
    ) -> Result<Value, ParserError> {
        let request_data =
            serde_json::to_string(&params).expect("serde_json serialization is infallible");

        let body = serde_json::json!({
            "action": "functions.invokeFunction",
            "dataVersion": "2019-08-16",
            "env": self.env,
            "function_name": func_name,
            "request_data": request_data,
        });

        let response = self
            .http
            .post(&self.endpoint)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ParserError::ApiError {
                code: response.status().as_str().to_string(),
                message: format!("HTTP {} from TCB endpoint", response.status()),
            });
        }

        let json: Value = response.json().await?;
        Ok(json)
    }
}
