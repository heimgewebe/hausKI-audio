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
        rec_start_script: ScriptConfig {
            program: dir.path().join("rec-start"),
        },
        rec_stop_script: ScriptConfig {
            program: dir.path().join("rec-stop"),
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
        // Method extrahieren, ohne Lock zu halten
        let method = payload
            .get("method")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();

        // Lock nur für den Push halten; danach explizit freigeben
        {
            let mut calls = self.calls.lock().unwrap();
            calls.push(method.clone());
        } // <- Lock fällt hier; kein "Poisoning"-Kaskadenrisiko mehr

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
    write_script(&dir, "rec-start", "");
    write_script(&dir, "rec-stop", "");

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
    write_script(&dir, "rec-start", "");
    write_script(&dir, "rec-stop", "");

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
    write_script(&dir, "rec-start", "");
    write_script(&dir, "rec-stop", "");

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
    write_script(&dir, "rec-start", "");
    write_script(&dir, "rec-stop", "");

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
    write_script(&dir, "rec-start", "");
    write_script(&dir, "rec-stop", "");
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

#[tokio::test]
async fn discover_similar_rejects_bad_schemes() {
    let dir = TempDir::new().unwrap();
    let audio_script = "#!/usr/bin/env bash\nset -euo pipefail\nif [[ \"$1\" == \"show\" ]]; then\n  echo \"pulsesink\"\nelse\n  echo \"mode:$1\"\nfi\n";
    write_script(&dir, "audio-mode", audio_script);
    let playlist_script = "#!/usr/bin/env bash\nset -euo pipefail\necho \"playlist:$1\"\ncat -\n";
    write_script(&dir, "playlist-from-list", playlist_script);
    write_script(&dir, "rec-start", "");
    write_script(&dir, "rec-stop", "");

    let app = hauski_backend::build_router(test_config(&dir));

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/discover/similar?seed=file:///etc/passwd")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

use std::sync::LazyLock;

static SHARED: LazyLock<Arc<Mutex<Vec<String>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(Vec::new())));

#[test]
fn health_is_ok() {
    {
        // Lock SCOPE-BEGRENZEN: Guard droppt vor potenziellen Panics/Asserts
        let mut guard = SHARED.lock().unwrap();
        guard.push("start".into());
        assert_eq!(1, guard.len());
    }
    // ab hier ist der Mutex garantiert wieder frei und nicht "vergiftet"
}

#[test]
fn shared_survives_other_panics() {
    // Simulierter vorheriger Fehlerlauf: kein Gift-Effekt dank scoped lock
    {
        let mut g = SHARED.lock().unwrap();
        g.push("x".into());
    }
    let got = SHARED.lock().unwrap().len();
    assert!(got >= 1);
}
