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
