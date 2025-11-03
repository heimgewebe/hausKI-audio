use crate::scripts::constants::{
    DEFAULT_AUDIO_MODE_CMD, DEFAULT_PLAYLIST_CMD, DEFAULT_REC_START_CMD, DEFAULT_REC_STOP_CMD,
};
use std::env;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::Duration;
use thiserror::Error;
use url::Url;

// ... (rest of the file is the same)

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub bind_addr: SocketAddr,
    pub mopidy_rpc_url: Url,
    pub audio_mode_script: ScriptConfig,
    pub playlist_script: ScriptConfig,
    pub rec_start_script: ScriptConfig,
    pub rec_stop_script: ScriptConfig,
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
    /// Standard-Timeout in Millisekunden (klar benannt, keine versteckte Umrechnung)
    const DEFAULT_COMMAND_TIMEOUT_MS: u64 = 10_000;

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
                    .unwrap_or_else(|_| DEFAULT_AUDIO_MODE_CMD.to_string()),
            ),
        };

        let playlist_script = ScriptConfig {
            program: PathBuf::from(
                env::var("HAUSKI_PLAYLIST_FROM_LIST_CMD")
                    .or_else(|_| env::var("HAUSKI_PLAYLIST_CMD"))
                    .unwrap_or_else(|_| DEFAULT_PLAYLIST_CMD.to_string()),
            ),
        };

        let rec_start_script = ScriptConfig {
            program: PathBuf::from(
                env::var("HAUSKI_REC_START_CMD")
                    .unwrap_or_else(|_| DEFAULT_REC_START_CMD.to_string()),
            ),
        };

        let rec_stop_script = ScriptConfig {
            program: PathBuf::from(
                env::var("HAUSKI_REC_STOP_CMD")
                    .unwrap_or_else(|_| DEFAULT_REC_STOP_CMD.to_string()),
            ),
        };

        let timeout_ms = std::env::var("HAUSKI_COMMAND_TIMEOUT_MS")
            .ok()
            .and_then(|raw| raw.parse().ok())
            .unwrap_or(Self::DEFAULT_COMMAND_TIMEOUT_MS);

        let check_mopidy_health = env_bool("HAUSKI_CHECK_MOPIDY_HEALTH", true);

        Ok(Self {
            bind_addr,
            mopidy_rpc_url,
            audio_mode_script,
            playlist_script,
            rec_start_script,
            rec_stop_script,
            script_workdir: workdir,
            command_timeout: Duration::from_millis(timeout_ms),
            check_mopidy_health,
        })
    }
    pub fn validate(&self) -> Result<(), crate::error::AppError> {
        let scripts = [
            &self.audio_mode_script,
            &self.playlist_script,
            &self.rec_start_script,
            &self.rec_stop_script,
        ];

        for script_config in scripts {
            let p = script_config.resolve_with(&self.script_workdir);
            if !p.exists() {
                return Err(crate::error::AppError::Validation(format!(
                    "script not found: {}",
                    p.display()
                )));
            }
            // Unix: echte Ausführbarkeitsprüfung über Exec-Bits
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let meta = std::fs::metadata(&p)?;
                if meta.permissions().mode() & 0o111 == 0 {
                    return Err(crate::error::AppError::Validation(format!(
                        "script not executable: {}",
                        p.display()
                    )));
                }
            }
            // Non-Unix (z. B. Windows): kein Exec-Bit – minimal prüfen, dass es eine reguläre Datei ist.
            #[cfg(not(unix))]
            {
                let meta = std::fs::metadata(&p)?;
                if !meta.is_file() {
                    return Err(crate::error::AppError::Validation(format!(
                        "script is not a regular file: {}",
                        p.display()
                    )));
                }
            }
        }
        Ok(())
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
pub fn parse_bool(s: &str) -> Option<bool> {
    match s.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Some(true),
        "false" | "0" | "no" | "off" => Some(false),
        _ => None,
    }
}
pub fn env_bool(key: &str, default: bool) -> bool {
    env::var(key)
        .ok()
        .and_then(|raw| parse_bool(&raw))
        .unwrap_or(default)
}
