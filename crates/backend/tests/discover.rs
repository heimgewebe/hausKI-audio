use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::time::Duration;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tempfile::TempDir;
use tower::ServiceExt;
use url::Url;

use hauski_backend::config::{AppConfig, ScriptConfig};

// Helper function to write a dummy executable script
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

// Helper function to create a test configuration
fn test_config(dir: &TempDir) -> AppConfig {
    AppConfig {
        bind_addr: "127.0.0.1:0".parse().unwrap(),
        mopidy_rpc_url: Url::parse("http://127.0.0.1:6680/mopidy/rpc").unwrap(),
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

#[tokio::test]
async fn rejects_empty_seed() {
    let dir = TempDir::new().unwrap();
    // Create dummy scripts so config validation passes
    write_script(&dir, "audio-mode", "");
    write_script(&dir, "playlist-from-list", "");
    write_script(&dir, "rec-start", "");
    write_script(&dir, "rec-stop", "");

    let app = hauski_backend::build_router(test_config(&dir));

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/discover/similar?seed=%20") // Use a whitespace seed
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
