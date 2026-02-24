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
    #[must_use]
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
        let get_env = |key: &str| env::var(key).ok();
        Self::from_source(&get_env, env::current_dir)
    }

    fn from_source<F, G>(get_env: &F, get_cwd: G) -> Result<Self, ConfigError>
    where
        F: Fn(&str) -> Option<String>,
        G: Fn() -> Result<PathBuf, std::io::Error>,
    {
        let bind_raw = get_env("HAUSKI_BACKEND_BIND")
            .or_else(|| get_env("HAUSKI_BIND"))
            .unwrap_or_else(|| Self::DEFAULT_BIND.into());
        let bind_addr: SocketAddr = bind_raw
            .parse()
            .map_err(|_| ConfigError::InvalidBindAddress(bind_raw.clone()))?;

        let mopidy_rpc_url = resolve_mopidy_rpc_url(get_env)?;

        let workdir = match get_env("HAUSKI_SCRIPT_WORKDIR") {
            Some(value) => PathBuf::from(value),
            None => get_cwd().map_err(ConfigError::WorkingDirectory)?,
        };

        let audio_mode_script = ScriptConfig {
            program: PathBuf::from(
                get_env("HAUSKI_AUDIO_MODE_CMD").unwrap_or_else(|| DEFAULT_AUDIO_MODE_CMD.into()),
            ),
        };

        let playlist_script = ScriptConfig {
            program: PathBuf::from(
                get_env("HAUSKI_PLAYLIST_FROM_LIST_CMD")
                    .or_else(|| get_env("HAUSKI_PLAYLIST_CMD"))
                    .unwrap_or_else(|| DEFAULT_PLAYLIST_CMD.into()),
            ),
        };

        let rec_start_script = ScriptConfig {
            program: PathBuf::from(
                get_env("HAUSKI_REC_START_CMD").unwrap_or_else(|| DEFAULT_REC_START_CMD.into()),
            ),
        };

        let rec_stop_script = ScriptConfig {
            program: PathBuf::from(
                get_env("HAUSKI_REC_STOP_CMD").unwrap_or_else(|| DEFAULT_REC_STOP_CMD.into()),
            ),
        };

        let timeout_ms = get_env("HAUSKI_COMMAND_TIMEOUT_MS")
            .and_then(|raw| raw.parse().ok())
            .unwrap_or(Self::DEFAULT_COMMAND_TIMEOUT_MS);

        let check_mopidy_health = env_bool_source("HAUSKI_CHECK_MOPIDY_HEALTH", true, get_env);

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
fn resolve_mopidy_rpc_url<F>(get_env: &F) -> Result<Url, ConfigError>
where
    F: Fn(&str) -> Option<String>,
{
    let direct = get_env("HAUSKI_MOPIDY_RPC_URL").or_else(|| get_env("MOPIDY_RPC_URL"));

    if let Some(raw) = direct {
        return Url::parse(&raw).map_err(|_| ConfigError::InvalidMopidyUrl(raw));
    }

    if let Some(base) = get_env("MOPIDY_HTTP_URL") {
        if let Ok(mut url) = Url::parse(&base) {
            url.set_path("/mopidy/rpc");
            return Ok(url);
        }
        return Err(ConfigError::InvalidMopidyUrl(base));
    }

    Url::parse(AppConfig::DEFAULT_MOPIDY_RPC)
        .map_err(|_| ConfigError::InvalidMopidyUrl(AppConfig::DEFAULT_MOPIDY_RPC.into()))
}
#[must_use]
pub fn parse_bool(s: &str) -> Option<bool> {
    match s.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Some(true),
        "false" | "0" | "no" | "off" => Some(false),
        _ => None,
    }
}
#[must_use]
pub fn env_bool(key: &str, default: bool) -> bool {
    let get_env = |k: &str| env::var(k).ok();
    env_bool_source(key, default, &get_env)
}

fn env_bool_source<F>(key: &str, default: bool, get_env: &F) -> bool
where
    F: Fn(&str) -> Option<String>,
{
    get_env(key)
        .and_then(|raw| parse_bool(&raw))
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;

    #[test]
    fn test_parse_bool() {
        assert_eq!(parse_bool("true"), Some(true));
        assert_eq!(parse_bool("1"), Some(true));
        assert_eq!(parse_bool("yes"), Some(true));
        assert_eq!(parse_bool("on"), Some(true));
        assert_eq!(parse_bool("TRUE "), Some(true));

        assert_eq!(parse_bool("false"), Some(false));
        assert_eq!(parse_bool("0"), Some(false));
        assert_eq!(parse_bool("no"), Some(false));
        assert_eq!(parse_bool("off"), Some(false));
        assert_eq!(parse_bool(" FALSE"), Some(false));

        assert_eq!(parse_bool("maybe"), None);
        assert_eq!(parse_bool(""), None);
    }

    #[test]
    fn test_env_bool_source() {
        let mut env = HashMap::<String, String>::new();
        env.insert("FOO".to_string(), "true".to_string());
        env.insert("BAR".to_string(), "0".to_string());

        let get_env = |k: &str| env.get(k).cloned();

        assert!(env_bool_source("FOO", false, &get_env));
        assert!(!env_bool_source("BAR", true, &get_env));
        assert!(env_bool_source("BAZ", true, &get_env));
        assert!(!env_bool_source("BAZ", false, &get_env));
    }

    #[test]
    fn test_script_config_resolve() {
        let base = PathBuf::from("/tmp");

        let relative = ScriptConfig {
            program: PathBuf::from("script.sh"),
        };
        assert_eq!(
            relative.resolve_with(&base),
            PathBuf::from("/tmp/script.sh")
        );

        let absolute = ScriptConfig {
            program: PathBuf::from("/usr/bin/python"),
        };
        assert_eq!(
            absolute.resolve_with(&base),
            PathBuf::from("/usr/bin/python")
        );
    }

    #[test]
    fn test_app_config_defaults() {
        let env = HashMap::<String, String>::new();
        let get_env = |k: &str| env.get(k).cloned();
        let get_cwd = || Ok(PathBuf::from("/app"));

        let config = AppConfig::from_source(&get_env, get_cwd).unwrap();

        assert_eq!(config.bind_addr, "127.0.0.1:8080".parse().unwrap());
        assert_eq!(
            config.mopidy_rpc_url.as_str(),
            "http://127.0.0.1:6680/mopidy/rpc"
        );
        assert_eq!(config.script_workdir, PathBuf::from("/app"));
        assert_eq!(config.command_timeout, Duration::from_millis(10_000));
        assert!(config.check_mopidy_health);
    }

    #[test]
    fn test_app_config_overrides() {
        let mut env = HashMap::<String, String>::new();
        env.insert("HAUSKI_BACKEND_BIND".into(), "0.0.0.0:9000".into());
        env.insert(
            "HAUSKI_MOPIDY_RPC_URL".into(),
            "http://mopidy:6680/rpc".into(),
        );
        env.insert("HAUSKI_SCRIPT_WORKDIR".into(), "/opt/scripts".into());
        env.insert("HAUSKI_AUDIO_MODE_CMD".into(), "my-audio-mode".into());
        env.insert("HAUSKI_COMMAND_TIMEOUT_MS".into(), "5000".into());
        env.insert("HAUSKI_CHECK_MOPIDY_HEALTH".into(), "false".into());

        let get_env = |k: &str| env.get(k).cloned();
        let get_cwd = || Ok(PathBuf::from("/app"));

        let config = AppConfig::from_source(&get_env, get_cwd).unwrap();

        assert_eq!(config.bind_addr, "0.0.0.0:9000".parse().unwrap());
        assert_eq!(config.mopidy_rpc_url.as_str(), "http://mopidy:6680/rpc");
        assert_eq!(config.script_workdir, PathBuf::from("/opt/scripts"));
        assert_eq!(
            config.audio_mode_script.program,
            PathBuf::from("my-audio-mode")
        );
        assert_eq!(config.command_timeout, Duration::from_millis(5000));
        assert!(!config.check_mopidy_health);
    }

    #[test]
    fn test_mopidy_url_resolution() {
        // Test MOPIDY_HTTP_URL fallback
        let mut env = HashMap::<String, String>::new();
        env.insert("MOPIDY_HTTP_URL".into(), "http://localhost:6680".into());
        let get_env = |k: &str| env.get(k).cloned();
        let get_cwd = || Ok(PathBuf::from("/app"));

        let config = AppConfig::from_source(&get_env, get_cwd).unwrap();
        assert_eq!(
            config.mopidy_rpc_url.as_str(),
            "http://localhost:6680/mopidy/rpc"
        );
    }

    #[test]
    fn test_invalid_bind_address() {
        let mut env = HashMap::<String, String>::new();
        env.insert("HAUSKI_BIND".into(), "invalid".into());
        let get_env = |k: &str| env.get(k).cloned();
        let get_cwd = || Ok(PathBuf::from("/app"));

        let result = AppConfig::from_source(&get_env, get_cwd);
        assert!(matches!(result, Err(ConfigError::InvalidBindAddress(_))));
    }

    #[test]
    fn test_env_bool_source_invalid_value_falls_back() {
        let mut env = HashMap::<String, String>::new();
        env.insert("HAUSKI_CHECK_MOPIDY_HEALTH".into(), "maybe".into());
        let get_env = |k: &str| env.get(k).cloned();

        // Should fall back to default when value is invalid
        assert!(env_bool_source(
            "HAUSKI_CHECK_MOPIDY_HEALTH",
            true,
            &get_env
        ));
        assert!(!env_bool_source(
            "HAUSKI_CHECK_MOPIDY_HEALTH",
            false,
            &get_env
        ));
    }

    #[test]
    fn test_invalid_mopidy_http_url_returns_error() {
        let mut env = HashMap::<String, String>::new();
        env.insert("MOPIDY_HTTP_URL".into(), "not a url".into());
        let get_env = |k: &str| env.get(k).cloned();
        let get_cwd = || Ok(PathBuf::from("/app"));

        let result = AppConfig::from_source(&get_env, get_cwd);
        assert!(matches!(result, Err(ConfigError::InvalidMopidyUrl(_))));
    }

    #[test]
    fn test_invalid_direct_mopidy_rpc_url_returns_error() {
        let env = RefCell::new(HashMap::<String, String>::new());
        let get_cwd = || Ok(PathBuf::from("/app"));
        let get_env = |k: &str| env.borrow().get(k).cloned();

        // 1. HAUSKI_MOPIDY_RPC_URL invalid
        env.borrow_mut()
            .insert("HAUSKI_MOPIDY_RPC_URL".into(), "://bad".into());
        let result = AppConfig::from_source(&get_env, get_cwd);
        assert!(matches!(result, Err(ConfigError::InvalidMopidyUrl(_))));

        // 2. MOPIDY_RPC_URL invalid (HAUSKI_* not set)
        env.borrow_mut().clear();
        env.borrow_mut()
            .insert("MOPIDY_RPC_URL".into(), "://bad".into());
        let result = AppConfig::from_source(&get_env, get_cwd);
        assert!(matches!(result, Err(ConfigError::InvalidMopidyUrl(_))));
    }

    #[test]
    fn test_mopidy_rpc_url_priority_hauski_overrides_mopidy() {
        let mut env = HashMap::<String, String>::new();
        env.insert(
            "HAUSKI_MOPIDY_RPC_URL".into(),
            "http://hauski-mopidy/rpc".into(),
        );
        env.insert("MOPIDY_RPC_URL".into(), "http://other-mopidy/rpc".into());
        let get_env = |k: &str| env.get(k).cloned();
        let get_cwd = || Ok(PathBuf::from("/app"));

        let config = AppConfig::from_source(&get_env, get_cwd).unwrap();
        assert_eq!(config.mopidy_rpc_url.as_str(), "http://hauski-mopidy/rpc");
    }

    #[test]
    fn test_get_cwd_error_without_workdir_env() {
        let env = HashMap::<String, String>::new();
        let get_env = |k: &str| env.get(k).cloned();
        let get_cwd = || Err(std::io::Error::new(std::io::ErrorKind::Other, "CWD error"));

        let result = AppConfig::from_source(&get_env, get_cwd);
        assert!(matches!(result, Err(ConfigError::WorkingDirectory(_))));
    }

    #[test]
    fn test_app_config_timeout_fallback() {
        let mut env = HashMap::<String, String>::new();
        env.insert("HAUSKI_COMMAND_TIMEOUT_MS".into(), "not-a-number".into());
        let get_env = |k: &str| env.get(k).cloned();
        let get_cwd = || Ok(PathBuf::from("/app"));

        let config = AppConfig::from_source(&get_env, get_cwd).unwrap();
        // Should fall back to DEFAULT_COMMAND_TIMEOUT_MS (10_000)
        assert_eq!(config.command_timeout, Duration::from_millis(10_000));
    }

    #[test]
    fn test_app_config_bind_fallbacks() {
        let env = RefCell::new(HashMap::<String, String>::new());
        let get_cwd = || Ok(PathBuf::from("/app"));
        let get_env = |k: &str| env.borrow().get(k).cloned();

        // HAUSKI_BIND as fallback for HAUSKI_BACKEND_BIND
        env.borrow_mut()
            .insert("HAUSKI_BIND".into(), "127.0.0.1:9999".into());
        let config = AppConfig::from_source(&get_env, get_cwd).unwrap();
        assert_eq!(config.bind_addr, "127.0.0.1:9999".parse().unwrap());

        // HAUSKI_BACKEND_BIND takes precedence
        env.borrow_mut()
            .insert("HAUSKI_BACKEND_BIND".into(), "127.0.0.1:8888".into());
        let config = AppConfig::from_source(&get_env, get_cwd).unwrap();
        assert_eq!(config.bind_addr, "127.0.0.1:8888".parse().unwrap());
    }

    #[test]
    fn test_app_config_playlist_script_fallbacks() {
        let env = RefCell::new(HashMap::<String, String>::new());
        let get_cwd = || Ok(PathBuf::from("/app"));
        let get_env = |k: &str| env.borrow().get(k).cloned();

        // HAUSKI_PLAYLIST_CMD as fallback
        env.borrow_mut()
            .insert("HAUSKI_PLAYLIST_CMD".into(), "old-playlist-cmd".into());
        let config = AppConfig::from_source(&get_env, get_cwd).unwrap();
        assert_eq!(
            config.playlist_script.program,
            PathBuf::from("old-playlist-cmd")
        );

        // HAUSKI_PLAYLIST_FROM_LIST_CMD takes precedence
        env.borrow_mut().insert(
            "HAUSKI_PLAYLIST_FROM_LIST_CMD".into(),
            "new-playlist-cmd".into(),
        );
        let config = AppConfig::from_source(&get_env, get_cwd).unwrap();
        assert_eq!(
            config.playlist_script.program,
            PathBuf::from("new-playlist-cmd")
        );
    }
}
