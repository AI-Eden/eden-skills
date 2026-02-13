use std::path::Path;
use std::process::Command;

use crate::config::Config;
use crate::error::EdenError;
use crate::paths::{normalize_lexical, resolve_path_string};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SyncSummary {
    pub cloned: usize,
    pub updated: usize,
    pub skipped: usize,
}

pub fn sync_sources(config: &Config, config_dir: &Path) -> Result<SyncSummary, EdenError> {
    let storage_root = resolve_path_string(&config.storage_root, config_dir)?;
    std::fs::create_dir_all(&storage_root)?;

    let mut summary = SyncSummary::default();
    for skill in &config.skills {
        let repo_dir = normalize_lexical(&storage_root.join(&skill.id));
        if repo_dir.join(".git").exists() {
            update_repo(&repo_dir, &skill.source.r#ref)?;
            summary.updated += 1;
            continue;
        }

        clone_repo(&skill.source.repo, &skill.source.r#ref, &repo_dir)?;
        summary.cloned += 1;
    }

    Ok(summary)
}

fn clone_repo(repo_url: &str, reference: &str, repo_dir: &Path) -> Result<(), EdenError> {
    if let Some(parent) = repo_dir.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let branch_clone = run_git(
        Command::new("git")
            .arg("clone")
            .arg("--depth")
            .arg("1")
            .arg("--branch")
            .arg(reference)
            .arg(repo_url)
            .arg(repo_dir),
        format!(
            "clone `{repo_url}` into `{}` with ref `{reference}`",
            repo_dir.display()
        ),
    );
    if branch_clone.is_ok() {
        return Ok(());
    }

    run_git(
        Command::new("git").arg("clone").arg(repo_url).arg(repo_dir),
        format!(
            "clone `{repo_url}` into `{}` without branch hint",
            repo_dir.display()
        ),
    )?;
    checkout_repo_ref(repo_dir, reference)
}

fn update_repo(repo_dir: &Path, reference: &str) -> Result<(), EdenError> {
    run_git(
        Command::new("git")
            .arg("-C")
            .arg(repo_dir)
            .arg("fetch")
            .arg("--all")
            .arg("--prune"),
        format!("fetch updates for `{}`", repo_dir.display()),
    )?;

    checkout_repo_ref(repo_dir, reference)?;

    // Pull is best-effort for branch refs; if ref is detached/commit/tag this may fail and is ignored.
    let _ = run_git(
        Command::new("git")
            .arg("-C")
            .arg(repo_dir)
            .arg("pull")
            .arg("--ff-only")
            .arg("origin")
            .arg(reference),
        format!("fast-forward pull for `{}`", repo_dir.display()),
    );
    Ok(())
}

fn checkout_repo_ref(repo_dir: &Path, reference: &str) -> Result<(), EdenError> {
    run_git(
        Command::new("git")
            .arg("-C")
            .arg(repo_dir)
            .arg("checkout")
            .arg(reference),
        format!("checkout ref `{reference}` in `{}`", repo_dir.display()),
    )
}

fn run_git(command: &mut Command, context: String) -> Result<(), EdenError> {
    let output = command.output().map_err(|err| {
        EdenError::Runtime(format!(
            "git invocation failed while trying to {context}: {err}"
        ))
    })?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    Err(EdenError::Runtime(format!(
        "git command failed while trying to {context}: status={} stderr=`{}` stdout=`{}`",
        output.status,
        stderr.trim(),
        stdout.trim()
    )))
}
