### 📄 merges/hausKI-audio_merge_2510262237__.github_workflows.md

**Größe:** 7 KB | **md5:** `02ed7a8cde79ef05829ea0f2747aeb2c`

```markdown
### 📄 .github/workflows/docs-ci.yml

**Größe:** 444 B | **md5:** `11d44789bd185729bc8482554888eb79`

```yaml
---
name: Docs CI
"on": [push, pull_request]
permissions:
  contents: read
jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: DavidAnson/markdownlint-cli2-action@v16
        with:
          globs: |
            **/*.md
            !**/node_modules/**
      - name: Lint YAML
        uses: ibiqlik/action-yamllint@v3
        with:
          file_or_dir: ".github/**/*.yml"
          strict: true
```

### 📄 .github/workflows/rust-ci.yml

**Größe:** 611 B | **md5:** `5218f271dce2347e7b4f6b1e3b0b7828`

```yaml
---
name: rust-ci
permissions:
  contents: read

"on":
  push:
    branches:
      - main
  pull_request:
    paths:
      - 'Cargo.toml'
      - 'Cargo.lock'
      - 'crates/backend/**'
      - 'Justfile'

jobs:
  backend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt
      - name: Cargo fmt
        run: cargo fmt --all -- --check
      - name: Cargo clippy
        run: cargo clippy --workspace --all-targets -- -D warnings
      - name: Cargo test
        run: cargo test --workspace
```

### 📄 .github/workflows/validate-audio-events.yml

**Größe:** 749 B | **md5:** `3c52ec0255c2127c70bd6629a1a178c2`

```yaml
---
name: validate audio events
permissions:
  contents: read

"on":
  push:
    paths:
      - "export/**"
      - "fixtures/**"
      - ".github/workflows/validate-audio-events.yml"
  pull_request:
  workflow_dispatch:

jobs:
  validate-jsonl:
    name: "audio.events.jsonl schema check"
    uses: heimgewebe/metarepo/.github/workflows/reusable-validate-jsonl.yml@contracts-v1
    strategy:
      fail-fast: false
      matrix:
        file:
          - export/audio.events.jsonl
          - fixtures/audio/events.jsonl
    with:
      jsonl_path: ${{ matrix.file }}
      schema_url: >-
        https://raw.githubusercontent.com/heimgewebe/metarepo/contracts-v1/contracts/audio.events.schema.json
      strict: false
      validate_formats: true
```

### 📄 .github/workflows/wgx-guard.yml

**Größe:** 5 KB | **md5:** `e2bf67d3ffa6b5ca2b6f2607a132fc31`

```yaml
---
name: WGX Guard
permissions:
  contents: read

"on":
  push:
    branches: [main]
  pull_request:

jobs:
  guard:
    name: Validate WGX setup
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Ensure required files exist
        run: |
          test -f pyproject.toml || { echo "Missing pyproject.toml"; exit 1; }
          test -f Justfile || { echo "Missing Justfile"; exit 1; }
          test -f .wgx/profile.yml || { echo "Missing .wgx/profile.yml"; exit 1; }

      - name: Set up Python
        uses: actions/setup-python@v5
        with:
          python-version: '3.12'

      - name: Validate pyproject.toml
        run: |
          python - <<'PY'
          from pathlib import Path
          import tomllib

          expected = {
              "name": "hauski-audio",
              "version": "0.1.0",
              "description": (
                  "HausKI Audio Layer – MOTU, Lake, Qobuz, Local Sound Stack"
              ),
              "authors": [
                  {"name": "Alexander Mohr", "email": "alexdermohr@gmail.com"}
              ],
              "requires-python": ">=3.10",
          }
          uv_expected = {
              "sync": True,
              "dev-dependencies": ["pytest", "ruff", "black", "mypy"],
          }
          build_expected = {
              "requires": ["setuptools"],
              "build-backend": "setuptools.build_meta",
          }

          data = tomllib.loads(
              Path("pyproject.toml").read_text(encoding="utf-8")
          )

          project = data.get("project")
          assert project, "[project] table missing"
          for key, value in expected.items():
              assert project.get(key) == value, (
                  f"project.{key} should be {value!r}"
              )

          tool = data.get("tool", {})
          uv = tool.get("uv")
          assert uv, "[tool.uv] table missing"
          for key, value in uv_expected.items():
              assert uv.get(key) == value, f"tool.uv.{key} should be {value!r}"

          build = data.get("build-system")
          assert build, "[build-system] table missing"
          for key, value in build_expected.items():
              assert build.get(key) == value, (
                  f"build-system.{key} should be {value!r}"
              )
          PY

      - name: Install PyYAML
        run: python -m pip install --upgrade pip pyyaml

      - name: Validate .wgx/profile.yml
        run: |
          python - <<'PY'
          from pathlib import Path
          import yaml

          data = yaml.safe_load(
              Path('.wgx/profile.yml').read_text(encoding='utf-8')
          )

          assert data.get('profile') == 'hauski-audio', (
              "profile must be 'hauski-audio'"
          )
          assert data.get('description') == (
              'Local audio orchestration layer for HausKI'
          ), "description mismatch"
          assert data.get('lang') == 'python', "lang must be 'python'"
          assert data.get('wgx-version') == '>=0.3', (
              "wgx-version must be '>=0.3'"
          )

          meta = data.get('meta') or {}
          assert meta.get('repo') == 'alexdermohr/hauski-audio', (
              "meta.repo mismatch"
          )
          assert meta.get('maintainer') == 'alexdermohr@gmail.com', (
              "meta.maintainer mismatch"
          )
          assert meta.get('tags') == [
              'audio', 'motu', 'qobuz', 'hauski', 'wgx'
          ], "meta.tags mismatch"

          env = data.get('env') or {}
          assert env.get('PYTHONUNBUFFERED') == '1', (
              "env.PYTHONUNBUFFERED mismatch"
          )
          assert env.get('UV_PIP_VERSION') == '24.0', (
              "env.UV_PIP_VERSION mismatch"
          )
          PY

      - name: Validate Justfile content
        run: |
          python - <<'PY'
          from pathlib import Path

          content = Path('Justfile').read_text(encoding='utf-8')
          required_chunks = [
              'set shell := ["bash", "-cu"]',
              ('default:\n    @echo "🧵 HausKI Audio Layer – choose a target '
               '(lint, test, run, doctor)"'),
              'lint:\n    uv run ruff check .\n    uv run black --check .',
              'test:\n    uv run pytest -q || echo "⚠️ no tests yet"',
              'doctor:\n    @echo "🔎 Environment check"',
          ]
          for chunk in required_chunks:
              assert chunk in content, (
                  f"Justfile missing required block: {chunk!r}"
              )
          PY
```
```

### 📄 merges/hausKI-audio_merge_2510262237__.wgx.md

**Größe:** 331 B | **md5:** `0a2bec4999c0fbee124d0fa17cc548e7`

```markdown
### 📄 .wgx/profile.yml

**Größe:** 225 B | **md5:** `5a8a114edb86252668123007969f24ee`

```yaml
# hauski-audio – WGX Profil
environment: development
features:
  - core
# Optionale Felder für spätere Fleet-Runs:
# repo:
#   role: app
#   tags: [audio, mopidy, backend]
# ci:
#   smoke: true
tool:
  uv:
    sync: true
```
```

### 📄 merges/hausKI-audio_merge_2510262237__crates_backend.md

**Größe:** 744 B | **md5:** `8d411c0e74afce261fdd4ee669ef7483`

```markdown
### 📄 crates/backend/Cargo.toml

**Größe:** 629 B | **md5:** `97474e3fe0d796e6e4e7e3e00e693e50`

```toml
[package]
name = "hauski-backend"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = { version = "0.7", features = ["macros"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }
tower-http = { version = "0.5", features = ["trace"] }
reqwest = { version = "0.11", features = ["json"] }
thiserror = "1"
dotenvy = "0.15"
url = "2"
async-trait = "0.1"

[dev-dependencies]
tempfile = "3"
tower = { version = "0.4", features = ["util"] }
http-body-util = "0.1"
http = "0.2"
```
```

### 📄 merges/hausKI-audio_merge_2510262237__crates_backend_src.md

**Größe:** 32 KB | **md5:** `49ead0d4c2bff0709638a0cfc5dd381e`

```markdown
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

<<TRUNCATED: max_file_lines=800>>
```

### 📄 merges/hausKI-audio_merge_2510262237__crates_backend_tests.md

**Größe:** 12 KB | **md5:** `411055d453a5389ba6fc0341ef5b62c8`

```markdown
### 📄 crates/backend/tests/http.rs

**Größe:** 11 KB | **md5:** `bf6e66001912dcf917ba9570a339867c`

```rust
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{json, Map, Value};
use tempfile::TempDir;
use tower::ServiceExt;
use url::Url;

use hauski_backend::config::{AppConfig, ScriptConfig};
use hauski_backend::{AppError, AudioMode, MopidyClient};

fn write_script(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, content).expect("write script");
    #[cfg(unix)]
    {
        let mut perms = fs::metadata(&path).expect("metadata").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).expect("permissions");
    }
    path
}

fn test_config(dir: &TempDir) -> AppConfig {
    test_config_with(dir, Url::parse("http://127.0.0.1:6680/mopidy/rpc").unwrap())
}

fn test_config_with(dir: &TempDir, mopidy_rpc_url: Url) -> AppConfig {
    AppConfig {
        bind_addr: "127.0.0.1:0".parse().unwrap(),
        mopidy_rpc_url,
        audio_mode_script: ScriptConfig {
            program: dir.path().join("audio-mode"),
        },
        playlist_script: ScriptConfig {
            program: dir.path().join("playlist-from-list"),
        },
        script_workdir: dir.path().to_path_buf(),
        command_timeout: Duration::from_secs(2),
        check_mopidy_health: false,
    }
}

struct FakeMopidy {
    calls: Arc<Mutex<Vec<String>>>,
    lookup: Value,
    search: Value,
    health_error: Option<String>,
}

impl FakeMopidy {
    fn new(calls: Arc<Mutex<Vec<String>>>, lookup: Value, search: Value) -> Self {
        Self {
            calls,
            lookup,
            search,
            health_error: None,
        }
    }

    fn with_health_error(mut self, error: impl Into<String>) -> Self {
        self.health_error = Some(error.into());
        self
    }
}

#[async_trait]
impl MopidyClient for FakeMopidy {
    async fn proxy(&self, payload: Value) -> Result<Value, AppError> {
        let method = payload
            .get("method")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        self.calls.lock().unwrap().push(method.clone());

        let id = payload.get("id").cloned().unwrap_or(Value::from(1));
        let mut response = Map::new();
        response.insert("jsonrpc".into(), Value::String("2.0".into()));
        response.insert("id".into(), id);

        match method.as_str() {
            "core.library.lookup" => {
                response.insert("result".into(), self.lookup.clone());
            }
            "core.library.search" => {
                response.insert("result".into(), self.search.clone());
            }
            "core.playback.get_state" => {
                if let Some(error) = &self.health_error {
                    let mut error_obj = Map::new();
                    error_obj.insert("message".into(), Value::String(error.clone()));
                    response.insert("error".into(), Value::Object(error_obj));
                } else {
                    response.insert("result".into(), Value::String("stopped".into()));
                }
            }
            other => return Err(AppError::internal(format!("unexpected method {other}"))),
        }

        Ok(Value::Object(response))
    }
}

#[tokio::test]
async fn health_endpoint_ok_without_mopidy() {
    let dir = TempDir::new().unwrap();
    let audio_script = "#!/usr/bin/env bash\nset -euo pipefail\nif [[ \"$1\" == \"show\" ]]; then\n  echo \"pulsesink\"\nelse\n  echo \"mode:$1\"\nfi\n";
    write_script(&dir, "audio-mode", audio_script);
    let playlist_script = "#!/usr/bin/env bash\nset -euo pipefail\necho \"playlist:$1\"\ncat -\n";
    write_script(&dir, "playlist-from-list", playlist_script);

    let app = hauski_backend::build_router(test_config(&dir));

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn health_endpoint_reports_mopidy_error() {
    let dir = TempDir::new().unwrap();
    let audio_script = "#!/usr/bin/env bash\nset -euo pipefail\nif [[ \"$1\" == \"show\" ]]; then\n  echo \"pulsesink\"\nelse\n  echo \"mode:$1\"\nfi\n";
    write_script(&dir, "audio-mode", audio_script);
    let playlist_script = "#!/usr/bin/env bash\nset -euo pipefail\necho \"playlist:$1\"\ncat -\n";
    write_script(&dir, "playlist-from-list", playlist_script);

    let calls = Arc::new(Mutex::new(Vec::new()));
    let mopidy_stub: Arc<dyn MopidyClient> = Arc::new(
        FakeMopidy::new(calls.clone(), json!([]), json!([])).with_health_error("pipewire down"),
    );

    let mut config = test_config(&dir);
    config.check_mopidy_health = true;

    let app = hauski_backend::build_router_with_mopidy(config, mopidy_stub);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["status"], "degraded");
    assert_eq!(body["mopidy"]["status"], "error");
    assert_eq!(body["mopidy"]["detail"], "pipewire down");

    let recorded = calls.lock().unwrap().clone();
    assert_eq!(recorded, vec!["core.playback.get_state".to_string()]);
}

#[tokio::test]
async fn mode_endpoints_invoke_script() {
    let dir = TempDir::new().unwrap();
    let audio_script = "#!/usr/bin/env bash\nset -euo pipefail\nif [[ \"$1\" == \"show\" ]]; then\n  echo \"pulsesink\"\nelse\n  echo \"mode:$1\"\nfi\n";
    write_script(&dir, "audio-mode", audio_script);
    let playlist_script = "#!/usr/bin/env bash\nset -euo pipefail\necho \"playlist:$1\"\ncat -\n";
    write_script(&dir, "playlist-from-list", playlist_script);

    let app = hauski_backend::build_router(test_config(&dir));

    let get_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/mode")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get_response.status(), StatusCode::OK);
    let body = get_response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["value"], "pulsesink");
    assert_eq!(json["mode"], AudioMode::Pulse.as_str());

    let payload = serde_json::json!({"mode": "alsa"});
    let post_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/mode")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(post_response.status(), StatusCode::OK);
    let body = post_response
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["stdout"], "mode:alsa");
}

#[tokio::test]
async fn playlist_endpoint_streams_uris() {
    let dir = TempDir::new().unwrap();
    let audio_script = "#!/usr/bin/env bash\nset -euo pipefail\nif [[ \"$1\" == \"show\" ]]; then\n  echo \"pulsesink\"\nelse\n  echo \"mode:$1\"\nfi\n";
    write_script(&dir, "audio-mode", audio_script);
    let playlist_script = "#!/usr/bin/env bash\nset -euo pipefail\necho \"playlist:$1\"\ncat -\n";
    write_script(&dir, "playlist-from-list", playlist_script);

    let app = hauski_backend::build_router(test_config(&dir));
    let payload = serde_json::json!({
        "name": "Test",
        "uris": ["qobuz:track:1", "qobuz:track:2"],
        "replace": true,
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/playlists/from-list")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["stdout"].as_str().unwrap().contains("playlist:Test"));
    assert!(json["stdout"].as_str().unwrap().contains("qobuz:track:1"));
}

#[tokio::test]
async fn discover_similar_returns_tracks() {
    let dir = TempDir::new().unwrap();
    let audio_script = "#!/usr/bin/env bash\nset -euo pipefail\nif [[ \"$1\" == \"show\" ]]; then\n  echo \"pulsesink\"\nelse\n  echo \"mode:$1\"\nfi\n";
    write_script(&dir, "audio-mode", audio_script);
    let playlist_script = "#!/usr/bin/env bash\nset -euo pipefail\necho \"playlist:$1\"\ncat -\n";
    write_script(&dir, "playlist-from-list", playlist_script);
    let calls = Arc::new(Mutex::new(Vec::new()));
    let mopidy_stub: Arc<dyn MopidyClient> = Arc::new(FakeMopidy::new(
        calls.clone(),
        json!([
            {
                "__model__": "Track",
                "uri": "qobuz:track:seed",
                "name": "Seed Track",
                "artists": [
                    {"name": "Seed Artist"}
                ],
                "album": {"name": "Seed Album"}
            }
        ]),
        json!([
            {
                "tracks": [
                    {
                        "uri": "qobuz:track:1",
                        "name": "Track One",
                        "artists": [
                            {"name": "Seed Artist"}
                        ],
                        "album": {"name": "Album One"}
                    },
                    {
                        "uri": "qobuz:track:seed",
                        "name": "Seed Track",
                        "artists": [
                            {"name": "Seed Artist"}
                        ]
                    }
                ]
            }
        ]),
    ));

    let app = hauski_backend::build_router_with_mopidy(test_config(&dir), mopidy_stub);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/discover/similar?seed=qobuz:track:seed&limit=5")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["seed"]["uri"], "qobuz:track:seed");
    let tracks = json["tracks"].as_array().unwrap();
    assert_eq!(tracks.len(), 1);
    assert_eq!(tracks[0]["uri"], "qobuz:track:1");

    let captured_calls = calls.lock().unwrap().clone();
    assert_eq!(
        captured_calls,
        vec![
            "core.library.lookup".to_string(),
            "core.library.search".to_string(),
        ]
    );
}
```
```

### 📄 merges/hausKI-audio_merge_2510262237__docs.md

**Größe:** 3 KB | **md5:** `834f1b02078126345bc4fbe009316e07`

```markdown
### 📄 docs/ARCHITECTURE.md

**Größe:** 926 B | **md5:** `4829d5ea1c1499183e1e5a5c999b87af`

```markdown
# Architektur (Entwurf)

- **Player-Backend:** Mopidy (Iris-Frontend), Qobuz-Plugin (Hi-Res).
- **Control-Plane:** kleine HTTP-API (axum) als Fassade für Mopidy
  JSON-RPC und lokale Skripte.
  - `/health` prüft Backend + optional Mopidy-RPC.
  - `/rpc` proxyt JSON-RPC Calls zu Mopidy.
  - `/mode` zeigt/ändert den Audio-Modus via `scripts/audio-mode`.
  - `/playlists/from-list` nutzt `scripts/playlist-from-list` (URIs als JSON).
  - `/discover/similar` leitet Mopidy-Suche (Seed-Track → ähnliche Titel) ab.
- **Audio-Pfade:**
  - *Komfort/Alltag:* PipeWire/Pulse → `pulsesink`
  - *Bitperfect/Hi-Res:* ALSA direkt → `alsasink device=hw:<card>,0`
- **Skriptbarkeit:** Shell/ Python-Snippets (Playlist-Builder, Mode-Switch, Recording).
- **UI (künftig):** Minimalpanel (Play/Pause, Volume, Queue, Modus, „echte“ Rate/Format).

> Ziel: reproduzierbares Setup, später portable (Systemd User Service / Docker).
```

### 📄 docs/README_ALSA.md

**Größe:** 551 B | **md5:** `09da467491c2b7082ad67b702574bef4`

```markdown
# Audio-Modi: ALSA vs. Pulse

- **Default = ALSA (bit-perfect):**
  - Mopidy → `alsasink device=hw:<MOTU>,0`
  - PipeWire/Pulse wird gestoppt (kein Mixing, reine Hi-Res-Wiedergabe).
  - Echte Rate/Format siehe `/proc/asound/cardX/pcm0p/sub0/hw_params`.

- **Pulse-Modus (Komfort):**
  - Mopidy → `pulsesink`
  - PipeWire/Pulse aktiv (System-Sounds, App-Lautstärken verfügbar).
  - Kann Resampling/Processing enthalten.

## Umschalten

```bash
./scripts/audio-mode alsa   # Bit-perfect, exklusiv
./scripts/audio-mode pulse  # Komfort, Mixing
```
```

### 📄 docs/io-contracts.md

**Größe:** 193 B | **md5:** `bde59ecc940585c1ba1906f8c82d9c7e`

```markdown
# IO-Contracts (Skizze)

- **Input:** WAV/FLAC/MP3; Mono/Stereo, 44.1–192 kHz.
- **Output:** WAV/FLAC, Normalisierung optional.
- **Metadaten:** Titel, Quelle, Zeitstempel (ISO-8601), Pfade.
```

### 📄 docs/troubleshooting.md

**Größe:** 236 B | **md5:** `2f47495c06e004f241f3e55242aab7eb`

```markdown
# Troubleshooting (kurz)

- Kein Audio? Prüfe ALSA/PipeWire, Gerätelatenz, `arecord -l`, `aplay -l`.
- Knackser: Puffer erhöhen (z. B. `--buffer 4096`), Sample-Rate angleichen.
- Feeds: Webradio-URLs ggf. via `ffprobe` verifizieren.
```

### 📄 docs/vibe-detection.md

**Größe:** 321 B | **md5:** `4d15903aeea53e13ec8fac3e6d981a9e`

```markdown
# Vibe Detection (optional)

Liefert emotionale/kontextuelle Signale (ohne Inhalt zu speichern):

- Prosodie der Stimme (Tempo, Tonfall)
- Musik-Features (Genre/Tempo/Lautstärke)

## Event-Skizze

```json
{
"ts": "...",
"source": "audio.vibe",
"vibe": "fokussiert",
"evidence": ["musik.techno", "speech.rate.low"]
}
```
```
```

### 📄 merges/hausKI-audio_merge_2510262237__docs_adr.md

**Größe:** 4 KB | **md5:** `a25caf97064859ab4fca377aeedabdd7`

```markdown
### 📄 docs/adr/0001-player-backend-mopidy-qobuz.md

**Größe:** 602 B | **md5:** `5fabeabbb6e3c5378a9ad7e9f38b6680`

```markdown
# 0001 – Player-Backend: Mopidy (+ Iris) mit Qobuz Hi-Res

## Kontext

Brauchen Linux-freundliches Backend für Qobuz Hi-Res, skriptbar und UI-fähig.

## Entscheidung

- Verwenden **Mopidy** als Kern (JSON-RPC, MPD, HTTP).
- Frontend **Iris** für Web-UI.
- **mopidy-qobuz-hires** als Qobuz-Backend (App-ID/Secret, Quality=7 standard).

## Konsequenzen

- Stabil auf Linux, Headless tauglich, skriptbar.
- Iris genügt als bequeme UI.
- Qobuz-App-ID/Secret pflegen; Login-Fehler sauber behandeln.

## Nächste Schritte

- Mode-Switch Skript (Pulse ↔ ALSA).
- Playlist-Builder & Recording-Skripte.
```

### 📄 docs/adr/0002-audio-path-pulse-vs-alsa.md

**Größe:** 601 B | **md5:** `81fe84038208bf62997e95cbcbaa4501`

```markdown
# 0002 – Audio-Pfad: Pulse (Komfort) vs. ALSA (Bitperfect)

## Kontext

Zwei konkurrierende Anforderungen: Alltag (System-Sounds, Apps) vs. Hi-Res-Bitperfect.

## Entscheidung

- **Pulse/Komfort:** `output = pulsesink`
- **ALSA/Bitperfect:** `output = alsasink device=hw:<MOTU_M2>,0`
- Umschalter per Script → Mopidy Neustart → Statusanzeige.

## Konsequenzen

- Alltag und Hi-Res koexistieren.
- Wechsel erfordert Mopidy-Restart; Dokumentation & Anzeige der „echten“ Rate nötig.

## Nächste Schritte

- Skript `audio-mode` (setzt Mopidy-Audio-Block).
- UI: aktuelle Rate/Format anzeigen.
```

### 📄 docs/adr/0003-repo-standards-docs-ci.md

**Größe:** 505 B | **md5:** `d2961455abe2a497712a919cddfd4e25`

```markdown
# 0003 – Repo-Standards: Docs, CI, Linting

## Kontext

Frisch angelegtes Repo; wir wollen zügig, aber ordentlich starten.

## Entscheidung

- Struktur: `docs/`, `docs/adr/`, `docs/runbooks/`, `scripts/`, `.github/workflows/`.
- CI minimal: Syntax/Lint für Markdown/YAML; später Rust/Node, wenn Code da ist.
- Editor-Standards: `.editorconfig`, `.gitattributes`.

## Konsequenzen

- Klarer Startpunkt, Konsistenz mit anderen Projekten.
- Anfangs zusätzlicher Overhead; zahlt sich mittelfristig aus.
```

### 📄 docs/adr/0004-recording-pw-record-helper.md

**Größe:** 1 KB | **md5:** `5726632cdfa5293441e1a24b72e8e348`

```markdown
# 0004 – Aufnahme-Flow mit PipeWire `pw-record`

## Kontext

Wir benötigen reproduzierbare Aufnahmen in Hi-Res-Qualität (MOTU M2),
die sowohl Skripting als auch Headless-Betrieb erlauben. Bisherige
Ad-hoc-Kommandos waren fehleranfällig (vergessene Parameter, fehlende
PID-Verwaltung, kein komfortabler Stop).

## Entscheidung

- Verwenden PipeWire `pw-record` als primäres Capture-Tool.
- Verpacken von Aufnahme/Stop in Scripts `rec-start` und `rec-stop` (Python)
  mit PID-File unter `~/.cache/hauski-audio/`.
- Konfigurieren von Sample-Rate, Format, Zielverzeichnis via Parameter oder
  `.env` (`AUDIO_RECORD_*`, `PW_RECORD_BINARY`).
- Ergänzen Runbook mit Workflow, optionalen Flags und Troubleshooting.

## Konsequenzen

- Smoke-Test `just rec-smoke` prüft Skripte ohne Audio.
- Konsistenter CLI-Workflow (Start/Stop, Auto-Dateinamen, Force-Option).
- Einfaches Wiederverwenden per `just rec-start`/`just rec-stop`.
- Trouble-Shooting & Monitoring (pw-top, soxi) dokumentiert.
- Neue Abhängigkeit auf PipeWire (bzw. `pw-record` verfügbar machen).
- Python-Skripte müssen gepflegt werden (Permissions, Signal-Handling).

## Nächste Schritte

- Pytest-Suite (`just test`) pflegen, zusätzliche Cases (z. B. Fehlerpfade) ergänzen.
- Überlegen, ob ALSA-Fallback (`arecord`) nötig wird (z. B. für minimalistische Systeme).
```

### 📄 docs/adr/README.md

**Größe:** 487 B | **md5:** `a758b1c795c9271e47c9e2bb6c08afd8`

```markdown
# Architecture Decision Records (ADR)

Konzentrierte Entscheidungen mit Kontext & Konsequenzen.

- [0001-player-backend-mopidy-qobuz.md](0001-player-backend-mopidy-qobuz.md)
- [0002-audio-path-pulse-vs-alsa.md](0002-audio-path-pulse-vs-alsa.md)
- [0003-repo-standards-docs-ci.md](0003-repo-standards-docs-ci.md)
- [0004-recording-pw-record-helper.md](0004-recording-pw-record-helper.md)

## Vorlage

- Titel in Imperativ
- Kontext → Entscheidung → Konsequenzen → Nächste Schritte
```
```

### 📄 merges/hausKI-audio_merge_2510262237__docs_policies.md

**Größe:** 315 B | **md5:** `0d479b90bbed6afa349d81dabecef367`

```markdown
### 📄 docs/policies/SECURITY.md

**Größe:** 196 B | **md5:** `4ecc35f7ca5aac8094ae13702087ca28`

```markdown
# Security Policy (Entwurf)

- Keine Secrets ins Repo.
- Qobuz App-ID/Secret & Login nur in `~/.config/mopidy/secret.conf`.
- Melde Sicherheitsprobleme privat (Issue als "security", oder direkt).
```
```

### 📄 merges/hausKI-audio_merge_2510262237__docs_process.md

**Größe:** 631 B | **md5:** `03e38a941b449831fa1ec6adcc6bbd77`

```markdown
### 📄 docs/process/CONTRIBUTING.md

**Größe:** 509 B | **md5:** `7e146132ce7bb524bd8e275cf4b2a4a1`

```markdown
# CONTRIBUTING

## Branch & Commit

- `main` gesichert, Feature-Branches nach Thema.
- Klarer Commit-Text (imperativ), kleine PRs.

## ADRs

- Wichtige Entscheidungen als ADR (siehe `docs/adr/`).

## Docs

- README aktuell halten.
- Runbooks, wenn wiederkehrende Handgriffe auftauchen.

## CI

- Lint für Markdown/YAML darf grün sein; später Build-Checks ergänzen.
- Lokal: `just lint` für markdownlint/yamllint (siehe `Justfile`).
- Tests: `just test` (Pytest) + `just rec-smoke` für Recorder-Dry-Run.
```
```

### 📄 merges/hausKI-audio_merge_2510262237__docs_runbooks.md

**Größe:** 4 KB | **md5:** `8a6928c1e2df2b2923c587e16c3c656e`

```markdown
### 📄 docs/runbooks/backend_service.md

**Größe:** 2 KB | **md5:** `f41de807d925f2eb00480ce05653a244`

```markdown
# Runbook: Hauski Backend Service

Ziel: Die HTTP-Fassade (`hauski-backend`) als User-Service betreiben.

## Voraussetzungen

- Rust Toolchain (`cargo`, `rustup`, `rustfmt`, `clippy`).
- `.env` (oder separates `backend.env`) mit Mopidy-/Script-Pfaden.
- Skripte unter `scripts/` ausführbar (werden vom Backend aufgerufen).

## Lokaler Start (Dev)

```bash
just backend-run                # bindet laut .env (Default 127.0.0.1:8080)
curl http://127.0.0.1:8080/mode # sollte pulsesink anzeigen
```

## Build & Deploy (systemd --user)

1. Release-Build erzeugen:

   ```bash
   cargo build --release -p hauski-backend
   install -Dm755 target/release/hauski-backend ~/.local/bin/hauski-backend
   ```

2. Environment-Datei anlegen (`~/.config/hauski-audio/backend.env`):

   ```ini
   MOPIDY_HTTP_URL=http://127.0.0.1:6680
   HAUSKI_BACKEND_BIND=127.0.0.1:8080
   HAUSKI_SCRIPT_WORKDIR=/home/alex/repos/hauski-audio
   HAUSKI_AUDIO_MODE_CMD=./scripts/audio-mode
   HAUSKI_PLAYLIST_FROM_LIST_CMD=./scripts/playlist-from-list
   ```

3. Systemd-Template nutzen (`tools/systemd/hauski-backend.service`):

   ```bash
   mkdir -p ~/.config/systemd/user
   cp tools/systemd/hauski-backend.service ~/.config/systemd/user/
   systemctl --user daemon-reload
   systemctl --user enable --now hauski-backend.service
   journalctl --user -u hauski-backend.service -f
   ```

## Endpoints (Kurzüberblick)

- `GET /health` → Backend-Status, optional Mopidy-Ping.
- `POST /rpc` → JSON-RPC Payload an Mopidy durchreichen.
- `GET/POST /mode` → `scripts/audio-mode` aufrufen.
- `POST /playlists/from-list` → URIs (JSON) an `scripts/playlist-from-list` streamen.
- `GET /discover/similar?seed=<uri>` → Mopidy-Suche nach ähnlichen Titeln.

## Fehlerbehebung

- `500 + command ... timed out`: Timeout in `HAUSKI_COMMAND_TIMEOUT_MS`
  erhöhen oder Skript prüfen.
- `502 + Mopidy returned`: Mopidy-HTTP-URL/Authentifizierung checken.
- Systemd: `systemctl --user status hauski-backend.service` bzw. Journal prüfen.
```

### 📄 docs/runbooks/mopidy_iris_qobuz.md

**Größe:** 2 KB | **md5:** `c2066aaad2a00281927635bf1a2f05f6`

```markdown
# Runbook – Mopidy / Iris / Qobuz (Hi-Res)

## Dienste

- Mopidy HTTP: <http://127.0.0.1:6680/> (Iris unter /iris)
- Mopidy MPD: 127.0.0.1:6600

## Konfig-Pfade

- `~/.config/mopidy/mopidy.conf` (Audio/HTTP/MPD)
- `~/.config/mopidy/secret.conf` ([qobuz] username, password, app_id, secret,
  quality)

## Qualitätsstufe

- `quality = 7` = Hi-Res bis 24/192
- (Optional) `27` versucht >96 kHz, bringt aber in der Praxis selten
  Mehrwert.

## Modus wechseln

- Komfort: Pulse → `output = pulsesink`
- Bitperfect: ALSA → `output = alsasink device=hw:<M2>,0`
- Nach Änderung: `systemctl --user restart mopidy`

## Aufnahme-Workflow

1. Audio-Modus prüfen: `just audio-mode MODE=show` → ggf. `MODE=alsa` für
   Bitperfect.
2. `just rec-start ARGS="--rate 96000 --channels 2"` startet PipeWire Aufnahme
   (`pw-record`).
3. CLI gibt Zielpfad mit Zeitstempel aus (`~/Music/Recordings/...`).
4. Stoppen via `just rec-stop` (sendet SIGINT, räumt PID-Datei).
5. Aufnahme validieren:
   - `pw-top` oder `pw-cli ls Node` zur Live-Überwachung.
   - `soxi <file>` / `mediainfo <file>` für Sample-Rate & Format.
   - `just rec-smoke` für Smoke-Test ohne aktive Aufnahme.

## Aufnahme-Optionen

- Sample-Format: `--format S32_LE` für 32-Bit float; Default `S24_LE`.
- Gerät wählen: `just rec-start ARGS="--device <pipewire-node>"` (z. B. MOTU
  Stream).
- Zusätzliche `pw-record` Flags: `--extra --latency=128` o.ä. werden direkt
  durchgereicht.
- Speicherort/Endung via `.env` (`AUDIO_RECORD_DIR`, `AUDIO_RECORD_EXT`),
  Binary mit `PW_RECORD_BINARY`.

## Troubleshooting

- **Recorder läuft schon:** `just rec-stop --force` beendet alte PID oder
  `just rec-start ARGS="--force"` räumt stale State.
- **Falsches Backend:** `just audio-mode MODE=pulse` für Alltag, danach Mopidy
  neu starten.
- **Keine Aufnahme hörbar:** `pw-top` prüfen, ob `pw-record` Streams
  empfängt; PipeWire-Source wählen (`pw-cli port set` oder `pavucontrol`).
- **Qobuz Login schlägt fehl:** Secrets in `~/.config/mopidy/secret.conf`
  prüfen, Mopidy-Logs (`journalctl --user -u mopidy`).
```
```

### 📄 merges/hausKI-audio_merge_2510262237__index.md

**Größe:** 19 KB | **md5:** `007190676bbb6af16862aff197deba04`

```markdown
# Ordner-Merge: hausKI-audio

**Zeitpunkt:** 2025-10-26 22:37
**Quelle:** `/home/alex/repos/hausKI-audio`
**Dateien (gefunden):** 48
**Gesamtgröße (roh):** 147 KB

**Exclude:** ['.gitignore']

## 📁 Struktur

- hausKI-audio/
  - .editorconfig
  - .env.example
  - .gitattributes
  - .gitignore
  - .hauski-reports
  - Cargo.lock
  - Cargo.toml
  - Justfile
  - README.md
  - pyproject.toml
  - tests/
    - __init__.py
    - test_audio_mode.py
    - test_rec_scripts.py
  - docs/
    - ARCHITECTURE.md
    - README_ALSA.md
    - io-contracts.md
    - troubleshooting.md
    - vibe-detection.md
    - adr/
      - 0001-player-backend-mopidy-qobuz.md
      - 0002-audio-path-pulse-vs-alsa.md
      - 0003-repo-standards-docs-ci.md
      - 0004-recording-pw-record-helper.md
      - README.md
    - process/
      - CONTRIBUTING.md
    - runbooks/
      - backend_service.md
      - mopidy_iris_qobuz.md
    - policies/
      - SECURITY.md
  - tools/
    - systemd/
      - hauski-backend.service
  - .github/
    - workflows/
      - docs-ci.yml
      - rust-ci.yml
      - validate-audio-events.yml
      - wgx-guard.yml
  - .wgx/
    - profile.yml
  - .git/
    - FETCH_HEAD
    - HEAD
    - ORIG_HEAD
    - config
    - description
    - index
    - packed-refs
    - hooks/
      - applypatch-msg.sample
      - commit-msg.sample
      - fsmonitor-watchman.sample
      - post-update.sample
      - pre-applypatch.sample
      - pre-commit.sample
      - pre-merge-commit.sample
      - pre-push
      - pre-push.sample
      - pre-rebase.sample
      - pre-receive.sample
      - prepare-commit-msg.sample
      - push-to-checkout.sample
      - update.sample
    - refs/
      - remotes/
        - origin/
          - HEAD
          - alert-autofix-1
          - alert-autofix-2
          - alert-autofix-3
          - alert-autofix-4
          - docs-troubleshooting-io-contracts
          - docs-vibe-detection
          - feat-add-audio-event-validation-workflow
          - fix-build-errors
          - fix-ci-config
          - fix-ci-profile-sync
          - fix-code-quality-improvements
          - fix-python-test-quality
          - fix-wgx-profile-sync
          - fix-yaml-linting-errors
          - main
          - docs/
            - audio-troubleshooting-io
            - justfile-consistency
          - refactor/
          - fix/
            - markdown-linting-errors
            - project-config-and-lint
            - repo-config-und-stil
          - feature/
            - robust-scripting-fallbacks
          - chore/
            - code-review
            - improve-dev-tooling
            - verify-linting
          - feat/
            - audio-events
      - tags/
      - heads/
        - main
        - backup/
          - main-20251017-182448
          - main-20251017-213708
          - main-20251018-090516
          - main-20251021-124257
          - main-20251023-070546
          - main-20251023-090518
          - main-20251025-233730
    - logs/
      - HEAD
      - refs/
        - remotes/
          - origin/
            - HEAD
            - alert-autofix-1
            - alert-autofix-2
            - alert-autofix-3
            - alert-autofix-4
            - docs-troubleshooting-io-contracts
            - docs-vibe-detection
            - feat-add-audio-event-validation-workflow
            - fix-build-errors
            - fix-ci-config
            - fix-ci-profile-sync
            - fix-code-quality-improvements
            - fix-python-test-quality
            - fix-wgx-profile-sync
            - fix-yaml-linting-errors
            - main
            - docs/
              - audio-troubleshooting-io
              - justfile-consistency
            - refactor/
              - bereinigung-python-setup
            - fix/
              - dependency-and-lint-errors
              - markdown-linting-errors
              - project-config-and-lint
              - repo-config-und-stil
            - feature/
              - robust-scripting-fallbacks
            - chore/
              - code-review
              - improve-dev-tooling
              - verify-linting
            - feat/
              - audio-events
        - heads/
          - main
          - backup/
            - main-20251017-182448
            - main-20251017-213708
            - main-20251018-090516
            - main-20251021-124257
            - main-20251023-070546
            - main-20251023-090518
            - main-20251025-233730
    - branches/
    - info/
      - exclude
      - refs
    - objects/
      - 48/
        - 1d527d52d5ba703acbfd3d96d3f260bf1efc37
        - 7698f59826f7b772bd685f89c1a30ba7117a6e
        - e0e0b1d2632bfddd75444d81604a3846dab795
      - e5/
        - 179be0544a2feb71ed0dc6f74004ddc018a187
      - 6d/
        - 2213db16c3ee4b4c70c4d9b3ba59a2f57e4b75
      - 0a/
        - 9f530b3aa1a887e63fae8cede6c454991fc3af
      - 06/
        - 6039bb68c490176f331996f9e48a30f472bee2
      - 2d/
        - b00359532841fadedb4d78775d99e17e453dec
      - d8/
        - c33d0cd5ecc27f9858f55c7721bc2bf8773176
      - 87/
        - dc44a37cdda3640c7dd81b6e6d70f0628dc389
      - ad/
        - 9f123876184516d1a9bd78ab55b1da2da00417
        - bd076068179797e7fc053485c3bb0358e74f97
      - 23/
        - 6ecb720a27cb8054aaf571e253c007f0aa54ff
      - ac/
        - 8c0fda0b37b147ac80e968ee095908584febf5
      - 72/
        - 1599874b6968f120c4e8ba62c644fff61b5543
      - 6b/
        - 2de191ade0daee5e261e33e17c040a23543d2f
      - 0e/
        - abb067c1fccacf58b2a1b0e754f90a065251b3
        - ac34d968a52bbc5b8573f27d9ce9c9a970ed08
        - ae069d4894640b79cee8b2bf68ff659a59add5
        - d357c1e6ac18497373ed6a9df59311c89c3ff2
      - e0/
        - 335b119b2cf5d30cb22a72d5d3cc20fd32a1fb
      - 54/
        - 35885bd101291d91d4f6a4ddf8ec2ae83cf12a
        - 70718468ac44da7d8082a034b31baa3c8e60f5
        - a88c02a0c8902fc905a2dc400ac4171a6088a7
      - 17/
        - 03d52d47ab13579cbb298e9899583a42a03d7a
      - fd/
        - 5df80a626e04b9b143de4ca23f7d37fa9396a5
      - 8f/
        - c258351a3c5bf5b5a43e61268c9645174bcd1e
      - b2/
        - 97f924216bbfa9b67bfc22763985c5077d725e
        - dea18f02866b8d3c696e0047d000d5bab0b41c
      - 38/
        - 5dceb9cf16f3ced012b8c033a4b8288f06b9cf
      - 14/
        - b67ff70748e422f4fe9bf05c120b9713f6279f
      - 70/
        - 3fa8c9689b15cd1cbaada1398eac3feecc5415
        - 816078e8b90203fe31a1d5897404847a2062fc
      - f9/
        - 5141ac098ef08a2219e434e99cdef91cd0e7d9
      - 77/
        - 93be315ca3e3fce6a73f4b3d508178fe92f685
        - 9ddb20685cc626c48de664d609f3e2fe2668d5
      - 4a/
        - b0807c0f6e9986b160e8b9e33cf9b457f175b2
      - 5a/
        - 276a1095e8e22ed0a3a189f7e62c5819559f9a
      - 12/
        - 0d0b59d1d5f87b909493c6859773c398d9b074
      - 01/
        - 3824b9d967f6e5ddf95f8f947301f1f8bb39f4
      - f8/
        - 018dc5f8f3b7d942c3a5d07888a2f784e273a7
        - b74d24ce7c8da06cb7b290ded30948d33b1031
      - 13/
        - 6d593bf0b45764d20277bbbb7d50b244a8b044
        - 81a9ab97ab18f9c0c6f34018940cfceb3913e0
      - 56/
        - 73a82566e83773924183574ec18b974e7ef472
      - 9e/
        - 07bf36bc7ee04ef1f0a6d4992d654cc59eea5a
        - 66a2cee85423f58a2f2883e85ae74e46d5e618
      - d3/
        - 9f72ce6bac2467d965b4c5e1b13c3ca5859b13
      - dd/
        - a156889866b3818e054c16e18aba9edf8e6930
      - 26/
        - a3cdf562fde48c5da94618d26fcccacb416a28
      - c6/
        - 999b0980f1138260fb235561df4520d03c86ea
      - 0f/
        - c4fe76b75407e46726a70e6a683d7c8bb84231
      - ab/
        - 6a5ca1bffdb66eda40c70c59d640ffb12a95f7
      - 57/
        - f39d2bb9851b1e011bf7f1f74f74188de5dda4
      - pack/
        - pack-71f9a9c11f1485f56d420d5f8ca884dd79fa767e.idx
        - pack-71f9a9c11f1485f56d420d5f8ca884dd79fa767e.pack
      - 9a/
        - 028bc66375f5605f42161522e699508b9db4da
        - 8c09e93a9e9a5404ce8438b495f601114c560e
        - cb92fef92e2c192e7e5d90b51394f897951f8b
      - 11/
        - ad21a55b0564a21c114f855c220d0cd935c638
      - 71/
        - 49820b8aeffcbf8b32e7dff4abe4860d4d2c5a
        - 8cb2fb219c69290ca123ec6e13e73e85730f01
      - c3/
        - 2803946a286c4f3f496d80b95f142fd2a38c6b
      - 66/
        - 8051a6f6e4d332c07f44277006c3dd39ee0be2
      - d7/
        - de0eab62ae67a5d19df12e4bfbe4e0d900afba
      - bb/
        - 792a60aefc9b204b4c837387f80798642fc67f
        - ffc10ffdf026a165cf9b8f6e79e862c8eadde8
      - 18/
        - 74f9250183cf7df5116ba8c5036ac325ed961d
      - a4/
        - a9250fb2c230dc127414e08e2f40ccdb549de4
      - 84/
        - b0ce5c1c9d22231a2fc36fb08562fb82bab932
        - b4cb661856c2ee3359a864c6eeb9d0e928ae30
      - fa/
        - 98b1d9cdb3318d774eeb1dc21ea74e0bfa3dc7
      - d1/
        - 48bdaacf38143782cd3937e6b3818143260b69
      - aa/
        - 7be45be5cb46eec342f1a9e14e1c006fc6f842
        - 902fa8e74f79df1deee540a721f9e100932f3b
      - 4b/
        - a556092e19d95130573e2857295bcfc1d2d36c
      - a0/
        - 4c44b0937a4ff42054e25c50cb62d95dec1693
        - bca8743dae6d67560793ef4d230928607fce02
      - 20/
        - 613eae54e3ec2986232d25235a672afc8e6929
      - 8a/
        - 19c2509dda2bd1a4dcd08aa37451d81c9c4795
      - 7b/
        - 32e99095835e24272c1ae0554b6a5ba95be9db
      - e6/
        - 9de29bb2d1d6434b8b29ae775ad8c2e48c5391
      - eb/
        - 0fbb24e134d658215c7ed03ad17e961fb1e5a5
      - 45/
        - 809b44e5edfe0bc12e4eb2bf870d3b293872ca
      - 5d/
        - 75e37757f2a269997c7cfbd71e55b175eb8a20
      - 8b/
        - 0ded015102ac9c84a522bb359db553de015da1
      - 30/
        - 74384b1f2bb8b6c64982f50495df3a1450afd5
        - d946af6a7a7c78846cd60a6e011ef074d1f2ad
      - 7a/
        - 75b6a4d239e928f1ea2609bcf3a4c17afb353a
      - f7/
        - 0ffb1dc96ce06df6f5d9e9d584f46eb77b4a6a
      - 16/
        - 2ab7b64f11c5ede5b2aae0fcec945cad2b12e0
        - 65cc526d041a6d62b2f36a506e37fb6d4a77de
        - 6acbb541322324938d9d138d5bc083c3b1c54a
      - 28/
        - 576d001e0fa9db75eca9321825b44e5bc0265d
      - fb/
        - 3777c83b811ca72d01197c819aeb7cad5284cf
      - 4c/
        - 24ca19088b134f491acc3344f6dc1078b41192
      - 53/
        - cedd45f0138883c70b9e4ed8000fa1728a1e48
        - cefc688ee518d8525eacccd07f11bb01d7e761
        - e368b3cd0d38fa895fa860b606508671a786cc
      - 46/
        - f3dd2ed534f3a74358aa97fb1dde2c6bdd5d6e
      - d9/
        - bc17254c1133a24a6cbe3268cecadfe051a2ef
      - 03/
        - aef2dd1c55869bf0d810d47056d15f5a3d4ac5
      - 75/
        - 8aaf553c8cf7b5149a84e28d59a015c7b34764
      - e8/
        - 305579df7124e292b75fb978e593d9941eec35
      - 63/
        - 2ab066d7b2cc5e4d7667316fe93dde3104cc18
      - 1b/
        - acdd869e148b2174a47e89ed6d86bf23aed24b
      - 74/
        - 95cb64a1a69ed2cf471321aff7849fea661dc5
        - e165404ea3c73a4d7824d6c63bac0457f855ec
      - 83/
        - 71a2a3e783ee5e6a393b48ec7904184c530429
      - 3a/
        - bea85a3ace65694d3783ebbfcf22e742ac598d
      - ce/
        - 2cf41b441820ff55a11ae272401f6e6674e28a
        - 48ff7dcb0f9766619de909187b2737870b6eac
      - b3/
        - 0bddc50babb33675fb63960c53272a3810a6e9
      - 81/
        - ac438092f489bc66eda97c4ad4f3fe95b06b0c
      - a5/
        - 4247765b469da03fc5128cc76548c9fda21838
      - c8/
        - 0a7ba4daf3445330a6f0a7d82b28cf83a5d639
      - 9c/
        - 0eb4394ab46ef20a2966abc038d45920513d40
      - 73/
        - 69d4f26e1d623095986beb9a7099587a2c0b09
        - a740827568775d9f81d4a27956ab0ab5f09f2f
      - 10/
        - ce66c08f28d5ba5acd7a90080134fd319edaef
        - e83077b5bfcb34fa19fd958560beb1beeb818b
      - 1c/
        - 8335726450220e836b4b9d9ac7c4abfadb61b9
      - 41/
        - 1979b3912ae697ef49bc8b8c3042fbf0d5bd3e
      - e1/
        - e1fab9b43a0f44dd11f572a31f410bf5cf220c
      - b4/
        - a9fd0a02fd9ce4508b10be7e8abbceec4fdee5
      - 58/
        - 21a46bbc7475d24bfbf0a9fed52de5deaa434e
      - 55/
        - 7830f65738f2ccab03cb66676c89a12f65673e
      - b1/
        - d409a08e9e9bd51e3c98b5e5efa112d17e91fd
        - db6d939a097f5cd9cc9632a043be9c426d0b62
      - 85/
        - 0ce339345c739357417ffeaa4fb82474f07679
        - 6fd34e1885e36222ce2cb0eb99a5180eab42e2
      - 04/
        - bdd718209eeb9a09dfac1a07dbf0f9cde39ad2
      - 1a/
        - 44dd93798833307a4e9be473d22b2fe69b1c08
        - 61823dcc4fc8d081a5511aa69eb42c3c5cb0d0
      - f2/
        - 0bb660dc8434a62269faf7de8cd124da87af0d
      - 91/
        - 4a6e71854ada3df51d4b981db6f1b873776dea
      - 8c/
        - c489e45c572901e54ac7eb62e176d101866282
      - 7f/
        - c3675d1ca34d074a044eed6cd9dc76df7ce93d
        - f2e914b62a0627874865334cb12b23c87e4969
        - f6ce8233d823647b30de3bae375af446a5d1cb
      - 25/
        - 0d530cea7184c1e1e18cf3384cdaccdf98b3fb
        - 9644b4007d58cc910e8e6a235aaa53c63e1929
      - c0/
        - 5cc0938d7445f2d1edb3dee4da53c35a4cae6d
      - 43/
        - 300f484c87d86b6263f723cfdd6a0efa76a617
        - b1f13824ff2159b340323d2888be7b2f89b1a4
      - c1/
        - 581aba1c9ed7ebc29c727df0ce20ef3d1b9561
      - 07/
        - be22e1d62c0f9e0273efa840084d49e26e32a3
      - 05/
        - f63ca16ebd063ee6dd4f5621d03d8cd735659f
        - f98169ba8de635b9ab00c935ebb41331431313
      - c5/
        - d0c4a0c46184a55f67e89c2b5c71487fc9f633
      - db/
        - 0ad91a04192e72cb3b58c4248a0db3c3594919
        - 3e2ce04e5fecec1e77c8e179bc8a01ab9c3bd4
      - a7/
        - 0db28893074882bd17e15b0a805b040e95bd64
      - e3/
        - 3c2791fb3834c87ee951de3fad085c91ca3beb
        - 6867da5217db9a533a41ab907823e5f425af44
        - 8ddec139f116395871ccf087d7fb1c5c452e8c
      - ea/
        - 92570694151abf6209b992d78a9efc8c0e8d1b
      - b8/
        - f33a614997f1be9228223fbcc5bab0f2841e22
      - 52/
        - d7ec9aaceb496f41aa2ad26e666a35f3c93daf
      - 93/
        - 00afb9fc42079d68e68becbb9870387f79d19f
      - 67/
        - 3e9300d705bd5feac402c15face55442f557a7
        - 86eecd6953410d20bbc9d15d4d2ef28e38077b
      - be/
        - 592c7dcf14d9dc666bff991fd6c91a96deb22e
      - c2/
        - a39957bfa3f2246f78809c7a52536c051c4cc6
      - info/
        - commit-graph
        - packs
      - 15/
        - 1a07cd377f79ec5065cbb3c47b65df6ce2b583
      - 88/
        - dc3cf8d2921142e1b4d9c847a4718fcd750f39
        - f5e8a2cb7c44e78b0d9280965a155eb259ce45
      - cf/
        - 10e3b7a7b2c194bd0291b1879b0828b2c0e93c
      - 33/
        - a4bc536139a05986e906468680d96af81461e0
      - 34/
        - 5bfd93aa60c749c0e1e142f07ece53b43a6448
        - c7c253241994fbc5cc6e96fa0c097a59a7a1b2
      - 5c/
        - 0fa1ad1eacc4492c6c1a29f2546c1574c51115
      - ba/
        - d7261c19283f1594f6ff8279714f13bba842e7
      - b7/
        - 8ce6c1963feac6d08a2cc9a64961e84d05c48f
      - ef/
        - f44109ba642a182c0011099e5ba7ac40e8addb
      - a2/
        - da252c01f56a3b4332e8cc39e075c6d9c91384
        - fc57a7d25ba00bc9beeae942eaa6535f6bf4c2
      - 7e/
        - 00c966b4cd9794764da1aa1acfd449313af013
        - a2f99af2724e58d1c5f5f53074991eedcc63f5
  - merges/
    - hausKI-audio_merge_2510262237__index.md
  - crates/
    - backend/
      - Cargo.toml
      - tests/
        - http.rs
      - src/
        - config.rs
        - discover.rs
        - error.rs
        - handlers.rs
        - lib.rs
        - main.rs
        - models.rs
        - mopidy.rs
        - scripts.rs
  - scripts/
    - README.md
    - audio-mode
    - playlist-from-list
    - rec-start
    - rec-stop

## 📦 Inhalte (Chunks)

- .editorconfig → `hausKI-audio_merge_2510262237__root.md`
- .env.example → `hausKI-audio_merge_2510262237__root.md`
- .gitattributes → `hausKI-audio_merge_2510262237__root.md`
- .gitignore → `hausKI-audio_merge_2510262237__root.md`
- Cargo.lock → `hausKI-audio_merge_2510262237__root.md`
- Cargo.toml → `hausKI-audio_merge_2510262237__root.md`
- Justfile → `hausKI-audio_merge_2510262237__root.md`
- README.md → `hausKI-audio_merge_2510262237__root.md`
- pyproject.toml → `hausKI-audio_merge_2510262237__root.md`
- tests/__init__.py → `hausKI-audio_merge_2510262237__tests.md`
- tests/test_audio_mode.py → `hausKI-audio_merge_2510262237__tests.md`
- tests/test_rec_scripts.py → `hausKI-audio_merge_2510262237__tests.md`
- docs/ARCHITECTURE.md → `hausKI-audio_merge_2510262237__docs.md`
- docs/README_ALSA.md → `hausKI-audio_merge_2510262237__docs.md`
- docs/io-contracts.md → `hausKI-audio_merge_2510262237__docs.md`
- docs/troubleshooting.md → `hausKI-audio_merge_2510262237__docs.md`
- docs/vibe-detection.md → `hausKI-audio_merge_2510262237__docs.md`
- docs/adr/0001-player-backend-mopidy-qobuz.md → `hausKI-audio_merge_2510262237__docs_adr.md`
- docs/adr/0002-audio-path-pulse-vs-alsa.md → `hausKI-audio_merge_2510262237__docs_adr.md`
- docs/adr/0003-repo-standards-docs-ci.md → `hausKI-audio_merge_2510262237__docs_adr.md`
- docs/adr/0004-recording-pw-record-helper.md → `hausKI-audio_merge_2510262237__docs_adr.md`
- docs/adr/README.md → `hausKI-audio_merge_2510262237__docs_adr.md`
- docs/process/CONTRIBUTING.md → `hausKI-audio_merge_2510262237__docs_process.md`
- docs/runbooks/backend_service.md → `hausKI-audio_merge_2510262237__docs_runbooks.md`
- docs/runbooks/mopidy_iris_qobuz.md → `hausKI-audio_merge_2510262237__docs_runbooks.md`
- docs/policies/SECURITY.md → `hausKI-audio_merge_2510262237__docs_policies.md`
- tools/systemd/hauski-backend.service → `hausKI-audio_merge_2510262237__tools_systemd.md`
- .github/workflows/docs-ci.yml → `hausKI-audio_merge_2510262237__.github_workflows.md`
- .github/workflows/rust-ci.yml → `hausKI-audio_merge_2510262237__.github_workflows.md`
- .github/workflows/validate-audio-events.yml → `hausKI-audio_merge_2510262237__.github_workflows.md`
- .github/workflows/wgx-guard.yml → `hausKI-audio_merge_2510262237__.github_workflows.md`
- .wgx/profile.yml → `hausKI-audio_merge_2510262237__.wgx.md`
- crates/backend/Cargo.toml → `hausKI-audio_merge_2510262237__crates_backend.md`
- crates/backend/tests/http.rs → `hausKI-audio_merge_2510262237__crates_backend_tests.md`
- crates/backend/src/config.rs → `hausKI-audio_merge_2510262237__crates_backend_src.md`
- crates/backend/src/discover.rs → `hausKI-audio_merge_2510262237__crates_backend_src.md`
- crates/backend/src/error.rs → `hausKI-audio_merge_2510262237__crates_backend_src.md`
- crates/backend/src/handlers.rs → `hausKI-audio_merge_2510262237__crates_backend_src.md`
- crates/backend/src/lib.rs → `hausKI-audio_merge_2510262237__crates_backend_src.md`
- crates/backend/src/main.rs → `hausKI-audio_merge_2510262237__crates_backend_src.md`
- crates/backend/src/models.rs → `hausKI-audio_merge_2510262237__crates_backend_src.md`
- crates/backend/src/mopidy.rs → `hausKI-audio_merge_2510262237__crates_backend_src.md`
- crates/backend/src/scripts.rs → `hausKI-audio_merge_2510262237__crates_backend_src.md`
- scripts/README.md → `hausKI-audio_merge_2510262237__scripts.md`
- scripts/audio-mode → `hausKI-audio_merge_2510262237__scripts.md`
- scripts/playlist-from-list → `hausKI-audio_merge_2510262237__scripts.md`
- scripts/rec-start → `hausKI-audio_merge_2510262237__scripts.md`
- scripts/rec-stop → `hausKI-audio_merge_2510262237__scripts.md`
```

### 📄 merges/hausKI-audio_merge_2510262237__part001.md

**Größe:** 43 B | **md5:** `ad150e6cdda3920dbef4d54c92745d83`

```markdown
<!-- chunk:1 created:2025-10-26 22:37 -->
```

### 📄 merges/hausKI-audio_merge_2510262237__root.md

**Größe:** 27 KB | **md5:** `a1025a05fa4fa87110f068f957d4e010`

```markdown
### 📄 .editorconfig

**Größe:** 146 B | **md5:** `7403bbacd268124861c84dc8cd4ccfc5`

```plaintext
root = true

[*]
end_of_line = lf
charset = utf-8
insert_final_newline = true
trim_trailing_whitespace = true

[*.{md,mdx}]
max_line_length = off
```

### 📄 .env.example

**Größe:** 975 B | **md5:** `7c9bd5930bc0627bd368946b64437013`

```plaintext
# Copy to .env and adjust values for local tooling. .env stays untracked.

# Mopidy endpoints
MOPIDY_HTTP_URL=http://127.0.0.1:6680
MOPIDY_IRIS_URL=http://127.0.0.1:6680/iris
MOPIDY_MPD_HOST=127.0.0.1
MOPIDY_MPD_PORT=6600

# Local configuration paths
MOPIDY_CONFIG=~/.config/mopidy/mopidy.conf
MOPIDY_SECRET_CONFIG=~/.config/mopidy/secret.conf

# Qobuz credentials (keep private; never commit to git)
QOBUZ_APP_ID=
QOBUZ_APP_SECRET=
QOBUZ_USERNAME=
QOBUZ_PASSWORD=

# Recording defaults
AUDIO_RECORD_DIR=~/Music/Recordings
AUDIO_RECORD_EXT=wav
PW_RECORD_BINARY=pw-record

# Backend service settings
HAUSKI_BACKEND_BIND=127.0.0.1:8080
# Override script paths if the backend runs outside the repo root
# HAUSKI_SCRIPT_WORKDIR=/home/alex/repos/hauski-audio
# HAUSKI_AUDIO_MODE_CMD=./scripts/audio-mode
# HAUSKI_PLAYLIST_FROM_LIST_CMD=./scripts/playlist-from-list
# Set to 0 to skip Mopidy health probe on /health
# HAUSKI_CHECK_MOPIDY_HEALTH=1
# HAUSKI_COMMAND_TIMEOUT_MS=10000
```

### 📄 .gitattributes

**Größe:** 19 B | **md5:** `19373440c0d117c03502243e420e6cbb`

```plaintext
* text=auto eol=lf
```

### 📄 .gitignore

**Größe:** 154 B | **md5:** `a3370ddbd429553c12fbb7ddca8a273d`

```plaintext
/.env
/node_modules/
/dist/
/target/
/venv/
/.cache/
*.log
.DS_Store

# Python caches
__pycache__/
*.egg-info/
.pytest_cache/
.ruff_cache/
.venv/
uv.lock
```

### 📄 Cargo.lock

**Größe:** 47 KB | **md5:** `637642e5602d593183abd6d2d2b038a5`

```plaintext
# This file is automatically @generated by Cargo.
# It is not intended for manual editing.
version = 4

[[package]]
name = "addr2line"
version = "0.25.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1b5d307320b3181d6d7954e663bd7c774a838b8220fe0593c86d9fb09f498b4b"
dependencies = [
 "gimli",
]

[[package]]
name = "adler2"
version = "2.0.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "320119579fcad9c21884f5c4861d16174d0e06250625266f50fe6898340abefa"

[[package]]
name = "aho-corasick"
version = "1.1.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8e60d3430d3a69478ad0993f19238d2df97c507009a52b3c10addcd7f6bcb916"
dependencies = [
 "memchr",
]

[[package]]
name = "async-trait"
version = "0.1.89"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9035ad2d096bed7955a320ee7e2230574d28fd3c3a0f186cbea1ff3c7eed5dbb"
dependencies = [
 "proc-macro2",
 "quote",
 "syn",
]

[[package]]
name = "atomic-waker"
version = "1.1.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1505bd5d3d116872e7271a6d4e16d81d0c8570876c8de68093a09ac269d8aac0"

[[package]]
name = "axum"
version = "0.7.9"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "edca88bc138befd0323b20752846e6587272d3b03b0343c8ea28a6f819e6e71f"
dependencies = [
 "async-trait",
 "axum-core",
 "axum-macros",
 "bytes",
 "futures-util",
 "http 1.3.1",
 "http-body 1.0.1",
 "http-body-util",
 "hyper 1.7.0",
 "hyper-util",
 "itoa",
 "matchit",
 "memchr",
 "mime",
 "percent-encoding",
 "pin-project-lite",
 "rustversion",
 "serde",
 "serde_json",
 "serde_path_to_error",
 "serde_urlencoded",
 "sync_wrapper 1.0.2",
 "tokio",
 "tower 0.5.2",
 "tower-layer",
 "tower-service",
 "tracing",
]

[[package]]
name = "axum-core"
version = "0.4.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "09f2bd6146b97ae3359fa0cc6d6b376d9539582c7b4220f041a33ec24c226199"
dependencies = [
 "async-trait",
 "bytes",
 "futures-util",
 "http 1.3.1",
 "http-body 1.0.1",
 "http-body-util",
 "mime",
 "pin-project-lite",
 "rustversion",
 "sync_wrapper 1.0.2",
 "tower-layer",
 "tower-service",
 "tracing",
]

[[package]]
name = "axum-macros"
version = "0.4.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "57d123550fa8d071b7255cb0cc04dc302baa6c8c4a79f55701552684d8399bce"
dependencies = [
 "proc-macro2",
 "quote",
 "syn",
]

[[package]]
name = "backtrace"
version = "0.3.76"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bb531853791a215d7c62a30daf0dde835f381ab5de4589cfe7c649d2cbe92bd6"
dependencies = [
 "addr2line",
 "cfg-if",
 "libc",
 "miniz_oxide",
 "object",
 "rustc-demangle",
 "windows-link",
]

[[package]]
name = "base64"
version = "0.21.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9d297deb1925b89f2ccc13d7635fa0714f12c87adce1c75356b39ca9b7178567"

[[package]]
name = "bitflags"
version = "1.3.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bef38d45163c2f1dde094a7dfd33ccf595c92905c8f8f4fdc18d06fb1037718a"

[[package]]
name = "bitflags"
version = "2.9.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2261d10cca569e4643e526d8dc2e62e433cc8aba21ab764233731f8d369bf394"

[[package]]
name = "bumpalo"
version = "3.19.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "46c5e41b57b8bba42a04676d81cb89e9ee8e859a1a66f80a5a72e1cb76b34d43"

[[package]]
name = "bytes"
version = "1.10.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d71b6127be86fdcfddb610f7182ac57211d4b18a3e9c82eb2d17662f2227ad6a"

[[package]]
name = "cc"
version = "1.2.40"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e1d05d92f4b1fd76aad469d46cdd858ca761576082cd37df81416691e50199fb"
dependencies = [
 "find-msvc-tools",
 "shlex",
]

[[package]]
name = "cfg-if"
version = "1.0.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2fd1289c04a9ea8cb22300a459a72a385d7c73d3259e2ed7dcb2af674838cfa9"

[[package]]
name = "core-foundation"
version = "0.9.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "91e195e091a93c46f7102ec7818a2aa394e1e1771c3ab4825963fa03e45afb8f"
dependencies = [
 "core-foundation-sys",
 "libc",
]

[[package]]
name = "core-foundation-sys"
version = "0.8.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "773648b94d0e5d620f64f280777445740e61fe701025087ec8b57f45c791888b"

[[package]]
name = "displaydoc"
version = "0.2.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "97369cbbc041bc366949bc74d34658d6cda5621039731c6310521892a3a20ae0"
dependencies = [
 "proc-macro2",
 "quote",
 "syn",
]

[[package]]
name = "dotenvy"
version = "0.15.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1aaf95b3e5c8f23aa320147307562d361db0ae0d51242340f558153b4eb2439b"

[[package]]
name = "encoding_rs"
version = "0.8.35"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "75030f3c4f45dafd7586dd6780965a8c7e8e285a5ecb86713e63a79c5b2766f3"
dependencies = [
 "cfg-if",
]

[[package]]
name = "equivalent"
version = "1.0.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "877a4ace8713b0bcf2a4e7eec82529c029f1d0619886d18145fea96c3ffe5c0f"

[[package]]
name = "errno"
version = "0.3.14"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "39cab71617ae0d63f51a36d69f866391735b51691dbda63cf6f96d042b63efeb"
dependencies = [
 "libc",
 "windows-sys 0.61.1",
]

[[package]]
name = "fastrand"
version = "2.3.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "37909eebbb50d72f9059c3b6d82c0463f2ff062c9e95845c43a6c9c0355411be"

[[package]]
name = "find-msvc-tools"
version = "0.1.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0399f9d26e5191ce32c498bebd31e7a3ceabc2745f0ac54af3f335126c3f24b3"

[[package]]
name = "fnv"
version = "1.0.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3f9eec918d3f24069decb9af1554cad7c880e2da24a9afd88aca000531ab82c1"

[[package]]
name = "foreign-types"
version = "0.3.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f6f339eb8adc052cd2ca78910fda869aefa38d22d5cb648e6485e4d3fc06f3b1"
dependencies = [
 "foreign-types-shared",
]

[[package]]
name = "foreign-types-shared"
version = "0.1.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "00b0228411908ca8685dba7fc2cdd70ec9990a6e753e89b6ac91a84c40fbaf4b"

[[package]]
name = "form_urlencoded"
version = "1.2.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "cb4cb245038516f5f85277875cdaa4f7d2c9a0fa0468de06ed190163b1581fcf"
dependencies = [
 "percent-encoding",
]

[[package]]
name = "futures-channel"
version = "0.3.31"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "2dff15bf788c671c1934e366d07e30c1814a8ef514e1af724a602e8a2fbe1b10"
dependencies = [
 "futures-core",
]

[[package]]
name = "futures-core"
version = "0.3.31"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "05f29059c0c2090612e8d742178b0580d2dc940c837851ad723096f87af6663e"

[[package]]
name = "futures-sink"
version = "0.3.31"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e575fab7d1e0dcb8d0c7bcf9a63ee213816ab51902e6d244a95819acacf1d4f7"

[[package]]
name = "futures-task"
version = "0.3.31"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f90f7dce0722e95104fcb095585910c0977252f286e354b5e3bd38902cd99988"

[[package]]
name = "futures-util"
version = "0.3.31"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9fa08315bb612088cc391249efdc3bc77536f16c91f6cf495e6fbe85b20a4a81"
dependencies = [
 "futures-core",
 "futures-task",
 "pin-project-lite",
 "pin-utils",
]

[[package]]
name = "getrandom"
version = "0.3.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "26145e563e54f2cadc477553f1ec5ee650b00862f0a58bcd12cbdc5f0ea2d2f4"
dependencies = [
 "cfg-if",
 "libc",
 "r-efi",
 "wasi 0.14.7+wasi-0.2.4",
]

[[package]]
name = "gimli"
version = "0.32.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e629b9b98ef3dd8afe6ca2bd0f89306cec16d43d907889945bc5d6687f2f13c7"

[[package]]
name = "h2"
version = "0.3.27"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0beca50380b1fc32983fc1cb4587bfa4bb9e78fc259aad4a0032d2080309222d"
dependencies = [
 "bytes",
 "fnv",
 "futures-core",
 "futures-sink",
 "futures-util",
 "http 0.2.12",
 "indexmap",
 "slab",
 "tokio",
 "tokio-util",
 "tracing",
]

[[package]]
name = "hashbrown"
version = "0.16.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5419bdc4f6a9207fbeba6d11b604d481addf78ecd10c11ad51e76c2f6482748d"

[[package]]
name = "hauski-backend"
version = "0.1.0"
dependencies = [
 "async-trait",
 "axum",
 "dotenvy",
 "http 0.2.12",
 "http-body-util",
 "reqwest",
 "serde",
 "serde_json",
 "tempfile",
 "thiserror",
 "tokio",
 "tower 0.4.13",
 "tower-http",
 "tracing",
 "tracing-subscriber",
 "url",
]

[[package]]
name = "http"
version = "0.2.12"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "601cbb57e577e2f5ef5be8e7b83f0f63994f25aa94d673e54a92d5c516d101f1"
dependencies = [
 "bytes",
 "fnv",
 "itoa",
]

[[package]]
name = "http"
version = "1.3.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f4a85d31aea989eead29a3aaf9e1115a180df8282431156e533de47660892565"
dependencies = [
 "bytes",
 "fnv",
 "itoa",
]

[[package]]
name = "http-body"
version = "0.4.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7ceab25649e9960c0311ea418d17bee82c0dcec1bd053b5f9a66e265a693bed2"
dependencies = [
 "bytes",
 "http 0.2.12",
 "pin-project-lite",
]

[[package]]
name = "http-body"
version = "1.0.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1efedce1fb8e6913f23e0c92de8e62cd5b772a67e7b3946df930a62566c93184"
dependencies = [
 "bytes",
 "http 1.3.1",
]

[[package]]
name = "http-body-util"
version = "0.1.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "b021d93e26becf5dc7e1b75b1bed1fd93124b374ceb73f43d4d4eafec896a64a"
dependencies = [
 "bytes",
 "futures-core",
 "http 1.3.1",
 "http-body 1.0.1",
 "pin-project-lite",
]

[[package]]
name = "httparse"
version = "1.10.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6dbf3de79e51f3d586ab4cb9d5c3e2c14aa28ed23d180cf89b4df0454a69cc87"

[[package]]
name = "httpdate"
version = "1.0.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "df3b46402a9d5adb4c86a0cf463f42e19994e3ee891101b1841f30a545cb49a9"

[[package]]
name = "hyper"
version = "0.14.32"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "41dfc780fdec9373c01bae43289ea34c972e40ee3c9f6b3c8801a35f35586ce7"
dependencies = [
 "bytes",
 "futures-channel",
 "futures-core",
 "futures-util",
 "h2",
 "http 0.2.12",
 "http-body 0.4.6",
 "httparse",
 "httpdate",
 "itoa",
 "pin-project-lite",
 "socket2 0.5.10",
 "tokio",
 "tower-service",
 "tracing",
 "want",
]

[[package]]
name = "hyper"
version = "1.7.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "eb3aa54a13a0dfe7fbe3a59e0c76093041720fdc77b110cc0fc260fafb4dc51e"
dependencies = [
 "atomic-waker",
 "bytes",
 "futures-channel",
 "futures-core",
 "http 1.3.1",
 "http-body 1.0.1",
 "httparse",
 "httpdate",
 "itoa",
 "pin-project-lite",
 "pin-utils",
 "smallvec",
 "tokio",
]

[[package]]
name = "hyper-tls"
version = "0.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d6183ddfa99b85da61a140bea0efc93fdf56ceaa041b37d553518030827f9905"
dependencies = [
 "bytes",
 "hyper 0.14.32",
 "native-tls",
 "tokio",
 "tokio-native-tls",
]

[[package]]
name = "hyper-util"
version = "0.1.17"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3c6995591a8f1380fcb4ba966a252a4b29188d51d2b89e3a252f5305be65aea8"
dependencies = [
 "bytes",
 "futures-core",
 "http 1.3.1",
 "http-body 1.0.1",
 "hyper 1.7.0",
 "pin-project-lite",
 "tokio",
 "tower-service",
]

[[package]]
name = "icu_collections"
version = "2.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "200072f5d0e3614556f94a9930d5dc3e0662a652823904c3a75dc3b0af7fee47"
dependencies = [
 "displaydoc",
 "potential_utf",
 "yoke",
 "zerofrom",
 "zerovec",
]

[[package]]
name = "icu_locale_core"
version = "2.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0cde2700ccaed3872079a65fb1a78f6c0a36c91570f28755dda67bc8f7d9f00a"
dependencies = [
 "displaydoc",
 "litemap",
 "tinystr",
 "writeable",
 "zerovec",
]

[[package]]
name = "icu_normalizer"
version = "2.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "436880e8e18df4d7bbc06d58432329d6458cc84531f7ac5f024e93deadb37979"
dependencies = [
 "displaydoc",
 "icu_collections",
 "icu_normalizer_data",
 "icu_properties",
 "icu_provider",
 "smallvec",
 "zerovec",
]

[[package]]
name = "icu_normalizer_data"
version = "2.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "00210d6893afc98edb752b664b8890f0ef174c8adbb8d0be9710fa66fbbf72d3"

[[package]]
name = "icu_properties"
version = "2.0.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "016c619c1eeb94efb86809b015c58f479963de65bdb6253345c1a1276f22e32b"
dependencies = [
 "displaydoc",
 "icu_collections",
 "icu_locale_core",
 "icu_properties_data",
 "icu_provider",
 "potential_utf",
 "zerotrie",
 "zerovec",
]

[[package]]
name = "icu_properties_data"
version = "2.0.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "298459143998310acd25ffe6810ed544932242d3f07083eee1084d83a71bd632"

[[package]]
name = "icu_provider"
version = "2.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "03c80da27b5f4187909049ee2d72f276f0d9f99a42c306bd0131ecfe04d8e5af"
dependencies = [
 "displaydoc",
 "icu_locale_core",
 "stable_deref_trait",
 "tinystr",
 "writeable",
 "yoke",
 "zerofrom",
 "zerotrie",
 "zerovec",
]

[[package]]
name = "idna"
version = "1.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3b0875f23caa03898994f6ddc501886a45c7d3d62d04d2d90788d47be1b1e4de"
dependencies = [
 "idna_adapter",
 "smallvec",
 "utf8_iter",
]

[[package]]
name = "idna_adapter"
version = "1.2.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3acae9609540aa318d1bc588455225fb2085b9ed0c4f6bd0d9d5bcd86f1a0344"
dependencies = [
 "icu_normalizer",
 "icu_properties",
]

[[package]]
name = "indexmap"
version = "2.11.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4b0f83760fb341a774ed326568e19f5a863af4a952def8c39f9ab92fd95b88e5"
dependencies = [
 "equivalent",
 "hashbrown",
]

[[package]]
name = "io-uring"
version = "0.7.10"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "046fa2d4d00aea763528b4950358d0ead425372445dc8ff86312b3c69ff7727b"
dependencies = [
 "bitflags 2.9.4",
 "cfg-if",
 "libc",
]

[[package]]
name = "ipnet"
version = "2.11.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "469fb0b9cefa57e3ef31275ee7cacb78f2fdca44e4765491884a2b119d4eb130"

[[package]]
name = "itoa"
version = "1.0.15"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4a5f13b858c8d314ee3e8f639011f7ccefe71f97f96e50151fb991f267928e2c"

[[package]]
name = "js-sys"
version = "0.3.81"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ec48937a97411dcb524a265206ccd4c90bb711fca92b2792c407f268825b9305"
dependencies = [
 "once_cell",
 "wasm-bindgen",
]

[[package]]
name = "lazy_static"
version = "1.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bbd2bcb4c963f2ddae06a2efc7e9f3591312473c50c6685e1f298068316e66fe"

[[package]]
name = "libc"
version = "0.2.176"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "58f929b4d672ea937a23a1ab494143d968337a5f47e56d0815df1e0890ddf174"

[[package]]
name = "linux-raw-sys"
version = "0.11.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "df1d3c3b53da64cf5760482273a98e575c651a67eec7f77df96b5b642de8f039"

[[package]]
name = "litemap"
version = "0.8.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "241eaef5fd12c88705a01fc1066c48c4b36e0dd4377dcdc7ec3942cea7a69956"

[[package]]
name = "lock_api"
version = "0.4.14"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "224399e74b87b5f3557511d98dff8b14089b3dadafcab6bb93eab67d3aace965"
dependencies = [
 "scopeguard",
]

[[package]]
name = "log"
version = "0.4.28"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "34080505efa8e45a4b816c349525ebe327ceaa8559756f0356cba97ef3bf7432"


<<TRUNCATED: max_file_lines=800>>
```

### 📄 merges/hausKI-audio_merge_2510262237__scripts.md

**Größe:** 25 KB | **md5:** `2e7cc856053737f62b552715caf05152`

```markdown
### 📄 scripts/README.md

**Größe:** 3 KB | **md5:** `2d0e4353735c419bcc3eac7a0f49ca4b`

```markdown
# scripts/

Geplante Helfer:

- `audio-mode`  → Pulse/ALSA umschalten (MOTU M2), Mopidy neustarten.
- `playlist-from-list` → Textliste in Qobuz-Playlist (via Mopidy RPC).
- `rec-start` / `rec-stop` → Audioaufnahme (arecord/pw-record).

## audio-mode

```bash
./audio-mode pulse           # PulseAudio Modus setzen
./audio-mode alsa            # ALSA Bitperfect Modus setzen
./audio-mode show            # aktuellen Output anzeigen
./audio-mode alsa --restart  # Mopidy nach dem Umschalten neustarten
```

Optionen:

- `--config` Pfad zu `mopidy.conf` (Default: `~/.config/mopidy/mopidy.conf`)
- `--alsa-output` Ziel-String für ALSA (Default:
  `alsasink device=hw:MOTU_M2,0`)
- `--pulse-output` Ziel-String für Pulse (Default: `pulsesink`)

## playlist-from-list

```bash
./playlist-from-list "HiRes Night" --input tracks.txt --replace
cat tracks.txt | ./playlist-from-list "HiRes Night" --scheme qobuz
```

Erwartet eine Textliste mit Mopidy-URIs (z. B. `qobuz:track:…`), jeweils
eine Zeile. Leere Zeilen und `#`-Kommentare werden ignoriert.

Optionen:

- `--input` Quelle (Datei oder `-` für stdin)
- `--scheme` Uri-Scheme-Hint (`m3u`, `qobuz`, …)
- `--replace` bestehende Playlist gleichen Namens leeren & überschreiben
- `--rpc-url` Ziel-Endpunkt (`http://127.0.0.1:6680/mopidy/rpc`)
- `--dry-run` nur anzeigen, wie viele Tracks gesendet würden

## rec-start / rec-stop

```bash
./rec-start --rate 96000 --channels 2
./rec-start --device alsa_output.usb-MOTU.M2-00.pro-output-0 --extra --latency=128
./rec-start --dry-run --json        # Smoke-Test ohne Aufnahme
./rec-stop --dry-run --json         # zeigt Signalplan (greift nicht ein)
./rec-stop                 # schickt SIGINT, wartet 5s, räumt PID-Datei auf
./rec-stop --force         # eskaliert zu SIGKILL, falls nötig
```

`rec-start` nutzt `pw-record` (PipeWire) und legt die PID in
`~/.cache/hauski-audio/recording.pid`. Standardziel ist `$AUDIO_RECORD_DIR`
(Default `~/Music/Recordings`) mit Zeitstempel und `.wav`-Extension.

Optionen (`rec-start`):

- `--output` fixer Dateiname (sonst Auto-Name)
- `--rate`, `--channels`, `--format` → direkt an `pw-record`
- `--device` PipeWire-Node (→ `--target`)
- `--pw-binary` alternativer Befehl (Default `pw-record`)
- `--extra` zusätzliche Argumente (mehrfach möglich)
- `--force` räumt verwaiste PID-Dateien, ohne laufende Aufnahme
- `--dry-run` zeigt Kommando & Ziel, startet nichts (`--json` für
  maschinenlesbar)

Optionen (`rec-stop`):

- `--signal` Grundsignal (`INT`/`TERM`/`KILL`)
- `--timeout` Wartezeit vor Eskalation
- `--force` sende am Ende `SIGKILL`, falls nötig
- `--dry-run`/`--json` zeigen den Signalplan ohne Prozesszugriff

**Smoke-Test:** `just rec-smoke` führt beide Skripte im Dry-Run aus
(CI-freundlich, kein Audio nötig).
```

### 📄 scripts/audio-mode

**Größe:** 7 KB | **md5:** `0df75dbf6b59c671e2126185080bd954`

```plaintext
#!/usr/bin/env python3
"""Toggle Mopidy audio output between PulseAudio and ALSA."""
from __future__ import annotations

import argparse
import subprocess
import sys
import re
from pathlib import Path

DEFAULT_CONFIG = Path.home() / ".config" / "mopidy" / "mopidy.conf"
DEFAULT_PULSE = "pulsesink"
DEFAULT_ALSA_FALLBACK = "alsasink device=hw:MOTU_M2,0"
DEFAULT_CARD_HINT = "M2"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Switch Mopidy's audio sink between PulseAudio and ALSA modes.",
    )
    parser.add_argument(
        "mode",
        nargs="?",
        choices=["pulse", "alsa", "show"],
        default="alsa",
        help="Target mode or show current configuration (default: alsa).",
    )
    parser.add_argument(
        "--config",
        default=str(DEFAULT_CONFIG),
        help="Path to mopidy.conf (default: %(default)s).",
    )
    parser.add_argument(
        "--alsa-output",
        default=None,
        help="ALSA output string to write when mode=alsa (auto-detect MOTU when unset).",
    )
    parser.add_argument(
        "--pulse-output",
        default=DEFAULT_PULSE,
        help="Pulse output string to write when mode=pulse.",
    )
    parser.add_argument(
        "--restart",
        dest="restart",
        action="store_true",
        help="Restart Mopidy via systemctl --user after writing audio mode.",
    )
    parser.add_argument(
        "--no-restart",
        dest="restart",
        action="store_false",
        help="Skip Mopidy restart (default: restart)",
    )
    parser.set_defaults(restart=True)
    parser.add_argument(
        "--control-pipewire",
        dest="control_pipewire",
        action="store_true",
        help="Toggle PipeWire/Pulse services when switching modes (default: enabled).",
    )
    parser.add_argument(
        "--no-control-pipewire",
        dest="control_pipewire",
        action="store_false",
        help="Do not manage PipeWire services.",
    )
    parser.set_defaults(control_pipewire=True)
    parser.add_argument(
        "--card-hint",
        default=DEFAULT_CARD_HINT,
        help="Substring to detect MOTU card via aplay -l (default: %(default)s).",
    )
    parser.add_argument(
        "--aplay-bin",
        default="aplay",
        help="Path to aplay executable for card detection (default: %(default)s).",
    )
    return parser.parse_args()


def load_lines(path: Path) -> list[str]:
    try:
        return path.read_text().splitlines(keepends=True)
    except FileNotFoundError:
        print(f"Config file not found: {path}", file=sys.stderr)
        sys.exit(2)


def find_section(lines: list[str], section: str) -> tuple[int | None, int | None]:
    section_header = f"[{section.lower()}]"
    start = None
    for idx, raw in enumerate(lines):
        if raw.strip().lower() == section_header:
            start = idx
            break
    if start is None:
        return None, None
    end = len(lines)
    for idx in range(start + 1, len(lines)):
        if lines[idx].lstrip().startswith("["):
            end = idx
            break
    return start, end


def extract_output(lines: list[str], start: int, end: int) -> tuple[int | None, str | None]:
    for idx in range(start + 1, end):
        stripped = lines[idx].split("#", 1)[0].split(";", 1)[0].strip()
        if stripped.lower().startswith("output"):
            parts = stripped.split("=", 1)
            if len(parts) == 2:
                return idx, parts[1].strip()
            return idx, None
    return None, None


def write_mode(path: Path, mode: str, pulse_output: str, alsa_output: str) -> str:
    lines = load_lines(path)
    start, end = find_section(lines, "audio")
    if start is None or end is None:
        print("No [audio] section found in config; unable to continue.", file=sys.stderr)
        sys.exit(2)

    output_idx, current_value = extract_output(lines, start, end)
    desired = pulse_output if mode == "pulse" else alsa_output

    if output_idx is None:
        insert_at = end
        newline = lines[start][-1:] if lines[start].endswith("\n") else "\n"
        lines.insert(insert_at, f"output = {desired}{newline}")
    else:
        newline = "\n" if lines[output_idx].endswith("\n") else ""
        lines[output_idx] = f"output = {desired}{newline}"

    path.write_text("".join(lines))
    return current_value or "(unset)"


def show_mode(path: Path) -> None:
    lines = load_lines(path)
    start, end = find_section(lines, "audio")
    if start is None or end is None:
        print("[audio] section missing")
        return
    _, current_value = extract_output(lines, start, end)
    print(current_value or "(output unset)")


def restart_mopidy() -> None:
    result = subprocess.run(
        ["systemctl", "--user", "restart", "mopidy"],
        check=False,
    )
    if result.returncode != 0:
        print("systemctl did not exit cleanly.", file=sys.stderr)
        sys.exit(result.returncode)


def manage_service(action: str, service: str) -> None:
    subprocess.run(
        ["systemctl", "--user", action, service],
        check=False,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )


def detect_motu_device(aplay_bin: str, card_hint: str) -> str | None:
    try:
        result = subprocess.run(
            [aplay_bin, "-l"],
            check=False,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            text=True,
        )
    except FileNotFoundError:
        return None

    if result.returncode != 0:
        return None

    pattern = re.compile(r"card\s+(\d+):.*" + re.escape(card_hint), re.IGNORECASE)
    for line in result.stdout.splitlines():
        match = pattern.search(line)
        if match:
            card_idx = match.group(1)
            return f"hw:{card_idx},0"
    return None


def control_pipewire(mode: str, enabled: bool) -> None:
    if not enabled:
        return

    if mode == "alsa":
        manage_service("stop", "pipewire-pulse")
        manage_service("stop", "pipewire")
    elif mode == "pulse":
        manage_service("start", "pipewire")
        manage_service("start", "pipewire-pulse")


def main() -> None:
    args = parse_args()
    config_path = Path(args.config).expanduser()

    if args.mode == "show":
        show_mode(config_path)
        return

    control_pipewire(args.mode, args.control_pipewire)

    alsa_output = args.alsa_output
    detected_device = None
    if args.mode == "alsa" and alsa_output is None:
        detected_device = detect_motu_device(args.aplay_bin, args.card_hint)
        if detected_device is None:
            alsa_output = DEFAULT_ALSA_FALLBACK
        else:
            alsa_output = f"alsasink device={detected_device}"

    if args.mode == "pulse":
        desired_output = args.pulse_output
    else:
        desired_output = alsa_output or DEFAULT_ALSA_FALLBACK

    previous = write_mode(
        config_path,
        args.mode,
        pulse_output=args.pulse_output,
        alsa_output=desired_output,
    )

    suffix = ""
    if detected_device:
        suffix = f" (card {detected_device})"
    print(f"Audio output changed from '{previous}' to '{args.mode}'{suffix}.")

    if args.restart:
        restart_mopidy()


if __name__ == "__main__":
    main()
```

### 📄 scripts/playlist-from-list

**Größe:** 5 KB | **md5:** `741d098ea12ff3aab2506ba6a9a1aaed`

```plaintext
#!/usr/bin/env python3
"""Create a Mopidy playlist from a text file of track URIs."""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from urllib import error, request

DEFAULT_RPC_URL = "http://127.0.0.1:6680/mopidy/rpc"


class MopidyClient:
    def __init__(self, rpc_url: str) -> None:
        self.rpc_url = rpc_url
        self._next_id = 1

    def call(self, method: str, params: dict | None = None) -> object:
        payload = {
            "jsonrpc": "2.0",
            "id": self._next_id,
            "method": method,
        }
        self._next_id += 1
        if params is not None:
            payload["params"] = params

        data = json.dumps(payload).encode("utf-8")
        req = request.Request(
            self.rpc_url,
            data=data,
            headers={"Content-Type": "application/json"},
        )
        try:
            with request.urlopen(req) as resp:
                response = json.load(resp)
        except error.HTTPError as exc:
            message = exc.read().decode("utf-8", errors="replace")
            raise RuntimeError(f"HTTP error {exc.code}: {message}") from exc
        except error.URLError as exc:  # network or connection problem
            raise RuntimeError(f"Failed to reach Mopidy at {self.rpc_url}: {exc.reason}") from exc

        if "error" in response:
            err = response["error"]
            raise RuntimeError(f"Mopidy RPC error: {err.get('message')} ({err.get('code')})")

        return response.get("result")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Create or replace a Mopidy playlist from newline-delimited track URIs.",
    )
    parser.add_argument("name", help="Playlist name to create or replace.")
    parser.add_argument(
        "--input",
        "-i",
        default="-",
        help="Source file with track URIs (default: stdin).",
    )
    parser.add_argument(
        "--rpc-url",
        default=DEFAULT_RPC_URL,
        help=f"Mopidy JSON-RPC endpoint (default: {DEFAULT_RPC_URL}).",
    )
    parser.add_argument(
        "--scheme",
        default=None,
        help="Optional uri_scheme hint for Mopidy (e.g. 'm3u', 'qobuz').",
    )
    parser.add_argument(
        "--replace",
        action="store_true",
        help="Replace existing playlist with the same name if one exists.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Show actions without calling Mopidy.",
    )
    return parser.parse_args()


def load_uris(source: str) -> list[str]:
    if source == "-":
        text = sys.stdin.read()
    else:
        text = Path(source).read_text()
    lines = [line.strip() for line in text.splitlines()]
    return [line for line in lines if line and not line.startswith("#")]


def find_existing(client: MopidyClient, name: str) -> dict | None:
    playlists = client.call("core.playlists.as_list") or []
    for playlist in playlists:
        if playlist.get("name") == name:
            return playlist
    return None


def load_playlist(client: MopidyClient, uri: str) -> dict:
    playlist = client.call("core.playlists.lookup", {"uri": uri})
    if isinstance(playlist, dict):
        return playlist
    raise RuntimeError(f"Playlist lookup returned no result for {uri}")


def ensure_playlist(
    client: MopidyClient, *, name: str, scheme: str | None, replace: bool
) -> dict:
    existing = find_existing(client, name)
    if existing and not replace:
        raise RuntimeError(
            f"Playlist '{name}' already exists. Use --replace to overwrite it."
        )
    if existing and replace:
        playlist_uri = existing.get("uri")
        if not playlist_uri:
            raise RuntimeError("Existing playlist reported without URI; aborting.")
        playlist = load_playlist(client, playlist_uri)
        playlist["tracks"] = []
        return playlist

    params = {"name": name}
    if scheme:
        params["uri_scheme"] = scheme
    created = client.call("core.playlists.create", params)
    if not isinstance(created, dict):
        raise RuntimeError("Mopidy returned unexpected response when creating playlist")
    created.setdefault("tracks", [])
    return created


def to_track(uri: str) -> dict:
    return {"__model__": "Track", "uri": uri}


def save_playlist(client: MopidyClient, playlist: dict) -> dict:
    playlist.setdefault("__model__", "Playlist")
    return client.call("core.playlists.save", {"playlist": playlist})


def main() -> None:
    args = parse_args()
    uris = load_uris(args.input)
    if not uris:
        print("No track URIs found in input.", file=sys.stderr)
        sys.exit(1)

    if args.dry_run:
        print(f"Would send {len(uris)} tracks to playlist '{args.name}'.")
        return

    client = MopidyClient(args.rpc_url)
    playlist = ensure_playlist(
        client,
        name=args.name,
        scheme=args.scheme,
        replace=args.replace,
    )
    playlist["tracks"] = [to_track(uri) for uri in uris]
    saved = save_playlist(client, playlist)
    uri = saved.get("uri") if isinstance(saved, dict) else None
    print(
        f"Playlist '{args.name}' updated with {len(uris)} tracks." +
        (f" URI: {uri}" if uri else "")
    )


if __name__ == "__main__":
    try:
        main()
    except RuntimeError as exc:
        print(str(exc), file=sys.stderr)
        sys.exit(2)
```

### 📄 scripts/rec-start

**Größe:** 5 KB | **md5:** `cc635d7df0d6925f94b62a9619128d97`

```plaintext
#!/usr/bin/env python3
"""Start a long-running audio capture using PipeWire's pw-record."""
from __future__ import annotations

import argparse
import datetime as dt
import json
import os
import subprocess
from pathlib import Path

STATE_DIR = Path.home() / ".cache" / "hauski-audio"
PID_FILE = STATE_DIR / "recording.pid"
DEFAULT_RECORD_DIR = Path(os.environ.get("AUDIO_RECORD_DIR", "~/Music/Recordings")).expanduser()
DEFAULT_EXTENSION = os.environ.get("AUDIO_RECORD_EXT", "wav")


class RecorderStateError(RuntimeError):
    pass


def ensure_state_dir() -> None:
    STATE_DIR.mkdir(parents=True, exist_ok=True)


def running_pid() -> int | None:
    if PID_FILE.exists():
        try:
            pid = int(PID_FILE.read_text().strip())
        except ValueError:
            PID_FILE.unlink(missing_ok=True)
            return None
        if pid <= 0:
            PID_FILE.unlink(missing_ok=True)
            return None
        try:
            os.kill(pid, 0)
        except OSError:
            PID_FILE.unlink(missing_ok=True)
            return None
        return pid
    return None


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Launch pw-record to capture audio until rec-stop is invoked.",
    )
    parser.add_argument(
        "--output",
        "-o",
        help="Output file path. Defaults to AUDIO_RECORD_DIR/recording-<timestamp>.wav.",
    )
    parser.add_argument(
        "--rate",
        type=int,
        default=96000,
        help="Sample rate in Hz (default: 96000).",
    )
    parser.add_argument(
        "--channels",
        type=int,
        default=2,
        help="Channel count (default: 2).",
    )
    parser.add_argument(
        "--format",
        default="S24_LE",
        help="Sample format passed to pw-record (default: S24_LE).",
    )
    parser.add_argument(
        "--device",
        help="Optional PipeWire node target (passes --target to pw-record).",
    )
    parser.add_argument(
        "--pw-binary",
        default=os.environ.get("PW_RECORD_BINARY", "pw-record"),
        help="Alternate pw-record binary (default: pw-record).",
    )
    parser.add_argument(
        "--extra",
        action="append",
        default=[],
        help="Additional arguments forwarded to pw-record (repeatable).",
    )
    parser.add_argument(
        "--force",
        action="store_true",
        help="Overwrite existing PID state if recorder looks stale.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print resolved command and exit without launching pw-record.",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Emit dry-run details as JSON (only with --dry-run).",
    )
    return parser.parse_args()


def resolve_output(path_arg: str | None) -> Path:
    if path_arg:
        path = Path(path_arg).expanduser()
    else:
        DEFAULT_RECORD_DIR.mkdir(parents=True, exist_ok=True)
        timestamp = dt.datetime.now().strftime("%Y%m%d-%H%M%S")
        path = DEFAULT_RECORD_DIR / f"recording-{timestamp}.{DEFAULT_EXTENSION}"
    if path.exists():
        raise RecorderStateError(f"Output file already exists: {path}")
    path.parent.mkdir(parents=True, exist_ok=True)
    return path


def build_command(args: argparse.Namespace, output: Path) -> list[str]:
    cmd = [args.pw_binary, "--rate", str(args.rate), "--channels", str(args.channels), "--format", args.format]
    if args.device:
        cmd.extend(["--target", args.device])
    if args.extra:
        cmd.extend(args.extra)
    cmd.append(str(output))
    return cmd


def launch(cmd: list[str]) -> int:
    try:
        proc = subprocess.Popen(cmd)
    except FileNotFoundError as exc:
        raise RecorderStateError(f"Unable to launch {cmd[0]}: {exc}") from exc
    except Exception as exc:  # pragma: no cover - defensive
        raise RecorderStateError(f"Failed to start recorder: {exc}") from exc
    return proc.pid


def ensure_stopped(force: bool) -> None:
    pid = running_pid()
    if pid is None:
        return
    if not force:
        raise RecorderStateError(
            f"Recording already running (pid {pid}). Use rec-stop first or pass --force to clear stale state."
        )
    PID_FILE.unlink(missing_ok=True)


def main() -> None:
    args = parse_args()
    ensure_state_dir()
    try:
        ensure_stopped(args.force)
        output = resolve_output(args.output)
        cmd = build_command(args, output)
    except RecorderStateError as exc:
        print(str(exc))
        raise SystemExit(1) from exc

    if args.dry_run:
        payload = {
            "output": str(output),
            "command": cmd,
            "binary": cmd[0],
        }
        if args.json:
            print(json.dumps(payload, indent=2))
        else:
            print(f"Would run: {' '.join(cmd)}")
            print(f"Output file: {output}")
        return

    try:
        pid = launch(cmd)
    except RecorderStateError as exc:
        print(str(exc))
        raise SystemExit(1) from exc
    PID_FILE.write_text(f"{pid}\n")
    print(f"Recording started (pid {pid}) → {output}")


if __name__ == "__main__":
    main()
```

### 📄 scripts/rec-stop

**Größe:** 4 KB | **md5:** `7bd0f15030adf90ebddafc0ff465ef17`

```plaintext
#!/usr/bin/env python3
"""Stop the active pw-record session launched via rec-start."""
from __future__ import annotations

import argparse
import json
import os
import signal
import time
from pathlib import Path

STATE_DIR = Path.home() / ".cache" / "hauski-audio"
PID_FILE = STATE_DIR / "recording.pid"


class RecorderStateError(RuntimeError):
    pass


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Stop the current recorder if running.")
    parser.add_argument(
        "--timeout",
        type=float,
        default=5.0,
        help="Seconds to wait for recorder to exit after sending SIGINT (default: 5).",
    )
    parser.add_argument(
        "--signal",
        choices=["INT", "TERM", "KILL"],
        default="INT",
        help="Primary signal to send (default: INT).",
    )
    parser.add_argument(
        "--force",
        action="store_true",
        help="If recorder ignores the primary signal, escalate to SIGKILL at the end of timeout.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Output the signal plan without touching the process.",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Emit dry-run details as JSON (only with --dry-run).",
    )
    return parser.parse_args()


def read_pid() -> int:
    if not PID_FILE.exists():
        raise RecorderStateError("No recorder PID state found. Is rec-start running?")
    try:
        pid = int(PID_FILE.read_text().strip())
    except ValueError:
        PID_FILE.unlink(missing_ok=True)
        raise RecorderStateError("PID file contained invalid data; cleared stale state.")
    return pid


def process_alive(pid: int) -> bool:
    try:
        os.kill(pid, 0)
    except OSError:
        return False
    return True


def send_signal(pid: int, signame: str) -> None:
    sig = getattr(signal, f"SIG{signame}")
    try:
        os.kill(pid, sig)
    except ProcessLookupError:
        pass
    except PermissionError as exc:
        raise RecorderStateError(f"Not permitted to signal process {pid}: {exc}")


def wait_exit(pid: int, timeout: float) -> bool:
    deadline = time.time() + timeout
    while time.time() < deadline:
        if not process_alive(pid):
            return True
        time.sleep(0.2)

<<TRUNCATED: max_file_lines=800>>
```

### 📄 merges/hausKI-audio_merge_2510262237__tests.md

**Größe:** 9 KB | **md5:** `84fe3fc9385795134d57e52685395c49`

```markdown
### 📄 tests/__init__.py

**Größe:** 0 B | **md5:** `d41d8cd98f00b204e9800998ecf8427e`

```python

```

### 📄 tests/test_audio_mode.py

**Größe:** 4 KB | **md5:** `b910134cd1308867722fc26b0d722166`

```python
import os
import subprocess
from pathlib import Path
from typing import NamedTuple

import pytest

REPO_ROOT = Path(__file__).resolve().parents[1]
SCRIPTS_DIR = REPO_ROOT / "scripts"
EXIT_CODE_INVALID_CONFIG = 2


class SubprocessResult(NamedTuple):
    """Data class for subprocess results."""

    returncode: int
    stdout: str
    stderr: str


def run_audio_mode(args: list[str], home: Path) -> SubprocessResult:
    """Run the audio-mode script in a subprocess."""
    env = os.environ.copy()
    env.update(
        {
            "HOME": str(home),
        },
    )
    cmd = ["python3", str(SCRIPTS_DIR / "audio-mode"), *args]
    result = subprocess.run(
        cmd,
        check=False,
        capture_output=True,
        text=True,
        env=env,
        cwd=REPO_ROOT,
    )
    return SubprocessResult(
        returncode=result.returncode,
        stdout=result.stdout,
        stderr=result.stderr,
    )


@pytest.fixture
def home(tmp_path: Path) -> Path:
    """Create a temporary home directory."""
    home_dir = tmp_path / "home"
    home_dir.mkdir()
    return home_dir


def write_config(home: Path, contents: str) -> Path:
    """Write a mopidy.conf file to the temp home dir."""
    config_dir = home / "config"
    config_dir.mkdir(parents=True, exist_ok=True)
    config_path = config_dir / "mopidy.conf"
    config_path.write_text(contents)
    return config_path


def test_audio_mode_show_reports_current_output(home: Path) -> None:
    """Verify the 'show' command prints the current output."""
    config = write_config(home, "[audio]\noutput = pulsesink\n")

    result = run_audio_mode(["show", "--config", str(config)], home)

    assert result.returncode == 0, result.stderr
    assert result.stdout.strip() == "pulsesink"


def test_audio_mode_switch_to_alsa_overwrites_output(home: Path) -> None:
    """Verify switching to ALSA mode correctly updates the config."""
    config = write_config(home, "[audio]\noutput = pulsesink\n")

    result = run_audio_mode(
        [
            "alsa",
            "--config",
            str(config),
            "--no-restart",
            "--no-control-pipewire",
            "--alsa-output",
            "alsasink device=hw:1,0",
        ],
        home,
    )

    assert result.returncode == 0, result.stderr
    assert "Audio output changed" in result.stdout
    assert "'alsa'" in result.stdout
    assert "alsasink device=hw:1,0" in config.read_text()


def test_audio_mode_switch_to_pulse_inserts_output_when_missing(
    home: Path,
) -> None:
    """Verify switching to Pulse mode adds the output if missing."""
    config = write_config(home, "[audio]\n# no output here yet\n")

    result = run_audio_mode(
        [
            "pulse",
            "--config",
            str(config),
            "--no-restart",
            "--no-control-pipewire",
            "--pulse-output",
            "pulsesink jumbo",
        ],
        home,
    )

    assert result.returncode == 0, result.stderr
    assert "'pulse'" in result.stdout
    assert "pulsesink jumbo" in config.read_text()


def test_audio_mode_missing_config_errors(home: Path) -> None:
    """Verify the script exits gracefully for a missing config file."""
    missing = home / "config" / "absent.conf"

    result = run_audio_mode(
        [
            "pulse",
            "--config",
            str(missing),
            "--no-restart",
            "--no-control-pipewire",
        ],
        home,
    )

    assert result.returncode == EXIT_CODE_INVALID_CONFIG
    assert "Config file not found" in result.stderr


def test_audio_mode_missing_audio_section_errors(home: Path) -> None:
    """Verify the script exits gracefully if the [audio] section is missing."""
    config = write_config(home, "[core]\ncache_dir = /tmp\n")

    result = run_audio_mode(
        [
            "pulse",
            "--config",
            str(config),
            "--no-restart",
            "--no-control-pipewire",
        ],
        home,
    )

    assert result.returncode == EXIT_CODE_INVALID_CONFIG
    assert "No [audio] section" in result.stderr
```

### 📄 tests/test_rec_scripts.py

**Größe:** 5 KB | **md5:** `6156983ea8db94963cff202a70bcf904`

```python
import json
import os
import subprocess
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[1]
SCRIPTS_DIR = REPO_ROOT / "scripts"


def run_script(
    script: str,
    args: list[str],
    home: Path,
    extra_env: dict[str, str] | None = None,
) -> subprocess.CompletedProcess[str]:
    """Run a script in a controlled environment."""
    env = os.environ.copy()
    env.update(
        {
            "HOME": str(home),
            "AUDIO_RECORD_DIR": str(home / "recordings"),
            "AUDIO_RECORD_EXT": "wav",
            "PW_RECORD_BINARY": "pw-record",
        },
    )
    if extra_env:
        env.update(extra_env)
    cmd = ["python3", str(SCRIPTS_DIR / script), *args]
    return subprocess.run(
        cmd,
        check=False,
        capture_output=True,
        text=True,
        env=env,
        cwd=REPO_ROOT,
    )


@pytest.fixture
def home(tmp_path: Path) -> Path:
    """Create a temporary home directory."""
    home_dir = tmp_path / "home"
    home_dir.mkdir()
    return home_dir


def test_rec_start_dry_run_json(home: Path) -> None:
    """Verify that `rec-start --dry-run --json` produces sane output."""
    target = home / "output.wav"
    result = run_script(
        "rec-start",
        ["--dry-run", "--json", "--output", str(target)],
        home,
    )
    assert result.returncode == 0, result.stderr
    payload = json.loads(result.stdout)
    assert payload["output"] == str(target)
    assert payload["command"][0] == "pw-record"
    assert payload["command"][-1] == str(target)


def test_rec_start_detects_running_process(home: Path) -> None:
    """Verify that `rec-start` aborts if a recording is already running."""
    proc = subprocess.Popen(["sleep", "5"])  # noqa: S607 (controlled command)
    try:
        state_dir = home / ".cache" / "hauski-audio"
        state_dir.mkdir(parents=True)
        (state_dir / "recording.pid").write_text(f"{proc.pid}\n")
        result = run_script("rec-start", ["--dry-run"], home)
        assert result.returncode == 1
        assert "Recording already running" in result.stdout
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=1)
        except subprocess.TimeoutExpired:
            proc.kill()


def test_rec_start_force_clears_running_process(home: Path) -> None:
    """Verify that `rec-start --force` removes an existing PID file."""
    proc = subprocess.Popen(["sleep", "5"])  # noqa: S607
    try:
        state_dir = home / ".cache" / "hauski-audio"
        pid_file = state_dir / "recording.pid"
        state_dir.mkdir(parents=True)
        pid_file.write_text(f"{proc.pid}\n")

        result = run_script(
            "rec-start",
            [
                "--dry-run",
                "--force",
                "--output",
                str(home / "recordings" / "forced.wav"),
            ],
            home,
        )

        assert result.returncode == 0, result.stderr
        assert "Would run:" in result.stdout
        assert not pid_file.exists()
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=1)
        except subprocess.TimeoutExpired:
            proc.kill()


def test_rec_start_rejects_existing_output(home: Path) -> None:
    """Verify that `rec-start` aborts if the output file already exists."""
    target = home / "recordings" / "exists.wav"
    target.parent.mkdir(parents=True)
    target.write_bytes(b"test")

    result = run_script(
        "rec-start",
        ["--dry-run", "--output", str(target)],
        home,
    )

    assert result.returncode == 1
    assert "Output file already exists" in result.stdout


def test_rec_stop_dry_run_json(home: Path) -> None:
    """Verify that `rec-stop --dry-run --json` produces sane output."""
    proc = subprocess.Popen(["sleep", "5"])  # noqa: S607
    state_dir = home / ".cache" / "hauski-audio"
    state_dir.mkdir(parents=True)
    (state_dir / "recording.pid").write_text(f"{proc.pid}\n")
    try:
        result = run_script("rec-stop", ["--dry-run", "--json"], home)
        assert result.returncode == 0, result.stderr
        payload = json.loads(result.stdout)
        assert payload["pid"] == proc.pid
        assert payload["signal"] == "INT"
        assert payload["force"] is False
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=1)
        except subprocess.TimeoutExpired:
            proc.kill()


def test_rec_stop_requires_pid_file(home: Path) -> None:
    """Verify that `rec-stop` fails if no PID file is present."""
    result = run_script("rec-stop", [], home)
    assert result.returncode == 1
    assert "No recorder PID state found" in result.stdout
```
```

### 📄 merges/hausKI-audio_merge_2510262237__tools_systemd.md

**Größe:** 538 B | **md5:** `d492a100f77d3ba529344c1978a576d7`

```markdown
### 📄 tools/systemd/hauski-backend.service

**Größe:** 407 B | **md5:** `73c0533d49ac86ed402c50f0f7831ce1`

```plaintext
[Unit]
Description=Hauski Audio Backend (axum)
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
# Passe das Repo/Wurzelverzeichnis an (WorkingDirectory sollte die Skripte finden).
WorkingDirectory=%h/repos/hauski-audio
EnvironmentFile=%h/.config/hauski-audio/backend.env
ExecStart=%h/.local/bin/hauski-backend
Restart=on-failure
RestartSec=5

[Install]
WantedBy=default.target
```
```

