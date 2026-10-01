use crate::error::{KeelError, Result};
use std::path::Path;
use tokio::process::Command;

pub async fn doctor(cli_path: &str, root: &str) -> Result<String> {
    let cli = Path::new(cli_path).canonicalize()?;
    let root = Path::new(root).canonicalize()?;
    let output = Command::new(cli)
        .arg("doctor")
        .arg("--root")
        .arg(root)
        .arg("--json")
        .output()
        .await
        .map_err(|e| KeelError::Engine(e.to_string()))?;
    if !output.status.success() {
        return Err(KeelError::Engine(
            String::from_utf8_lossy(&output.stderr).trim().into(),
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into())
}

pub async fn train(cli_path: &str, root: &str, profile: &str, run_id: &str) -> Result<String> {
    if !matches!(profile, "smoke" | "chunked" | "context" | "voice") {
        return Err(KeelError::Validation("Unknown training profile".into()));
    }
    if run_id.is_empty()
        || !run_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(KeelError::Validation(
            "Run ID may contain letters, numbers, hyphens, and underscores".into(),
        ));
    }
    let cli = Path::new(cli_path).canonicalize()?;
    let root = Path::new(root).canonicalize()?;
    let output = Command::new(cli)
        .arg("train")
        .arg(profile)
        .arg("writer")
        .arg("--run-id")
        .arg(run_id)
        .arg("--root")
        .arg(&root)
        .output()
        .await
        .map_err(|e| KeelError::Engine(e.to_string()))?;
    if !output.status.success() {
        return Err(KeelError::Engine(format!(
            "Training stopped: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    let adapter = root
        .join("adapters")
        .join("writer")
        .join(run_id)
        .join("adapters.safetensors");
    if !adapter.is_file() || adapter.metadata()?.len() == 0 {
        return Err(KeelError::Policy(
            "Training exited without a non-empty adapter. Nothing was promoted.".into(),
        ));
    }
    Ok(adapter.parent().unwrap_or(&adapter).display().to_string())
}
