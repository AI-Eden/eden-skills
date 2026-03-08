//! Git source synchronization and repo-level caching.
//!
//! Manages the `.repos/` cache directory under the storage root.  Each
//! unique `(repo_url, ref)` pair maps to a single cache directory keyed by
//! [`repo_cache_key`].  Synchronization is parallelized through the
//! [`SkillReactor`] and supports clone, fetch, checkout, and
//! fast-forward pull stages.

use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::{Config, SkillConfig};
use crate::error::ReactorError;
use crate::paths::{normalize_lexical, resolve_path_string};
use crate::reactor::SkillReactor;

const FETCHED_AT_FILE: &str = ".eden-fetched-at";
const DEFAULT_FRESHNESS_SECS: u64 = 300;

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
    Runtime,
}

impl SyncFailureStage {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Clone => "clone",
            Self::Fetch => "fetch",
            Self::Checkout => "checkout",
            Self::Runtime => "runtime",
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

impl From<ReactorError> for SyncOperationError {
    fn from(value: ReactorError) -> Self {
        Self {
            stage: SyncFailureStage::Runtime,
            detail: value.to_string(),
        }
    }
}

#[derive(Debug)]
struct SyncTask {
    skill_id: String,
    repo_url: String,
    reference: String,
    repo_dir: PathBuf,
    skip: bool,
    force_refresh: bool,
}

#[derive(Debug)]
struct GitOutput {
    stdout: String,
}

/// Normalize a git URL into a flat, lowercase, filesystem-safe string
/// suitable for use as a cache directory name component.
pub fn normalize_repo_url(url: &str) -> String {
    let without_scheme = url.split_once("://").map_or(url, |(_, rest)| rest);
    let scp_normalized = if let Some(rest) = without_scheme.strip_prefix("git@") {
        if let Some((host, path)) = rest.split_once(':') {
            format!("{host}/{path}")
        } else {
            rest.to_string()
        }
    } else {
        without_scheme.to_string()
    };
    let without_git_suffix = scp_normalized
        .strip_suffix(".git")
        .unwrap_or(&scp_normalized);

    without_git_suffix
        .chars()
        .map(|ch| match ch {
            '/' | '\\' | ':' => '_',
            _ => ch.to_ascii_lowercase(),
        })
        .collect()
}

/// Strip non-portable characters from a git ref so it can safely appear
/// in a filesystem path.
pub fn sanitize_ref(reference: &str) -> String {
    reference
        .chars()
        .filter_map(|ch| match ch {
            '/' => Some('_'),
            '.' | '_' | '-' => Some(ch),
            _ if ch.is_ascii_alphanumeric() => Some(ch),
            _ => None,
        })
        .collect()
}

/// Build the canonical cache key for a `(repo_url, ref)` pair:
/// `normalize_repo_url(url)@sanitize_ref(ref)`.
pub fn repo_cache_key(repo_url: &str, reference: &str) -> String {
    format!(
        "{}@{}",
        normalize_repo_url(repo_url),
        sanitize_ref(reference)
    )
}

/// Resolve the absolute path to a repo cache directory under
/// `<storage_root>/.repos/<cache_key>`.
pub fn resolve_repo_cache_root(storage_root: &Path, repo_url: &str, reference: &str) -> PathBuf {
    normalize_lexical(
        &storage_root
            .join(".repos")
            .join(repo_cache_key(repo_url, reference)),
    )
}

/// Resolve the storage root for a single skill — either the repo cache
/// directory (for remote sources) or `<storage_root>/<skill_id>` (for
/// local absolute-path sources).
pub fn resolve_skill_storage_root(storage_root: &Path, skill: &SkillConfig) -> PathBuf {
    if is_local_source_repo(&skill.source.repo) {
        normalize_lexical(&storage_root.join(&skill.id))
    } else {
        resolve_repo_cache_root(storage_root, &skill.source.repo, &skill.source.r#ref)
    }
}

/// Resolve the full source path for a skill by joining its storage root
/// with the configured `subpath`.
pub fn resolve_skill_source_path(storage_root: &Path, skill: &SkillConfig) -> PathBuf {
    let source_root = resolve_skill_storage_root(storage_root, skill);
    normalize_lexical(&source_root.join(&skill.source.subpath))
}

/// Synchronous wrapper around [`sync_sources_async`].  Creates a
/// single-threaded tokio runtime; panics if called from within an
/// existing async context.
pub fn sync_sources(config: &Config, config_dir: &Path) -> Result<SyncSummary, ReactorError> {
    if tokio::runtime::Handle::try_current().is_ok() {
        return Err(ReactorError::RuntimeInitialization {
            detail: "sync_sources called from async context; use sync_sources_async".to_string(),
        });
    }

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|err| ReactorError::RuntimeInitialization {
            detail: err.to_string(),
        })?;
    runtime.block_on(sync_sources_async(config, config_dir))
}

/// Clone or update all remote skill sources in parallel using the
/// default [`SkillReactor`].  Skips repos that were fetched within the
/// last 5 minutes (freshness window).
pub async fn sync_sources_async(
    config: &Config,
    config_dir: &Path,
) -> Result<SyncSummary, ReactorError> {
    let skip_repos = HashSet::new();
    sync_sources_async_inner(
        config,
        config_dir,
        SkillReactor::default(),
        &skip_repos,
        false,
    )
    .await
}

/// Like [`sync_sources_async`] but accepts a custom reactor (for
/// concurrency tuning).  When `force_refresh` is true, the freshness
/// window is bypassed and every repo is fetched unconditionally.
pub async fn sync_sources_async_with_reactor(
    config: &Config,
    config_dir: &Path,
    reactor: SkillReactor,
    force_refresh: bool,
) -> Result<SyncSummary, ReactorError> {
    let skip_repos = HashSet::new();
    sync_sources_async_inner(config, config_dir, reactor, &skip_repos, force_refresh).await
}

/// Full-featured async sync entry point.  Deduplicates by
/// [`repo_cache_key`], skips repos in `skip_repos`, and groups
/// remaining tasks for parallel execution via the reactor.
pub async fn sync_sources_async_with_reactor_skipping_repos(
    config: &Config,
    config_dir: &Path,
    reactor: SkillReactor,
    skip_repos: &HashSet<String>,
) -> Result<SyncSummary, ReactorError> {
    sync_sources_async_inner(config, config_dir, reactor, skip_repos, false).await
}

async fn sync_sources_async_inner(
    config: &Config,
    config_dir: &Path,
    reactor: SkillReactor,
    skip_repos: &HashSet<String>,
    force_refresh: bool,
) -> Result<SyncSummary, ReactorError> {
    let storage_root = resolve_path_string(&config.storage_root, config_dir).map_err(|err| {
        ReactorError::Config {
            detail: err.to_string(),
        }
    })?;
    tokio::fs::create_dir_all(&storage_root).await?;

    let mut grouped_tasks = BTreeMap::new();
    for skill in &config.skills {
        if is_local_source_repo(&skill.source.repo) {
            continue;
        }
        let cache_key = repo_cache_key(&skill.source.repo, &skill.source.r#ref);
        grouped_tasks.entry(cache_key).or_insert_with(|| SyncTask {
            skill_id: skill.id.clone(),
            repo_url: skill.source.repo.clone(),
            reference: skill.source.r#ref.clone(),
            repo_dir: resolve_repo_cache_root(
                &storage_root,
                &skill.source.repo,
                &skill.source.r#ref,
            ),
            skip: skip_repos.contains(&repo_cache_key(&skill.source.repo, &skill.source.r#ref)),
            force_refresh,
        });
    }

    if !grouped_tasks.is_empty() {
        tokio::fs::create_dir_all(storage_root.join(".repos")).await?;
    }

    let tasks = grouped_tasks.into_values().collect::<Vec<_>>();

    let outcomes = reactor
        .run_phase_a(tasks, move |task| {
            let reactor = reactor;
            async move { sync_one_source(task, reactor).await }
        })
        .await?;

    let mut summary = SyncSummary::default();
    for outcome in outcomes {
        match outcome.result {
            Ok(SyncOutcome::Cloned) => summary.cloned += 1,
            Ok(SyncOutcome::Updated) => summary.updated += 1,
            Ok(SyncOutcome::Skipped) => summary.skipped += 1,
            Err(failure) => {
                summary.failed += 1;
                summary.failures.push(failure);
            }
        }
    }

    Ok(summary)
}

async fn sync_one_source(
    task: SyncTask,
    reactor: SkillReactor,
) -> Result<SyncOutcome, SyncFailure> {
    if task.skip {
        return Ok(SyncOutcome::Skipped);
    }

    let repo_exists = task.repo_dir.join(".git").exists();

    if repo_exists && !task.force_refresh && repo_is_fresh(&task.repo_dir) {
        return Ok(SyncOutcome::Skipped);
    }

    let repo_dir_display = task.repo_dir.display().to_string();
    let skill_id = task.skill_id.clone();
    let task_name = format!("sync source `{}`", task.skill_id);

    let sync_result = if repo_exists {
        let repo_dir = task.repo_dir.clone();
        let reference = task.reference.clone();
        reactor
            .run_blocking(&task_name, move || update_repo(&repo_dir, &reference))
            .await
    } else {
        let repo_url = task.repo_url.clone();
        let reference = task.reference.clone();
        let repo_dir = task.repo_dir.clone();
        reactor
            .run_blocking(&task_name, move || {
                clone_repo(&repo_url, &reference, &repo_dir)
            })
            .await
    };

    match sync_result {
        Ok(outcome) => {
            write_fetched_at(&task.repo_dir);
            Ok(outcome)
        }
        Err(err) => Err(SyncFailure {
            skill_id,
            stage: err.stage,
            repo_dir: repo_dir_display,
            detail: err.detail,
        }),
    }
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

    record_test_git_clone_if_configured();
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

    record_test_git_clone_if_configured();
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

    record_test_git_fetch_if_configured();
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

fn is_local_source_repo(repo_url: &str) -> bool {
    Path::new(repo_url).is_absolute()
}

fn repo_is_fresh(repo_dir: &Path) -> bool {
    let fetched_at_path = repo_dir.join(FETCHED_AT_FILE);
    let Ok(content) = std::fs::read_to_string(&fetched_at_path) else {
        return false;
    };
    let Ok(fetched_at) = content.trim().parse::<u64>() else {
        return false;
    };
    let Ok(now) = SystemTime::now().duration_since(UNIX_EPOCH) else {
        return false;
    };
    now.as_secs().saturating_sub(fetched_at) < DEFAULT_FRESHNESS_SECS
}

fn write_fetched_at(repo_dir: &Path) {
    let Ok(now) = SystemTime::now().duration_since(UNIX_EPOCH) else {
        return;
    };
    let _ = std::fs::write(repo_dir.join(FETCHED_AT_FILE), now.as_secs().to_string());
}

fn record_test_git_clone_if_configured() {
    record_test_git_event_if_configured("EDEN_SKILLS_TEST_GIT_CLONE_LOG", b"clone\n");
}

fn record_test_git_fetch_if_configured() {
    record_test_git_event_if_configured("EDEN_SKILLS_TEST_GIT_FETCH_LOG", b"fetch\n");
}

fn record_test_git_event_if_configured(env_var: &str, line: &[u8]) {
    let Some(log_path) = std::env::var_os(env_var) else {
        return;
    };
    let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
    else {
        return;
    };
    let _ = std::io::Write::write_all(&mut file, line);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};
    use tempfile::tempdir;

    #[test]
    fn repo_is_fresh_returns_false_when_no_timestamp_file() {
        let temp = tempdir().expect("tempdir");
        assert!(!repo_is_fresh(temp.path()));
    }

    #[test]
    fn repo_is_fresh_returns_true_within_window() {
        let temp = tempdir().expect("tempdir");
        write_fetched_at(temp.path());
        assert!(repo_is_fresh(temp.path()));
    }

    #[test]
    fn repo_is_fresh_returns_false_after_window() {
        let temp = tempdir().expect("tempdir");
        let stale = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - DEFAULT_FRESHNESS_SECS
            - 10;
        std::fs::write(temp.path().join(FETCHED_AT_FILE), stale.to_string()).unwrap();
        assert!(!repo_is_fresh(temp.path()));
    }

    #[test]
    fn repo_is_fresh_returns_false_for_corrupted_file() {
        let temp = tempdir().expect("tempdir");
        std::fs::write(temp.path().join(FETCHED_AT_FILE), "not-a-number").unwrap();
        assert!(!repo_is_fresh(temp.path()));
    }

    #[test]
    fn write_fetched_at_creates_parseable_timestamp() {
        let temp = tempdir().expect("tempdir");
        write_fetched_at(temp.path());
        let content = std::fs::read_to_string(temp.path().join(FETCHED_AT_FILE)).unwrap();
        let ts: u64 = content
            .trim()
            .parse()
            .expect("should be a valid u64 timestamp");
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        assert!(now - ts < 5, "timestamp should be within 5 seconds of now");
    }
}
