//! Shared utilities for command implementations.
//!
//! Provides config I/O helpers, path resolution, git/docker preflight
//! checks, plan execution, lock file writes, output formatting helpers,
//! and registry-mode skill resolution used across multiple commands.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::ui::{abbreviate_home_path, UiContext};
use eden_skills_core::config::{
    decode_registry_mode_repo, is_registry_mode_repo, load_from_file, LoadOptions, LoadedConfig,
    SourceConfig,
};
use eden_skills_core::config::{AgentKind, Config, InstallMode, SkillConfig, TargetConfig};
use eden_skills_core::error::EdenError;
use eden_skills_core::lock::{build_lock_from_config, lock_path_for_config, write_lock_file};
use eden_skills_core::paths::resolve_path_string;
use eden_skills_core::plan::PlanItem;
use eden_skills_core::reactor::{MAX_CONCURRENCY_LIMIT, MIN_CONCURRENCY_LIMIT};
use eden_skills_core::registry::{
    parse_registry_specs_from_toml, resolve_skill_from_registry_sources,
    sort_registry_specs_by_priority, RegistrySource,
};
use eden_skills_core::safety::{LicenseStatus, SkillSafetyReport};
use eden_skills_core::source::SyncSummary;
use owo_colors::OwoColorize;

pub(crate) const REGISTRY_SYNC_MARKER_FILE: &str = ".eden-last-sync";

/// Resolve a possibly-relative or tilde-prefixed config path against `cwd`.
///
/// # Errors
///
/// Returns [`EdenError::Io`] if the current directory cannot be determined.
pub(crate) fn resolve_config_path(config_path: &str) -> Result<PathBuf, EdenError> {
    let cwd = std::env::current_dir().map_err(EdenError::Io)?;
    resolve_path_string(config_path, &cwd)
}

pub(crate) fn with_hint(message: impl Into<String>, hint: impl Into<String>) -> String {
    format!("{}\nhint: {}", message.into(), hint.into())
}

/// Load and parse `skills.toml`, wrapping common I/O errors with
/// user-friendly messages and hints (e.g. "Run `eden-skills init`").
///
/// In strict mode, config warnings are promoted to errors.
///
/// # Errors
///
/// Returns [`EdenError::Runtime`] with an abbreviated path when the
/// file is missing or unreadable, or propagates parse/validation errors.
pub(crate) fn load_config_with_context(
    config_path: &Path,
    strict: bool,
) -> Result<LoadedConfig, EdenError> {
    match load_from_file(config_path, LoadOptions { strict }) {
        Ok(loaded) => Ok(loaded),
        Err(EdenError::Io(err)) if err.kind() == std::io::ErrorKind::NotFound => {
            let display_path = abbreviate_home_path(&config_path.display().to_string());
            Err(EdenError::Runtime(with_hint(
                format!("config file not found: {display_path}"),
                "Run `eden-skills init` to create a new config.",
            )))
        }
        Err(EdenError::Io(err)) if err.kind() == std::io::ErrorKind::PermissionDenied => {
            let display_path = abbreviate_home_path(&config_path.display().to_string());
            Err(EdenError::Runtime(with_hint(
                format!("permission denied reading config file: {display_path}"),
                "Check file permissions or run with appropriate privileges.",
            )))
        }
        Err(err) => Err(err),
    }
}

pub(crate) fn git_bin() -> String {
    std::env::var("EDEN_SKILLS_GIT_BIN")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "git".to_string())
}

/// Verify that `git` is available on `$PATH` before operations that require it.
///
/// # Errors
///
/// Returns [`EdenError::Runtime`] with an install hint when git is absent.
pub(crate) fn ensure_git_available() -> Result<(), EdenError> {
    let git = git_bin();
    match Command::new(&git).arg("--version").output() {
        Ok(output) if output.status.success() => Ok(()),
        Ok(output) => Err(EdenError::Runtime(with_hint(
            format!(
                "git executable not found (command `{git}` exited with status {})",
                output.status
            ),
            "Install Git: https://git-scm.com/downloads",
        ))),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            Err(EdenError::Runtime(with_hint(
                "git executable not found",
                "Install Git: https://git-scm.com/downloads",
            )))
        }
        Err(err) => Err(EdenError::Runtime(with_hint(
            format!("failed to invoke git executable `{git}`: {err}"),
            "Ensure Git is installed and available on PATH.",
        ))),
    }
}

/// Verify that `docker` is on `$PATH` when any target uses a Docker environment.
///
/// Skips the check entirely when all targets are local.
///
/// # Errors
///
/// Returns [`EdenError::Runtime`] with an install hint when docker is
/// required but absent.
pub(crate) fn ensure_docker_available_for_targets<'a, I>(targets: I) -> Result<(), EdenError>
where
    I: IntoIterator<Item = &'a str>,
{
    if !targets.into_iter().any(|environment| {
        environment
            .strip_prefix("docker:")
            .is_some_and(|container| !container.is_empty())
    }) {
        return Ok(());
    }

    let docker_bin = doctor_docker_bin();
    match Command::new(&docker_bin).arg("--version").output() {
        Ok(output) if output.status.success() => Ok(()),
        Ok(output) => Err(EdenError::Runtime(with_hint(
            format!(
                "docker executable not found (command `{docker_bin}` exited with status {})",
                output.status
            ),
            "Install Docker: https://docs.docker.com/get-docker/",
        ))),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            Err(EdenError::Runtime(with_hint(
                "docker executable not found",
                "Install Docker: https://docs.docker.com/get-docker/",
            )))
        }
        Err(err) => Err(EdenError::Runtime(with_hint(
            format!("failed to invoke docker executable `{docker_bin}`: {err}"),
            "Install Docker or ensure `docker` is available on PATH.",
        ))),
    }
}

pub(crate) fn doctor_docker_bin() -> String {
    std::env::var("EDEN_SKILLS_DOCKER_BIN")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "docker".to_string())
}

pub(crate) fn resolve_effective_reactor_concurrency(
    cli_override: Option<usize>,
    config_concurrency: usize,
    field_path: &str,
) -> Result<usize, EdenError> {
    let concurrency = cli_override.unwrap_or(config_concurrency);
    if !(MIN_CONCURRENCY_LIMIT..=MAX_CONCURRENCY_LIMIT).contains(&concurrency) {
        return Err(EdenError::Validation(format!(
            "INVALID_CONCURRENCY: {field_path}: expected value in [{MIN_CONCURRENCY_LIMIT}, {MAX_CONCURRENCY_LIMIT}], got {concurrency}"
        )));
    }
    Ok(concurrency)
}

pub(crate) fn block_on_command_future<F>(future: F) -> Result<(), EdenError>
where
    F: Future<Output = Result<(), EdenError>>,
{
    if tokio::runtime::Handle::try_current().is_ok() {
        return Err(EdenError::Runtime(
            "sync command API called inside async runtime; use async command entrypoints"
                .to_string(),
        ));
    }

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|err| EdenError::Runtime(format!("failed to initialize tokio runtime: {err}")))?;
    runtime.block_on(future)
}

pub(crate) fn run_git_command(command: &mut Command, context: &str) -> Result<String, String> {
    let output = command
        .output()
        .map_err(|err| format!("git invocation failed while trying to {context}: {err}"))?;
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if output.status.success() {
        return Ok(stdout);
    }
    Err(format!(
        "git command failed while trying to {context}: status={} stderr=`{}` stdout=`{}`",
        output.status, stderr, stdout
    ))
}

pub(crate) fn extract_git_clone_failure_reason(stderr: &str) -> &str {
    let lower = stderr.to_ascii_lowercase();
    if lower.contains("repository not found") {
        return "repository not found";
    }
    if lower.contains("could not resolve host") {
        return "could not resolve host";
    }
    if lower.contains("authentication failed") {
        return "authentication failed";
    }
    if lower.contains("permission denied") {
        return "permission denied (publickey)";
    }
    if lower.contains("could not find remote branch") || lower.contains("not found in upstream") {
        return "remote branch not found";
    }
    if lower.contains("connection refused") {
        return "connection refused";
    }
    if lower.contains("connection timed out") || lower.contains("timed out") {
        return "connection timed out";
    }
    if lower.contains("ssl certificate problem") {
        return "SSL certificate error";
    }
    if lower.contains("unable to access") {
        return "unable to access remote";
    }
    "git clone failed"
}

pub(crate) fn git_clone_failure_hint(reason: &str, repo_url: &str) -> String {
    match reason {
        "repository not found" => {
            format!("Check the URL spelling and ensure `{repo_url}` exists and is accessible.")
        }
        "could not resolve host" | "connection refused" | "connection timed out" => {
            "Check your internet connection and DNS settings.".to_string()
        }
        "authentication failed" | "permission denied (publickey)" => {
            format!(
                "Ensure you have access to `{repo_url}` and your git credentials are configured."
            )
        }
        "remote branch not found" => {
            "Check that the branch, tag, or commit exists in the repository.".to_string()
        }
        "SSL certificate error" => "Check your system's SSL/TLS certificates.".to_string(),
        _ => format!("Run `git clone {repo_url}` manually to diagnose the issue."),
    }
}

pub(crate) fn read_head_sha(repo_dir: &Path) -> Option<String> {
    let stdout = run_git_command(
        Command::new("git")
            .arg("-C")
            .arg(repo_dir)
            .arg("rev-parse")
            .arg("HEAD"),
        &format!("read HEAD for `{}`", repo_dir.display()),
    )
    .ok()?;
    let sha = stdout.lines().next()?.trim();
    if sha.is_empty() {
        None
    } else {
        Some(sha.to_string())
    }
}

pub(crate) fn agent_kind_label(agent: &AgentKind) -> &'static str {
    agent.as_str()
}

pub(crate) fn parse_target_specs(specs: &[String]) -> Result<Vec<TargetConfig>, EdenError> {
    let mut targets = Vec::with_capacity(specs.len());
    for spec in specs {
        if let Some(agent) = AgentKind::from_target_spec(spec) {
            targets.push(TargetConfig {
                agent,
                expected_path: None,
                path: None,
                environment: "local".to_string(),
            });
            continue;
        }

        if let Some(rest) = spec.strip_prefix("custom:") {
            if rest.trim().is_empty() {
                return Err(EdenError::InvalidArguments(
                    "invalid target spec `custom:`: path is required".to_string(),
                ));
            }
            targets.push(TargetConfig {
                agent: AgentKind::Custom,
                expected_path: None,
                path: Some(rest.to_string()),
                environment: "local".to_string(),
            });
            continue;
        }

        return Err(EdenError::InvalidArguments(format!(
            "invalid target spec `{spec}` (expected a supported agent id or `custom:<path>`)"
        )));
    }
    Ok(targets)
}

pub(crate) fn format_quoted_ids(ids: &[String]) -> String {
    ids.iter()
        .map(|id| format!("'{id}'"))
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn unique_ids(ids: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut unique = Vec::new();
    for id in ids {
        if seen.insert(id.as_str()) {
            unique.push(id.clone());
        }
    }
    unique
}

pub(crate) fn print_source_sync_step_summary_human(ui: &UiContext, synced: usize, failed: usize) {
    let repo_word = if synced == 1 { "repo" } else { "repos" };
    println!(
        "{}  {} {repo_word} synced, {} failed",
        ui.action_prefix("Syncing"),
        style_count(ui, synced, CountStyle::GreenIfNonZero),
        style_count(ui, failed, CountStyle::RedIfNonZero),
    );
}

pub(crate) fn print_source_sync_summary_human(ui: &UiContext, summary: &SyncSummary) {
    println!(
        "{}  {} cloned, {} updated, {} skipped, {} failed",
        ui.action_prefix("Syncing"),
        style_count(ui, summary.cloned, CountStyle::GreenIfNonZero),
        style_count(ui, summary.updated, CountStyle::GreenIfNonZero),
        style_count(ui, summary.skipped, CountStyle::DimAlways),
        style_count(ui, summary.failed, CountStyle::RedIfNonZero),
    );
}

pub(crate) fn source_sync_failure_error(summary: &SyncSummary) -> Option<EdenError> {
    if summary.failed == 0 {
        return None;
    }

    let details = summary
        .failures
        .iter()
        .map(|failure| {
            let reason = extract_git_clone_failure_reason(&failure.detail);
            format!(
                "'{}' ({} — {reason})",
                failure.skill_id,
                failure.stage.as_str()
            )
        })
        .collect::<Vec<_>>()
        .join(", ");

    let hint = summary
        .failures
        .first()
        .map(|f| {
            let reason = extract_git_clone_failure_reason(&f.detail);
            git_clone_failure_hint(reason, &f.repo_dir)
        })
        .unwrap_or_default();

    Some(EdenError::Runtime(format!(
        "source sync failed for {}: {details}\nhint: {hint}",
        if summary.failed == 1 {
            "1 repo".to_string()
        } else {
            format!("{} repos", summary.failed)
        }
    )))
}

pub(crate) fn print_safety_summary_human(ui: &UiContext, reports: &[SkillSafetyReport]) {
    let permissive = reports
        .iter()
        .filter(|r| matches!(r.license_status, LicenseStatus::Permissive))
        .count();
    let non_permissive = reports
        .iter()
        .filter(|r| matches!(r.license_status, LicenseStatus::NonPermissive))
        .count();
    let risk_labeled = reports.iter().filter(|r| !r.risk_labels.is_empty()).count();
    let no_exec = reports.iter().filter(|r| r.no_exec_metadata_only).count();
    let risk_flags = non_permissive + risk_labeled;

    println!(
        "{}  {} permissive, {} risk flags, {} no-exec",
        ui.action_prefix("Safety"),
        style_count(ui, permissive, CountStyle::GreenIfNonZero),
        style_count(ui, risk_flags, CountStyle::YellowIfNonZero),
        style_count(ui, no_exec, CountStyle::DimAlways),
    );
}

pub(crate) fn print_warning(ui: &UiContext, warning: &str) {
    let prefix = if ui.colors_enabled() {
        "warning:".yellow().bold().to_string()
    } else {
        "warning:".to_string()
    };
    eprintln!("  {prefix} {warning}");
}

pub(crate) fn style_count_for_action(ui: &UiContext, action: &str, count: usize) -> String {
    let style = match action {
        "create" => CountStyle::GreenIfNonZero,
        "update" => CountStyle::CyanIfNonZero,
        "noop" => CountStyle::DimAlways,
        "conflict" => CountStyle::YellowIfNonZero,
        "remove" => CountStyle::RedIfNonZero,
        _ => CountStyle::Plain,
    };
    style_count(ui, count, style)
}

enum CountStyle {
    Plain,
    GreenIfNonZero,
    CyanIfNonZero,
    YellowIfNonZero,
    RedIfNonZero,
    DimAlways,
}

fn style_count(ui: &UiContext, count: usize, style: CountStyle) -> String {
    let raw = count.to_string();
    if !ui.colors_enabled() {
        return raw;
    }
    match style {
        CountStyle::Plain => raw,
        CountStyle::GreenIfNonZero if count > 0 => raw.green().to_string(),
        CountStyle::CyanIfNonZero if count > 0 => raw.cyan().to_string(),
        CountStyle::YellowIfNonZero if count > 0 => raw.yellow().to_string(),
        CountStyle::RedIfNonZero if count > 0 => raw.red().to_string(),
        CountStyle::DimAlways => raw.dimmed().to_string(),
        _ => raw,
    }
}

pub(crate) fn resolve_registry_mode_skills_for_execution(
    config_path: &Path,
    config: &Config,
    config_dir: &Path,
    ui: &UiContext,
) -> Result<Config, EdenError> {
    if !config
        .skills
        .iter()
        .any(|skill| is_registry_mode_repo(&skill.source.repo))
    {
        return Ok(config.clone());
    }

    let raw_toml = fs::read_to_string(config_path)?;
    let sorted_specs = sort_registry_specs_by_priority(
        &parse_registry_specs_from_toml(&raw_toml).map_err(EdenError::from)?,
    );
    if sorted_specs.is_empty() {
        return Err(EdenError::Runtime(
            "Registry index not found. Run `eden-skills update` first.".to_string(),
        ));
    }

    let storage_root = resolve_path_string(&config.storage_root, config_dir)?;
    let registries_root = storage_root.join("registries");
    let registry_sources = sorted_specs
        .into_iter()
        .map(|spec| RegistrySource {
            name: spec.name.clone(),
            priority: spec.priority,
            root: registries_root.join(spec.name),
        })
        .collect::<Vec<_>>();

    let mut resolved = config.clone();
    for skill in &mut resolved.skills {
        if !is_registry_mode_repo(&skill.source.repo) {
            continue;
        }

        let preferred_registry = decode_registry_mode_repo(&skill.source.repo)
            .unwrap_or(None)
            .unwrap_or_default();
        let sources_for_skill = if preferred_registry.is_empty() {
            registry_sources.clone()
        } else {
            registry_sources
                .iter()
                .filter(|source| source.name == preferred_registry)
                .cloned()
                .collect::<Vec<_>>()
        };

        if sources_for_skill.is_empty() {
            let repo_dir = storage_root.join(&skill.id);
            return Err(EdenError::Runtime(format!(
                "source sync failed for 1 skill(s): skill={} stage=clone repo_dir={} detail=registry `{}` is not configured",
                skill.id,
                repo_dir.display(),
                preferred_registry
            )));
        }
        for source in &sources_for_skill {
            validate_registry_manifest_for_resolution(source, ui)?;
        }
        if sources_for_skill.iter().all(|source| !source.root.exists()) {
            let repo_dir = storage_root.join(&skill.id);
            return Err(EdenError::Runtime(format!(
                "source sync failed for 1 skill(s): skill={} stage=clone repo_dir={} detail=Registry index not found. Run `eden-skills update` first.",
                skill.id,
                repo_dir.display()
            )));
        }

        let resolved_skill = resolve_skill_from_registry_sources(
            &sources_for_skill,
            &skill.id,
            Some(skill.source.r#ref.as_str()),
        )
        .map_err(|err| {
            let repo_dir = storage_root.join(&skill.id);
            EdenError::Runtime(format!(
                "source sync failed for 1 skill(s): skill={} stage=clone repo_dir={} detail=registry resolution failed: {}",
                skill.id,
                repo_dir.display(),
                err
            ))
        })?;

        skill.source = SourceConfig {
            repo: resolved_skill.repo,
            subpath: resolved_skill.subpath,
            r#ref: resolved_skill.git_ref,
        };
    }

    Ok(resolved)
}

fn validate_registry_manifest_for_resolution(
    source: &RegistrySource,
    ui: &UiContext,
) -> Result<(), EdenError> {
    if !source.root.exists() {
        return Ok(());
    }

    let manifest_path = source.root.join("manifest.toml");
    if !manifest_path.exists() {
        print_warning(
            ui,
            &format!(
                "registry `{}` is missing manifest.toml; assuming format_version = 1",
                source.name
            ),
        );
        return Ok(());
    }

    let raw_manifest = fs::read_to_string(&manifest_path)?;
    let manifest: toml::Value = toml::from_str(&raw_manifest).map_err(|err| {
        EdenError::Runtime(format!(
            "registry `{}` manifest.toml is invalid TOML: {err}",
            source.name
        ))
    })?;
    let Some(format_version) = manifest.get("format_version").and_then(|v| v.as_integer()) else {
        return Err(EdenError::Runtime(format!(
            "registry `{}` manifest.toml is missing `format_version`",
            source.name
        )));
    };
    if format_version != 1 {
        print_warning(
            ui,
            &format!(
                "registry `{}` manifest format_version={} (expected 1); continuing",
                source.name, format_version
            ),
        );
    }
    Ok(())
}

pub(crate) fn write_normalized_config(path: &Path, config: &Config) -> Result<(), EdenError> {
    let registries = read_existing_registries(path)?;
    let toml = normalized_config_toml(config, registries.as_ref());
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, toml)?;
    Ok(())
}

pub(crate) fn normalized_config_toml(
    config: &Config,
    registries: Option<&BTreeMap<String, ExistingRegistryConfig>>,
) -> String {
    let mut out = String::new();

    out.push_str(&format!("version = {}\n\n", config.version));
    out.push_str("[storage]\n");
    out.push_str(&format!(
        "root = \"{}\"\n\n",
        toml_escape_str(&config.storage_root)
    ));
    if config.reactor.concurrency != eden_skills_core::reactor::DEFAULT_CONCURRENCY_LIMIT {
        out.push_str("[reactor]\n");
        out.push_str(&format!("concurrency = {}\n\n", config.reactor.concurrency));
    }
    if let Some(registries) = registries {
        if !registries.is_empty() {
            out.push_str(&render_registries_toml(registries));
            out.push('\n');
        }
    }

    for skill in &config.skills {
        out.push_str(&normalized_skill_toml(skill));
        out.push('\n');
    }

    out
}

pub(crate) fn read_existing_registries(
    path: &Path,
) -> Result<Option<BTreeMap<String, ExistingRegistryConfig>>, EdenError> {
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(path)?;
    let parsed: ExistingRegistryFile = match toml::from_str(&raw) {
        Ok(value) => value,
        Err(_) => return Ok(None),
    };
    if parsed.registries.is_empty() {
        Ok(None)
    } else {
        Ok(Some(parsed.registries))
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
struct ExistingRegistryFile {
    #[serde(default)]
    registries: BTreeMap<String, ExistingRegistryConfig>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct ExistingRegistryConfig {
    pub(crate) url: String,
    pub(crate) priority: Option<i64>,
    pub(crate) auto_update: Option<bool>,
}

fn render_registries_toml(registries: &BTreeMap<String, ExistingRegistryConfig>) -> String {
    let mut out = String::new();
    out.push_str("[registries]\n");
    for (name, registry) in registries {
        out.push_str(&format!(
            "{} = {{ url = \"{}\"",
            toml_escape_str(name),
            toml_escape_str(&registry.url)
        ));
        if let Some(priority) = registry.priority {
            out.push_str(&format!(", priority = {priority}"));
        }
        if let Some(auto_update) = registry.auto_update {
            out.push_str(&format!(", auto_update = {auto_update}"));
        }
        out.push_str(" }\n");
    }
    out
}

fn normalized_skill_toml(skill: &SkillConfig) -> String {
    let mut out = String::new();

    out.push_str("[[skills]]\n");
    if is_registry_mode_repo(&skill.source.repo) {
        out.push_str(&format!("name = \"{}\"\n", toml_escape_str(&skill.id)));
        out.push_str(&format!(
            "version = \"{}\"\n",
            toml_escape_str(&skill.source.r#ref)
        ));
        if let Some(Some(registry_name)) = decode_registry_mode_repo(&skill.source.repo) {
            out.push_str(&format!(
                "registry = \"{}\"\n",
                toml_escape_str(&registry_name)
            ));
        }
        out.push('\n');
    } else {
        out.push_str(&format!("id = \"{}\"\n\n", toml_escape_str(&skill.id)));

        out.push_str("[skills.source]\n");
        out.push_str(&format!(
            "repo = \"{}\"\n",
            toml_escape_str(&skill.source.repo)
        ));
        out.push_str(&format!(
            "subpath = \"{}\"\n",
            toml_escape_str(&skill.source.subpath)
        ));
        out.push_str(&format!(
            "ref = \"{}\"\n\n",
            toml_escape_str(&skill.source.r#ref)
        ));
    }

    out.push_str("[skills.install]\n");
    out.push_str(&format!(
        "mode = \"{}\"\n\n",
        toml_escape_str(skill.install.mode.as_str())
    ));

    for target in &skill.targets {
        out.push_str(&normalized_target_toml(target));
        out.push('\n');
    }

    out.push_str("[skills.verify]\n");
    out.push_str(&format!("enabled = {}\n", skill.verify.enabled));
    out.push_str("checks = [");
    out.push_str(
        &skill
            .verify
            .checks
            .iter()
            .map(|c| format!("\"{}\"", toml_escape_str(c)))
            .collect::<Vec<_>>()
            .join(", "),
    );
    out.push_str("]\n\n");

    out.push_str("[skills.safety]\n");
    out.push_str(&format!(
        "no_exec_metadata_only = {}\n",
        skill.safety.no_exec_metadata_only
    ));

    out
}

fn normalized_target_toml(target: &TargetConfig) -> String {
    let mut out = String::new();
    out.push_str("[[skills.targets]]\n");
    out.push_str(&format!(
        "agent = \"{}\"\n",
        toml_escape_str(agent_kind_label(&target.agent))
    ));
    if let Some(expected) = &target.expected_path {
        out.push_str(&format!(
            "expected_path = \"{}\"\n",
            toml_escape_str(expected)
        ));
    }
    if let Some(path) = &target.path {
        out.push_str(&format!("path = \"{}\"\n", toml_escape_str(path)));
    }
    if target.environment != "local" {
        out.push_str(&format!(
            "environment = \"{}\"\n",
            toml_escape_str(&target.environment)
        ));
    }
    out
}

fn toml_escape_str(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\"', "\\\"")
}

pub(crate) fn write_lock_for_config(
    config_path: &Path,
    config: &Config,
    config_dir: &Path,
) -> Result<(), EdenError> {
    write_lock_for_config_with_commits(config_path, config, config_dir, &HashMap::new())
}

pub(crate) fn write_lock_for_config_with_commits(
    config_path: &Path,
    config: &Config,
    config_dir: &Path,
    resolved_commits: &HashMap<String, String>,
) -> Result<(), EdenError> {
    let lock_path = lock_path_for_config(config_path);
    let lock = build_lock_from_config(config, config_dir, resolved_commits)?;
    write_lock_file(&lock_path, &lock)
}

pub(crate) fn apply_plan_item(item: &PlanItem) -> Result<(), EdenError> {
    let source_path = PathBuf::from(&item.source_path);
    let target_path = PathBuf::from(&item.target_path);

    if !source_path.exists() {
        return Err(EdenError::Runtime(format!(
            "source path missing for skill `{}`: {}",
            item.skill_id, item.source_path
        )));
    }

    match item.install_mode {
        InstallMode::Symlink => apply_symlink(&source_path, &target_path),
        InstallMode::Copy => apply_copy(&source_path, &target_path),
    }
}

pub(crate) fn path_is_symlink_or_junction(path: &Path, metadata: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        metadata.file_type().is_symlink() || junction::exists(path).unwrap_or(false)
    }

    #[cfg(not(windows))]
    {
        let _ = path;
        metadata.file_type().is_symlink()
    }
}

fn apply_symlink(source_path: &Path, target_path: &Path) -> Result<(), EdenError> {
    ensure_parent_dir(target_path)?;
    if fs::symlink_metadata(target_path).is_ok() {
        remove_path(target_path)?;
    }

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(source_path, target_path)?;
    }
    #[cfg(windows)]
    {
        if source_path.is_dir() {
            match std::os::windows::fs::symlink_dir(source_path, target_path) {
                Ok(()) => {}
                Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => {
                    junction::create(source_path, target_path).map_err(|junction_err| {
                        map_windows_symlink_fallback_error(
                            err,
                            junction_err,
                            source_path,
                            target_path,
                        )
                    })?;
                }
                Err(err) => return Err(map_windows_symlink_error(err, source_path, target_path)),
            }
        } else {
            std::os::windows::fs::symlink_file(source_path, target_path)
                .map_err(|err| map_windows_symlink_error(err, source_path, target_path))?;
        }
    }

    Ok(())
}

#[cfg(windows)]
fn map_windows_symlink_fallback_error(
    symlink_err: std::io::Error,
    junction_err: std::io::Error,
    source_path: &Path,
    target_path: &Path,
) -> EdenError {
    EdenError::Runtime(format!(
        "failed to create symlink `{}` -> `{}`: {}. Enable Developer Mode or run as Administrator if symlink privileges are unavailable; otherwise verify write permissions for the target path. junction fallback failed: {}",
        target_path.display(),
        source_path.display(),
        symlink_err,
        junction_err
    ))
}

#[cfg(windows)]
fn map_windows_symlink_error(
    err: std::io::Error,
    source_path: &Path,
    target_path: &Path,
) -> EdenError {
    if err.kind() == std::io::ErrorKind::PermissionDenied {
        return EdenError::Runtime(format!(
            "failed to create symlink `{}` -> `{}`: {}. Enable Developer Mode or run as Administrator.",
            target_path.display(),
            source_path.display(),
            err
        ));
    }
    EdenError::Io(err)
}

fn apply_copy(source_path: &Path, target_path: &Path) -> Result<(), EdenError> {
    ensure_parent_dir(target_path)?;
    if fs::symlink_metadata(target_path).is_ok() {
        remove_path(target_path)?;
    }
    copy_recursively(source_path, target_path)
}

pub(crate) fn ensure_parent_dir(path: &Path) -> Result<(), EdenError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

pub(crate) fn remove_path(path: &Path) -> Result<(), EdenError> {
    let metadata = fs::symlink_metadata(path)?;
    if path_is_symlink_or_junction(path, &metadata) {
        remove_symlink_path(path)?;
        return Ok(());
    }
    if metadata.is_file() {
        fs::remove_file(path)?;
        return Ok(());
    }
    if metadata.is_dir() {
        fs::remove_dir_all(path)?;
    }
    Ok(())
}

#[cfg(not(windows))]
fn remove_symlink_path(path: &Path) -> Result<(), EdenError> {
    fs::remove_file(path)?;
    Ok(())
}

#[cfg(windows)]
fn remove_symlink_path(path: &Path) -> Result<(), EdenError> {
    if junction::exists(path).unwrap_or(false) {
        junction::delete(path).map_err(|err| {
            EdenError::Runtime(format!(
                "failed to delete junction `{}`: {err}",
                path.display()
            ))
        })?;
        match fs::remove_dir(path) {
            Ok(()) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => return Err(EdenError::Io(err)),
        }
        return Ok(());
    }

    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => {
            fs::remove_dir(path)?;
            Ok(())
        }
        Err(err) => Err(EdenError::Io(err)),
    }
}

pub(crate) fn copy_recursively(source: &Path, target: &Path) -> Result<(), EdenError> {
    if source.is_file() {
        fs::copy(source, target)?;
        return Ok(());
    }

    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let child_source = entry.path();
        let child_target = target.join(entry.file_name());
        if child_source.is_dir() {
            copy_recursively(&child_source, &child_target)?;
        } else {
            fs::copy(&child_source, &child_target)?;
        }
    }
    Ok(())
}
