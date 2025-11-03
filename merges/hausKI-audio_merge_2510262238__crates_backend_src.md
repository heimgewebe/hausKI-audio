### 📄 crates/backend/src/config.rs

**Größe:** 4 KB | **md5:** `97e05ae9645e2a2a6435bccbf9f27a5f`

```rust
use std::env;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::Duration;

use thiserror::Error;
use url::Url;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub bind_addr: SocketAddr,
    pub mopidy_rpc_url: Url,
    pub audio_mode_script: ScriptConfig,
    pub playlist_script: ScriptConfig,
    pub script_workdir: PathBuf,
    pub command_timeout: Duration,
    pub check_mopidy_health: bool,
}

#[derive(Debug, Clone)]
pub struct ScriptConfig {
    pub program: PathBuf,
}

impl ScriptConfig {
    pub fn resolve_with(&self, base: &Path) -> PathBuf {
        if self.program.is_absolute() {
            self.program.clone()
        } else {
            base.join(&self.program)
        }
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("invalid bind address '{0}'")]
    InvalidBindAddress(String),
    #[error("invalid Mopidy RPC URL '{0}'")]
    InvalidMopidyUrl(String),
    #[error("failed to determine working directory: {0}")]
    WorkingDirectory(std::io::Error),
}

impl AppConfig {
    const DEFAULT_BIND: &'static str = "127.0.0.1:8080";
    const DEFAULT_MOPIDY_RPC: &'static str = "http://127.0.0.1:6680/mopidy/rpc";
    const DEFAULT_AUDIO_MODE_CMD: &'static str = "./scripts/audio-mode";
    const DEFAULT_PLAYLIST_CMD: &'static str = "./scripts/playlist-from-list";
    const DEFAULT_TIMEOUT_SECS: u64 = 10;

    pub fn from_env() -> Result<Self, ConfigError> {
        let bind_raw = env::var("HAUSKI_BACKEND_BIND")
            .or_else(|_| env::var("HAUSKI_BIND"))
            .unwrap_or_else(|_| Self::DEFAULT_BIND.to_string());
        let bind_addr: SocketAddr = bind_raw
            .parse()
            .map_err(|_| ConfigError::InvalidBindAddress(bind_raw.clone()))?;

        let mopidy_rpc_url = resolve_mopidy_rpc_url()?;

        let workdir = match env::var("HAUSKI_SCRIPT_WORKDIR") {
            Ok(value) => PathBuf::from(value),
            Err(_) => std::env::current_dir().map_err(ConfigError::WorkingDirectory)?,
        };

        let audio_mode_script = ScriptConfig {
            program: PathBuf::from(
                env::var("HAUSKI_AUDIO_MODE_CMD")
                    .unwrap_or_else(|_| Self::DEFAULT_AUDIO_MODE_CMD.to_string()),
            ),
        };

        let playlist_script = ScriptConfig {
            program: PathBuf::from(
                env::var("HAUSKI_PLAYLIST_FROM_LIST_CMD")
                    .or_else(|_| env::var("HAUSKI_PLAYLIST_CMD"))
                    .unwrap_or_else(|_| Self::DEFAULT_PLAYLIST_CMD.to_string()),
            ),
        };

        let timeout_ms: u64 = env::var("HAUSKI_COMMAND_TIMEOUT_MS")
            .ok()
            .and_then(|raw| raw.parse().ok())
            .unwrap_or(Self::DEFAULT_TIMEOUT_SECS * 1000);

        let check_mopidy_health = env::var("HAUSKI_CHECK_MOPIDY_HEALTH")
            .ok()
            .and_then(|raw| parse_bool(&raw))
            .unwrap_or(true);

        Ok(Self {
            bind_addr,
            mopidy_rpc_url,
            audio_mode_script,
            playlist_script,
            script_workdir: workdir,
            command_timeout: Duration::from_millis(timeout_ms),
            check_mopidy_health,
        })
    }
}

fn resolve_mopidy_rpc_url() -> Result<Url, ConfigError> {
    let direct = env::var("HAUSKI_MOPIDY_RPC_URL")
        .or_else(|_| env::var("MOPIDY_RPC_URL"))
        .ok();

    if let Some(raw) = direct {
        return Url::parse(&raw).map_err(|_| ConfigError::InvalidMopidyUrl(raw));
    }

    if let Ok(base) = env::var("MOPIDY_HTTP_URL") {
        if let Ok(mut url) = Url::parse(&base) {
            url.set_path("/mopidy/rpc");
            return Ok(url);
        } else {
            return Err(ConfigError::InvalidMopidyUrl(base));
        }
    }

    Url::parse(AppConfig::DEFAULT_MOPIDY_RPC)
        .map_err(|_| ConfigError::InvalidMopidyUrl(AppConfig::DEFAULT_MOPIDY_RPC.to_string()))
}

fn parse_bool(raw: &str) -> Option<bool> {
    match raw.to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}
```

### 📄 crates/backend/src/discover.rs

**Größe:** 7 KB | **md5:** `d55bdcbc99edcc49a76c8d6cda343833`

```rust
use std::collections::HashSet;

use serde_json::Value;
use tracing::instrument;

use crate::error::AppError;
use crate::models::{SimilarResponse, SimilarTrack};
use crate::mopidy::MopidyClient;

#[instrument(skip(mopidy))]
pub async fn similar_tracks(
    mopidy: &dyn MopidyClient,
    seed: &str,
    limit: Option<usize>,
) -> Result<SimilarResponse, AppError> {
    let seed_track_value = mopidy.lookup_track(seed).await?;
    let seed_track_value =
        seed_track_value.ok_or_else(|| AppError::bad_request("seed track not found in Mopidy"))?;

    let seed_track = build_track(&seed_track_value)
        .ok_or_else(|| AppError::internal("seed track missing uri"))?;

    let query = build_query(&seed_track_value)
        .ok_or_else(|| AppError::internal("unable to derive search query from seed track"))?;

    let search_results = mopidy.search_any(&query).await?;
    let mut seen: HashSet<String> = HashSet::new();
    seen.insert(seed_track.uri.clone());
    let mut collected: Vec<SimilarTrack> = Vec::new();

    let target_limit = limit.unwrap_or(10);
    if target_limit == 0 {
        return Ok(SimilarResponse {
            seed: seed_track,
            query,
            tracks: collected,
        });
    }

    for backend in search_results {
        if let Some(tracks) = backend.get("tracks").and_then(Value::as_array) {
            for track in tracks {
                let Some(candidate) = build_track(track) else {
                    continue;
                };
                if !seen.insert(candidate.uri.clone()) {
                    continue;
                }
                collected.push(candidate);
                if collected.len() >= target_limit {
                    break;
                }
            }
        }
        if collected.len() >= target_limit {
            break;
        }
    }

    Ok(SimilarResponse {
        seed: seed_track,
        query,
        tracks: collected,
    })
}

fn build_query(track: &Value) -> Option<String> {
    let name = track.get("name").and_then(Value::as_str)?.trim();
    if name.is_empty() {
        return None;
    }

    let artists = track
        .get("artists")
        .and_then(Value::as_array)
        .and_then(|arr| arr.first())
        .and_then(|artist| artist.get("name"))
        .and_then(Value::as_str)
        .map(|s| s.trim())
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string());

    let query = if let Some(artist) = artists {
        format!("{artist} {name}")
    } else {
        name.to_string()
    };

    Some(query)
}

fn build_track(track: &Value) -> Option<SimilarTrack> {
    let uri = track.get("uri").and_then(Value::as_str)?.to_string();
    let name = track
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let album = track
        .get("album")
        .and_then(|album| album.get("name"))
        .and_then(Value::as_str)
        .map(|s| s.to_string());
    let artists = track
        .get("artists")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|artist| artist.get("name").and_then(Value::as_str))
                .map(|name| name.to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Some(SimilarTrack {
        uri,
        name,
        album,
        artists,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use serde_json::json;
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    struct StubMopidy {
        lookup: Option<Value>,
        search: Vec<Value>,
        queries: Arc<Mutex<Vec<String>>>,
    }

    impl StubMopidy {
        fn new(lookup: Option<Value>, search: Vec<Value>) -> Self {
            Self {
                lookup,
                search,
                queries: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    #[async_trait]
    impl MopidyClient for StubMopidy {
        async fn proxy(&self, _payload: Value) -> Result<Value, AppError> {
            unreachable!("proxy should not be invoked directly in tests");
        }

        async fn lookup_track(&self, _uri: &str) -> Result<Option<Value>, AppError> {
            Ok(self.lookup.clone())
        }

        async fn search_any(&self, query: &str) -> Result<Vec<Value>, AppError> {
            self.queries.lock().unwrap().push(query.to_string());
            Ok(self.search.clone())
        }
    }

    #[tokio::test]
    async fn similar_tracks_returns_empty_when_limit_is_zero() {
        let seed = json!({
            "uri": "qobuz:track:seed",
            "name": "Seed",
            "artists": [{"name": "Artist"}]
        });
        let mopidy = StubMopidy::new(Some(seed), vec![json!({"tracks": []})]);

        let response = similar_tracks(&mopidy, "qobuz:track:seed", Some(0))
            .await
            .expect("response");

        assert!(response.tracks.is_empty());
        assert_eq!(response.query, "Artist Seed");
        let recorded = mopidy.queries.lock().unwrap().clone();
        assert_eq!(recorded, vec!["Artist Seed".to_string()]);
    }

    #[tokio::test]
    async fn similar_tracks_skips_seed_and_duplicates() {
        let seed_track = json!({
            "uri": "qobuz:track:seed",
            "name": "Seed",
            "artists": [{"name": "Artist"}]
        });
        let results = json!({
            "tracks": [
                {
                    "uri": "qobuz:track:seed",
                    "name": "Seed",
                    "artists": [{"name": "Artist"}]
                },
                {
                    "uri": "qobuz:track:1",
                    "name": "Track One",
                    "artists": [{"name": "Artist"}]
                },
                {
                    "uri": "qobuz:track:1",
                    "name": "Track One",
                    "artists": [{"name": "Artist"}]
                },
                {
                    "uri": "qobuz:track:2",
                    "name": "Track Two",
                    "artists": [{"name": "Artist"}],
                    "album": {"name": "Album"}
                }
            ]
        });
        let mopidy = StubMopidy::new(Some(seed_track), vec![results]);

        let response = similar_tracks(&mopidy, "qobuz:track:seed", Some(10))
            .await
            .expect("response");

        let uris: Vec<_> = response
            .tracks
            .iter()
            .map(|track| track.uri.as_str())
            .collect();
        assert_eq!(uris, vec!["qobuz:track:1", "qobuz:track:2"]);
        assert_eq!(response.tracks[1].album.as_deref(), Some("Album"));
    }
}
```

### 📄 crates/backend/src/error.rs

**Größe:** 1 KB | **md5:** `172ee822ad76d7a9ed0c31b679e913f3`

```rust
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    Upstream(String),
    #[error("{0}")]
    Internal(String),
}

impl AppError {
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::BadRequest(message.into())
    }

    pub fn upstream(message: impl Into<String>) -> Self {
        Self::Upstream(message.into())
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::BadRequest(message) => (StatusCode::BAD_REQUEST, message.as_str()),
            AppError::Upstream(message) => (StatusCode::BAD_GATEWAY, message.as_str()),
            AppError::Internal(message) => (StatusCode::INTERNAL_SERVER_ERROR, message.as_str()),
        };

        let payload = json!({
            "error": message,
        });

        (status, Json(payload)).into_response()
    }
}
```

### 📄 crates/backend/src/handlers.rs

**Größe:** 3 KB | **md5:** `149524e98f36479866f570167fcd97cc`

```rust
use axum::extract::{Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::Value;
use tower_http::trace::TraceLayer;
use tracing::instrument;

use crate::error::AppError;
use crate::models::{
    CommandResponse, HealthResponse, ModeGetResponse, ModeSetRequest, MopidyHealth,
    PlaylistRequest, PlaylistResponse, SimilarQuery, SimilarResponse,
};
use crate::{discover, scripts, AppState};

pub fn app_routes(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/rpc", post(proxy_rpc))
        .route("/mode", get(get_mode).post(set_mode))
        .route("/playlists/from-list", post(playlist_from_list))
        .route("/discover/similar", get(discover_similar))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}

#[instrument(skip(state))]
pub async fn health(State(state): State<AppState>) -> Result<Json<HealthResponse>, AppError> {
    let mut overall = "ok";
    let mopidy_status = if state.config.check_mopidy_health {
        match state.mopidy.health_check().await {
            Ok(_) => Some(MopidyHealth {
                status: "ok",
                detail: None,
            }),
            Err(err) => {
                overall = "degraded";
                Some(MopidyHealth {
                    status: "error",
                    detail: Some(err),
                })
            }
        }
    } else {
        None
    };

    Ok(Json(HealthResponse {
        status: overall,
        version: env!("CARGO_PKG_VERSION"),
        mopidy: mopidy_status,
    }))
}

#[instrument(skip(state, payload))]
pub async fn proxy_rpc(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, AppError> {
    let response = state.mopidy.proxy(payload).await?;
    Ok(Json(response))
}

#[instrument(skip(state))]
pub async fn get_mode(State(state): State<AppState>) -> Result<Json<ModeGetResponse>, AppError> {
    let output = scripts::show_mode(&state.config).await?;
    let trimmed = output.stdout.trim().to_string();
    let inferred = crate::models::AudioMode::infer(&trimmed);

    Ok(Json(ModeGetResponse {
        value: trimmed,
        mode: inferred,
    }))
}

#[instrument(skip(state, body))]
pub async fn set_mode(
    State(state): State<AppState>,
    Json(body): Json<ModeSetRequest>,
) -> Result<Json<CommandResponse>, AppError> {
    let output = scripts::set_mode(&state.config, &body).await?;
    Ok(Json(CommandResponse {
        stdout: output.stdout,
        stderr: output.stderr,
    }))
}

#[instrument(skip(state, body))]
pub async fn playlist_from_list(
    State(state): State<AppState>,
    Json(body): Json<PlaylistRequest>,
) -> Result<Json<PlaylistResponse>, AppError> {
    let output = scripts::playlist_from_list(&state.config, &body).await?;
    Ok(Json(PlaylistResponse {
        stdout: output.stdout,
        stderr: output.stderr,
    }))
}

#[instrument(skip(state, params))]
pub async fn discover_similar(
    State(state): State<AppState>,
    Query(params): Query<SimilarQuery>,
) -> Result<Json<SimilarResponse>, AppError> {
    let response =
        discover::similar_tracks(state.mopidy.as_ref(), &params.seed, params.limit).await?;

    Ok(Json(response))
}
```

### 📄 crates/backend/src/lib.rs

**Größe:** 943 B | **md5:** `bf7a64e4c3e1695d5a7c5edb7e81d062`

```rust
pub mod config;
mod discover;
mod error;
mod handlers;
mod models;
mod mopidy;
mod scripts;

pub use error::AppError;
pub use models::AudioMode;
pub use mopidy::{HttpMopidyClient, MopidyClient};

use axum::Router;
use std::sync::Arc;

use crate::config::AppConfig;
use crate::handlers::app_routes;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub mopidy: Arc<dyn MopidyClient>,
}

pub fn build_router(config: AppConfig) -> Router {
    let config = Arc::new(config);
    let mopidy_client = Arc::new(HttpMopidyClient::new(
        reqwest::Client::new(),
        config.mopidy_rpc_url.clone(),
    )) as Arc<dyn MopidyClient>;

    app_routes(AppState {
        config,
        mopidy: mopidy_client,
    })
}

pub fn build_router_with_mopidy(config: AppConfig, mopidy_client: Arc<dyn MopidyClient>) -> Router {
    app_routes(AppState {
        config: Arc::new(config),
        mopidy: mopidy_client,
    })
}
```

### 📄 crates/backend/src/main.rs

**Größe:** 1 KB | **md5:** `d2e96dd1bba271aecb368c9d7e6206e3`

```rust
use hauski_backend::build_router;
use hauski_backend::config::AppConfig;
use tokio::net::TcpListener;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .compact()
        .init();

    let config = match AppConfig::from_env() {
        Ok(cfg) => cfg,
        Err(err) => {
            error!("failed to load configuration: {err}");
            std::process::exit(1);
        }
    };

    let bind_addr = config.bind_addr;
    let listener = match TcpListener::bind(bind_addr).await {
        Ok(listener) => listener,
        Err(err) => {
            error!("failed to bind to {bind_addr}: {err}");
            std::process::exit(1);
        }
    };

    info!("listening on {bind_addr}");

    if let Err(err) = axum::serve(listener, build_router(config)).await {
        error!("server error: {err}");
    }
}
```

### 📄 crates/backend/src/models.rs

**Größe:** 3 KB | **md5:** `ef6ffcbb44062761bd6eebedfe5a5fab`

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum AudioMode {
    Pulse,
    Alsa,
}

impl AudioMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            AudioMode::Pulse => "pulse",
            AudioMode::Alsa => "alsa",
        }
    }

    pub fn infer(raw: &str) -> Option<Self> {
        let normalized = raw.to_ascii_lowercase();
        if normalized.contains("alsa") {
            Some(AudioMode::Alsa)
        } else if normalized.contains("pulse") {
            Some(AudioMode::Pulse)
        } else {
            None
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ModeSetRequest {
    pub mode: AudioMode,
    #[serde(default)]
    pub restart: bool,
    #[serde(default)]
    pub config: Option<String>,
    #[serde(default)]
    pub alsa_output: Option<String>,
    #[serde(default)]
    pub pulse_output: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ModeGetResponse {
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<AudioMode>,
}

#[derive(Debug, Serialize)]
pub struct CommandResponse {
    pub stdout: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub stderr: String,
}

#[derive(Debug, Deserialize)]
pub struct PlaylistRequest {
    pub name: String,
    pub uris: Vec<String>,
    #[serde(default)]
    pub scheme: Option<String>,
    #[serde(default)]
    pub replace: bool,
    #[serde(default)]
    pub dry_run: bool,
}

#[derive(Debug, Serialize)]
pub struct PlaylistResponse {
    pub stdout: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub stderr: String,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub version: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mopidy: Option<MopidyHealth>,
}

#[derive(Debug, Serialize)]
pub struct MopidyHealth {
    pub status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SimilarQuery {
    pub seed: String,
    #[serde(default)]
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize, Clone)]
pub struct SimilarTrack {
    pub uri: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub album: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub artists: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct SimilarResponse {
    pub seed: SimilarTrack,
    pub query: String,
    pub tracks: Vec<SimilarTrack>,
}
```

### 📄 crates/backend/src/mopidy.rs

**Größe:** 8 KB | **md5:** `e60d5c13beb348b2a68769d70958a030`

```rust
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
```

### 📄 crates/backend/src/scripts.rs

**Größe:** 4 KB | **md5:** `decdd8cda7882ee0143d8c7fc0a5754f`

```rust
use std::process::Stdio;

use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::time;
use tracing::instrument;

use crate::config::{AppConfig, ScriptConfig};
use crate::error::AppError;
use crate::models::{ModeSetRequest, PlaylistRequest};

#[derive(Debug)]
pub struct ScriptOutput {
    pub stdout: String,
    pub stderr: String,
}

pub async fn show_mode(config: &AppConfig) -> Result<ScriptOutput, AppError> {
    run_script(
        &config.audio_mode_script,
        &["show".to_string()],
        config,
        None,
    )
    .await
}

pub async fn set_mode(
    config: &AppConfig,
    request: &ModeSetRequest,
) -> Result<ScriptOutput, AppError> {
    let mut args = vec![request.mode.as_str().to_string()];

    if let Some(path) = &request.config {
        args.push("--config".to_string());
        args.push(path.clone());
    }

    if let Some(value) = &request.alsa_output {
        args.push("--alsa-output".to_string());
        args.push(value.clone());
    }

    if let Some(value) = &request.pulse_output {
        args.push("--pulse-output".to_string());
        args.push(value.clone());
    }

    if request.restart {
        args.push("--restart".to_string());
    }

    run_script(&config.audio_mode_script, &args, config, None).await
}

pub async fn playlist_from_list(
    config: &AppConfig,
    request: &PlaylistRequest,
) -> Result<ScriptOutput, AppError> {
    if request.uris.is_empty() {
        return Err(AppError::bad_request("uris must not be empty"));
    }

    let mut args = vec![request.name.clone(), "--input".to_string(), "-".to_string()];

    if let Some(scheme) = &request.scheme {
        args.push("--scheme".to_string());
        args.push(scheme.clone());
    }

    if request.replace {
        args.push("--replace".to_string());
    }

    if request.dry_run {
        args.push("--dry-run".to_string());
    }

    let stdin_payload = request.uris.join("\n") + "\n";

    run_script(&config.playlist_script, &args, config, Some(stdin_payload)).await
}

#[instrument(skip(config, args, input))]
async fn run_script(
    script: &ScriptConfig,
    args: &[String],
    config: &AppConfig,
    input: Option<String>,
) -> Result<ScriptOutput, AppError> {
    let program = script.resolve_with(&config.script_workdir);
    let mut command = Command::new(&program);
    command.args(args);
    command.current_dir(&config.script_workdir);
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    if input.is_some() {
        command.stdin(Stdio::piped());
    } else {
        command.stdin(Stdio::null());
    }

    let mut child = command
        .spawn()
        .map_err(|err| AppError::internal(format!("failed to spawn {program:?}: {err}")))?;

    if let Some(payload) = input {
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(payload.as_bytes()).await.map_err(|err| {
                AppError::internal(format!("failed to write to {program:?}: {err}"))
            })?;
        }
    }

    let output = time::timeout(config.command_timeout, child.wait_with_output())
        .await
        .map_err(|_| {
            AppError::internal(format!(
                "command {program:?} timed out after {:?}",
                config.command_timeout
            ))
        })
        .and_then(|result| {
            result.map_err(|err| AppError::internal(format!("command error: {err}")))
        })?;

    if !output.status.success() {
        return Err(AppError::internal(format!(
            "command {program:?} exited with status {status}",
            status = output.status
        )));
    }

    Ok(ScriptOutput {
        stdout: String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .to_string(),
        stderr: String::from_utf8_lossy(&output.stderr)
            .trim_end()
            .to_string(),
    })
}
```

