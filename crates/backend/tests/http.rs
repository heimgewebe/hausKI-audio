use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::time::Duration;

use axum::body::Body;
use http::{Request, StatusCode};
use http_body_util::BodyExt;
use httpmock::MockServer;
use serde_json::json;
use tempfile::TempDir;
use tower::ServiceExt;
use url::Url;

use hauski_backend::config::{AppConfig, ScriptConfig};
use hauski_backend::models::AudioMode;

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
async fn mode_endpoints_invoke_script() {
    let dir = TempDir::new().unwrap();
    let audio_script = "#!/usr/bin/env bash\nset -euo pipefail\nif [[ \"$1\" == \"show\" ]]; then\n  echo \"pulsesink\"\nelse\n  echo \"mode:$1\"\nfi\n";
    write_script(&dir, "audio-mode", audio_script);
    let playlist_script = "#!/usr/bin/env bash\nset -euo pipefail\necho \"playlist:$1\"\ncat -\n";
    write_script(&dir, "playlist-from-list", playlist_script);

    let mut app = hauski_backend::build_router(test_config(&dir));

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
    let body = post_response.into_body().collect().await.unwrap().to_bytes();
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

    let mut app = hauski_backend::build_router(test_config(&dir));
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

    let server = MockServer::start();
    let mopidy_url = Url::parse(&format!("{}/mopidy/rpc", server.base_url())).unwrap();

    let lookup_mock = server.mock(|when, then| {
        when.method("POST")
            .path("/mopidy/rpc")
            .json_body_partial(json!({"method": "core.library.lookup"}));
        then.status(200).json_body(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": [
                {
                    "__model__": "Track",
                    "uri": "qobuz:track:seed",
                    "name": "Seed Track",
                    "artists": [
                        {"name": "Seed Artist"}
                    ],
                    "album": {"name": "Seed Album"}
                }
            ],
        }));
    });

    let search_mock = server.mock(|when, then| {
        when.method("POST")
            .path("/mopidy/rpc")
            .json_body_partial(json!({"method": "core.library.search"}));
        then.status(200).json_body(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": [
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
            ],
        }));
    });

    let config = test_config_with(&dir, mopidy_url);
    let mut app = hauski_backend::build_router(config);

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

    lookup_mock.assert();
    search_mock.assert();
}
