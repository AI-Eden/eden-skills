use std::path::Path;
use std::process::Command;

use crate::config::Config;
use crate::error::EdenError;
use crate::paths::{normalize_lexical, resolve_path_string};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SyncSummary {
    pub cloned: usize,
    pub updated: usize,
    pub skipped: usize,
    pub failed: usize,
    pub failures: Vec<SyncFailure>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncFailure {
    pub skill_id: String,
    pub stage: SyncFailureStage,
    pub repo_dir: String,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncFailureStage {
    Clone,
    Fetch,
    Checkout,
}

impl SyncFailureStage {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Clone => "clone",
            Self::Fetch => "fetch",
            Self::Checkout => "checkout",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SyncOutcome {
    Cloned,
    Updated,
    Skipped,
}

#[derive(Debug, Clone)]
struct SyncOperationError {
    stage: SyncFailureStage,
    detail: String,
}

#[derive(Debug)]
struct GitOutput {
    stdout: String,
}

pub fn sync_sources(config: &Config, config_dir: &Path) -> Result<SyncSummary, EdenError> {
    let storage_root = resolve_path_string(&config.storage_root, config_dir)?;
    std::fs::create_dir_all(&storage_root)?;

    let mut summary = SyncSummary::default();
    for skill in &config.skills {
        let repo_dir = normalize_lexical(&storage_root.join(&skill.id));
        let result = if repo_dir.join(".git").exists() {
            update_repo(&repo_dir, &skill.source.r#ref)
        } else {
            clone_repo(&skill.source.repo, &skill.source.r#ref, &repo_dir)
        };

        match result {
            Ok(SyncOutcome::Cloned) => summary.cloned += 1,
            Ok(SyncOutcome::Updated) => summary.updated += 1,
            Ok(SyncOutcome::Skipped) => summary.skipped += 1,
            Err(err) => {
                summary.failed += 1;
                summary.failures.push(SyncFailure {
                    skill_id: skill.id.clone(),
                    stage: err.stage,
                    repo_dir: repo_dir.display().to_string(),
                    detail: err.detail,
                });
            }
        }
    }

    Ok(summary)
}

fn clone_repo(
    repo_url: &str,
    reference: &str,
    repo_dir: &Path,
) -> Result<SyncOutcome, SyncOperationError> {
    if let Some(parent) = repo_dir.parent() {
        std::fs::create_dir_all(parent).map_err(|err| SyncOperationError {
            stage: SyncFailureStage::Clone,
            detail: format!(
                "failed to create repository parent directory `{}`: {err}",
                parent.display()
            ),
        })?;
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
        &format!(
            "clone `{repo_url}` into `{}` with ref `{reference}`",
            repo_dir.display()
        ),
    );
    let branch_error = match branch_clone {
        Ok(_) => return Ok(SyncOutcome::Cloned),
        Err(err) => err,
    };

    let fallback_clone = run_git(
        Command::new("git").arg("clone").arg(repo_url).arg(repo_dir),
        &format!(
            "clone `{repo_url}` into `{}` without branch hint",
            repo_dir.display()
        ),
    );
    if let Err(fallback_error) = fallback_clone {
        return Err(SyncOperationError {
            stage: SyncFailureStage::Clone,
            detail: format!(
                "branch clone attempt failed: {branch_error}; fallback clone attempt failed: {fallback_error}"
            ),
        });
    }

    checkout_repo_ref(repo_dir, reference)?;
    Ok(SyncOutcome::Cloned)
}

fn update_repo(repo_dir: &Path, reference: &str) -> Result<SyncOutcome, SyncOperationError> {
    let head_before = read_head_sha(repo_dir);

    run_git(
        Command::new("git")
            .arg("-C")
            .arg(repo_dir)
            .arg("fetch")
            .arg("--all")
            .arg("--prune"),
        &format!("fetch updates for `{}`", repo_dir.display()),
    )
    .map_err(|detail| SyncOperationError {
        stage: SyncFailureStage::Fetch,
        detail,
    })?;

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
        &format!("fast-forward pull for `{}`", repo_dir.display()),
    );
    let head_after = read_head_sha(repo_dir);
    if matches!(
        (&head_before, &head_after),
        (Some(before), Some(after)) if before == after
    ) {
        return Ok(SyncOutcome::Skipped);
    }

    Ok(SyncOutcome::Updated)
}

fn checkout_repo_ref(repo_dir: &Path, reference: &str) -> Result<(), SyncOperationError> {
    run_git(
        Command::new("git")
            .arg("-C")
            .arg(repo_dir)
            .arg("checkout")
            .arg(reference),
        &format!("checkout ref `{reference}` in `{}`", repo_dir.display()),
    )
    .map(|_| ())
    .map_err(|detail| SyncOperationError {
        stage: SyncFailureStage::Checkout,
        detail,
    })
}

fn read_head_sha(repo_dir: &Path) -> Option<String> {
    let output = run_git(
        Command::new("git")
            .arg("-C")
            .arg(repo_dir)
            .arg("rev-parse")
            .arg("HEAD"),
        &format!("read HEAD for `{}`", repo_dir.display()),
    )
    .ok()?;

    let head = output.stdout.lines().next()?.trim();
    if head.is_empty() {
        return None;
    }
    Some(head.to_string())
}

fn run_git(command: &mut Command, context: &str) -> Result<GitOutput, String> {
    let output = command
        .output()
        .map_err(|err| format!("git invocation failed while trying to {context}: {err}"))?;

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if output.status.success() {
        return Ok(GitOutput { stdout });
    }

    Err(format!(
        "git command failed while trying to {context}: status={} stderr=`{}` stdout=`{}`",
        output.status, stderr, stdout
    ))
}
