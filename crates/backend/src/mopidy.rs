use async_trait::async_trait;
use serde_json::{json, Map, Value};
use url::Url;

use crate::error::AppError;

#[async_trait]
pub trait MopidyClient: Send + Sync + 'static {
    async fn proxy(&self, payload: Value) -> Result<Value, AppError>;

    async fn call_method(&self, method: &str, params: Option<Value>) -> Result<Value, AppError> {
        let mut payload = Map::new();
        payload.insert("jsonrpc".into(), Value::String("2.0".into()));
        payload.insert("id".into(), Value::from(1));
        payload.insert("method".into(), Value::String(method.to_string()));
        if let Some(params) = params {
            payload.insert("params".into(), params);
        }

        let response = self.proxy(Value::Object(payload)).await?;
        if let Some(error) = response.get("error") {
            let message = error
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("unknown Mopidy error");
            return Err(AppError::upstream(message));
        }

        response
            .get("result")
            .cloned()
            .ok_or_else(|| AppError::upstream("Mopidy response missing result"))
    }

    async fn lookup_track(&self, uri: &str) -> Result<Option<Value>, AppError> {
        let result = self
            .call_method("core.library.lookup", Some(json!({ "uri": uri })))
            .await?;

        Ok(result.as_array().and_then(|arr| arr.first().cloned()))
    }

    async fn search_any(&self, query: &str) -> Result<Vec<Value>, AppError> {
        let result = self
            .call_method(
                "core.library.search",
                Some(json!({
                    "query": {
                        "any": [query],
                    },
                    "exact": false,
                })),
            )
            .await?;

        Ok(result.as_array().cloned().unwrap_or_default())
    }

    async fn health_check(&self) -> Result<(), String> {
        let payload = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "core.playback.get_state",
        });

        match self.proxy(payload).await {
            Ok(value) => {
                if let Some(error) = value.get("error") {
                    let message = error
                        .get("message")
                        .and_then(Value::as_str)
                        .unwrap_or("RPC error returned");
                    Err(message.to_string())
                } else {
                    Ok(())
                }
            }
            Err(err) => Err(err.to_string()),
        }
    }
}

#[derive(Clone)]
pub struct HttpMopidyClient {
    client: reqwest::Client,
    url: Url,
}

impl HttpMopidyClient {
    pub fn new(client: reqwest::Client, url: Url) -> Self {
        Self { client, url }
    }
}

#[async_trait]
impl MopidyClient for HttpMopidyClient {
    async fn proxy(&self, payload: Value) -> Result<Value, AppError> {
        send_rpc(&self.client, &self.url, payload).await
    }
}

async fn send_rpc(client: &reqwest::Client, url: &Url, payload: Value) -> Result<Value, AppError> {
    let response = client
        .post(url.as_str())
        .json(&payload)
        .send()
        .await
        .map_err(|err| AppError::upstream(format!("failed to reach Mopidy: {err}")))?;

    let status = response.status();
    let bytes = response
        .bytes()
        .await
        .map_err(|err| AppError::upstream(format!("failed to read Mopidy response: {err}")))?;

    if !status.is_success() {
        let body = String::from_utf8_lossy(&bytes);
        return Err(AppError::upstream(format!(
            "Mopidy returned {status}: {body}",
            status = status
        )));
    }

    serde_json::from_slice::<Value>(&bytes)
        .map_err(|err| AppError::upstream(format!("invalid Mopidy JSON response: {err}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct StubClient {
        responses: Mutex<HashMap<String, Value>>,
        calls: Mutex<Vec<String>>,
    }

    impl StubClient {
        fn new() -> Self {
            Self {
                responses: Mutex::new(HashMap::new()),
                calls: Mutex::new(Vec::new()),
            }
        }

        fn set_response(&self, method: &str, response: Value) {
            self.responses
                .lock()
                .unwrap()
                .insert(method.to_string(), response);
        }

        fn calls(&self) -> Vec<String> {
            self.calls.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl MopidyClient for StubClient {
        async fn proxy(&self, payload: Value) -> Result<Value, AppError> {
            let method = payload
                .get("method")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            self.calls.lock().unwrap().push(method.clone());

            self.responses
                .lock()
                .unwrap()
                .get(&method)
                .cloned()
                .ok_or_else(|| AppError::internal(format!("unexpected method {method}")))
        }
    }

    #[tokio::test]
    async fn call_method_returns_result_payload() {
        let client = StubClient::new();
        client.set_response(
            "core.library.lookup",
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": "ok",
            }),
        );

        let value = client
            .call_method("core.library.lookup", None)
            .await
            .expect("result");

        assert_eq!(value, Value::String("ok".into()));
        assert_eq!(client.calls(), vec!["core.library.lookup".to_string()]);
    }

    #[tokio::test]
    async fn call_method_maps_error_response() {
        let client = StubClient::new();
        client.set_response(
            "core.library.lookup",
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "error": {"message": "boom"},
            }),
        );

        let err = client
            .call_method("core.library.lookup", None)
            .await
            .expect_err("should fail");

        match err {
            AppError::Upstream(message) => assert_eq!(message, "boom"),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[tokio::test]
    async fn lookup_track_returns_first_entry() {
        let client = StubClient::new();
        client.set_response(
            "core.library.lookup",
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": [
                    {"uri": "track:1"},
                    {"uri": "track:2"}
                ],
            }),
        );

        let track = client
            .lookup_track("track:seed")
            .await
            .expect("result")
            .expect("track");

        assert_eq!(track.get("uri").unwrap(), "track:1");
    }

    #[tokio::test]
    async fn search_any_returns_results_vector() {
        let client = StubClient::new();
        client.set_response(
            "core.library.search",
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": [
                    {"tracks": []},
                ],
            }),
        );

        let results = client.search_any("q").await.expect("result");

        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn health_check_surfaces_message() {
        let client = StubClient::new();
        client.set_response(
            "core.playback.get_state",
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "error": {"message": "offline"},
            }),
        );

        let err = client.health_check().await.expect_err("should error");
        assert_eq!(err, "offline");
    }
}
