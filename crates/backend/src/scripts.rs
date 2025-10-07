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
