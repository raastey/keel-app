use crate::{
    domain::{DeveloperGate, DeveloperProposal},
    error::{KeelError, Result},
};
use std::{
    collections::HashSet,
    path::{Component, Path},
};
use tokio::process::Command;

fn safe_relative(path: &str) -> bool {
    let path = Path::new(path);
    !path.is_absolute()
        && path
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
}

pub fn validate(proposal: &DeveloperProposal) -> Result<()> {
    let root = Path::new(&proposal.repository_root).canonicalize()?;
    if !root.is_dir() {
        return Err(KeelError::Validation(
            "Repository root must be a directory".into(),
        ));
    }
    if proposal.scope.trim().is_empty()
        || proposal.files.is_empty()
        || proposal.diff.trim().is_empty()
    {
        return Err(KeelError::Validation(
            "Scope, file list, and diff are required".into(),
        ));
    }
    if proposal.files.iter().any(|path| !safe_relative(path)) {
        return Err(KeelError::Policy(
            "Every proposed file must be a safe path inside the repository".into(),
        ));
    }
    let declared: HashSet<_> = proposal.files.iter().map(String::as_str).collect();
    let changed: Vec<_> = proposal
        .diff
        .lines()
        .filter_map(|line| {
            line.strip_prefix("+++ b/")
                .or_else(|| line.strip_prefix("--- a/"))
        })
        .filter(|path| *path != "/dev/null")
        .collect();
    if changed.is_empty() {
        return Err(KeelError::Validation(
            "The proposal is not a unified diff".into(),
        ));
    }
    if changed
        .iter()
        .any(|path| !safe_relative(path) || !declared.contains(path))
    {
        return Err(KeelError::Policy(
            "The diff changes a file outside the reviewed file list".into(),
        ));
    }
    Ok(())
}

pub async fn check_and_apply(proposal: &DeveloperProposal) -> Result<String> {
    validate(proposal)?;
    if proposal.gate != DeveloperGate::ApplyChoice || !proposal.apply_authorized {
        return Err(KeelError::Policy(
            "Review the diff and explicitly authorize Apply before writing".into(),
        ));
    }
    let root = Path::new(&proposal.repository_root).canonicalize()?;
    let mut check = Command::new("git")
        .arg("apply")
        .arg("--check")
        .arg("--whitespace=error-all")
        .arg("-")
        .current_dir(&root)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| KeelError::Engine(e.to_string()))?;
    use tokio::io::AsyncWriteExt;
    check
        .stdin
        .take()
        .ok_or_else(|| KeelError::Engine("Could not open patch input".into()))?
        .write_all(proposal.diff.as_bytes())
        .await
        .map_err(|e| KeelError::Engine(e.to_string()))?;
    let checked = check
        .wait_with_output()
        .await
        .map_err(|e| KeelError::Engine(e.to_string()))?;
    if !checked.status.success() {
        return Err(KeelError::Policy(format!(
            "Patch check failed: {}",
            String::from_utf8_lossy(&checked.stderr).trim()
        )));
    }
    let mut apply = Command::new("git")
        .arg("apply")
        .arg("--whitespace=error-all")
        .arg("-")
        .current_dir(&root)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| KeelError::Engine(e.to_string()))?;
    apply
        .stdin
        .take()
        .ok_or_else(|| KeelError::Engine("Could not open patch input".into()))?
        .write_all(proposal.diff.as_bytes())
        .await
        .map_err(|e| KeelError::Engine(e.to_string()))?;
    let output = apply
        .wait_with_output()
        .await
        .map_err(|e| KeelError::Engine(e.to_string()))?;
    if !output.status.success() {
        return Err(KeelError::Engine(format!(
            "Patch could not be applied: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(format!(
        "Applied reviewed diff to {} file(s)",
        proposal.files.len()
    ))
}

pub async fn run_test(proposal: &DeveloperProposal) -> Result<String> {
    if proposal.gate != DeveloperGate::TestCommand || !proposal.applied || !proposal.test_authorized
    {
        return Err(KeelError::Policy(
            "Applying a diff does not authorize a test command".into(),
        ));
    }
    let parts = proposal
        .test_command
        .as_ref()
        .ok_or_else(|| KeelError::Validation("No test command was selected".into()))?;
    let (program, args) = parts
        .split_first()
        .ok_or_else(|| KeelError::Validation("Test command is empty".into()))?;
    let allowed = matches!(
        (program.as_str(), args.first().map(String::as_str)),
        ("cargo", Some("test" | "check"))
            | ("swift", Some("test"))
            | ("npm", Some("test"))
            | ("pnpm", Some("test"))
            | ("yarn", Some("test"))
            | ("pytest", _)
    );
    if !allowed {
        return Err(KeelError::Policy("Allowed test commands are cargo test/check, swift test, npm/pnpm/yarn test, and pytest".into()));
    }
    if parts
        .iter()
        .any(|part| part.contains('\0') || matches!(part.as_str(), "--exec" | "-c"))
    {
        return Err(KeelError::Policy(
            "The test command contains a blocked argument".into(),
        ));
    }
    let root = Path::new(&proposal.repository_root).canonicalize()?;
    let output = Command::new(program)
        .args(args)
        .current_dir(root)
        .output()
        .await
        .map_err(|e| KeelError::Engine(e.to_string()))?;
    let mut text = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    if text.len() > 100_000 {
        text.truncate(100_000);
        text.push_str("\n[output truncated]");
    }
    Ok(format!(
        "Exit {}\n{}",
        output.status.code().unwrap_or(-1),
        text.trim()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;
    #[test]
    fn traversal_is_rejected() {
        let p = DeveloperProposal {
            id: Uuid::new_v4(),
            project_id: Uuid::new_v4(),
            repository_root: "/tmp".into(),
            scope: "x".into(),
            files: vec!["../secret".into()],
            diff: "--- a/../secret\n+++ b/../secret\n@@ -1 +1 @@\n-a\n+b".into(),
            gate: DeveloperGate::Diff,
            apply_authorized: false,
            applied: false,
            test_command: None,
            test_authorized: false,
            test_output: None,
            result_summary: None,
            created_at: Utc::now(),
        };
        assert!(validate(&p).is_err());
    }
}
