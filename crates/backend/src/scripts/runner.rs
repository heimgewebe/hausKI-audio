use anyhow::{anyhow, Context, Result};
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::time;
use crate::config::AppConfig;


pub async fn run_script(
    config: &AppConfig,
    program: &str,
    args: &[&str],
    input: Option<&str>,
) -> Result<String> {
    let mut command = Command::new(program);
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
        .with_context(|| format!("failed to spawn {}", program))?;

    if let Some(payload) = input {
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(payload.as_bytes()).await
                .with_context(|| format!("failed to write to stdin for {}", program))?;
        }
    }

    let output = time::timeout(config.command_timeout, child.wait_with_output())
        .await
        .map_err(|_| anyhow!("command {} timed out after {:?}", program, config.command_timeout))
        .and_then(|result| {
            result.with_context(|| format!("command {} error", program))
        })?;

    if !output.status.success() {
        return Err(anyhow!(
            "command {} exited with status {}: stderr={}",
            program,
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let s = String::from_utf8(output.stdout)
        .map_err(|e| anyhow!("script did not output valid UTF-8: {}", e))?;
    Ok(s)
}
