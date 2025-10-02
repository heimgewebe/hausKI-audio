use serde_json::{json, Map, Value};
use url::Url;

use crate::error::AppError;

pub async fn proxy(
    client: &reqwest::Client,
    url: &Url,
    payload: Value,
) -> Result<Value, AppError> {
    send_rpc(client, url, payload).await
}

pub async fn call_method(
    client: &reqwest::Client,
    url: &Url,
    method: &str,
    params: Option<Value>,
) -> Result<Value, AppError> {
    let mut payload = serde_json::Map::new();
    payload.insert("jsonrpc".into(), Value::String("2.0".into()));
    payload.insert("id".into(), Value::from(1));
    payload.insert("method".into(), Value::String(method.to_string()));
    if let Some(params) = params {
        payload.insert("params".into(), params);
    }

    let response = send_rpc(client, url, Value::Object(payload)).await?;
    if let Some(error) = response.get("error") {
        let message = error
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("unknown Mopidy error");
        return Err(AppError::upstream(format!("{message}")));
    }

    response
        .get("result")
        .cloned()
        .ok_or_else(|| AppError::upstream("Mopidy response missing result".into()))
}

pub async fn lookup_track(
    client: &reqwest::Client,
    url: &Url,
    uri: &str,
) -> Result<Option<Value>, AppError> {
    let result = call_method(
        client,
        url,
        "core.library.lookup",
        Some(json!({ "uri": uri })),
    )
    .await?;

    Ok(result.as_array().and_then(|arr| arr.first().cloned()))
}

pub async fn search_any(
    client: &reqwest::Client,
    url: &Url,
    query: &str,
) -> Result<Vec<Value>, AppError> {
    let result = call_method(
        client,
        url,
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

pub async fn health_check(client: &reqwest::Client, url: &Url) -> Result<(), String> {
    let payload = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "core.playback.get_state",
    });

    match send_rpc(client, url, payload).await {
        Ok(value) => {
            if value.get("error").is_some() {
                Err("RPC error returned".to_string())
            } else {
                Ok(())
            }
        }
        Err(err) => Err(err.to_string()),
    }
}

async fn send_rpc(
    client: &reqwest::Client,
    url: &Url,
    payload: Value,
) -> Result<Value, AppError> {
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
