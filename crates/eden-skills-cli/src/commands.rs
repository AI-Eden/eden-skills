use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use eden_skills_core::adapter::create_adapter;
use eden_skills_core::config::InstallMode;
use eden_skills_core::config::{
    config_dir_from_path, decode_registry_mode_repo, default_verify_checks_for_mode,
    encode_registry_mode_repo, is_registry_mode_repo, load_from_file, validate_config, LoadOptions,
    SourceConfig,
};
use eden_skills_core::config::{AgentKind, Config, SkillConfig, TargetConfig};
use eden_skills_core::error::EdenError;
use eden_skills_core::paths::{normalize_lexical, resolve_path_string, resolve_target_path};
use eden_skills_core::plan::{build_plan, Action, PlanItem};
use eden_skills_core::reactor::{SkillReactor, MAX_CONCURRENCY_LIMIT, MIN_CONCURRENCY_LIMIT};
use eden_skills_core::registry::{
    parse_registry_specs_from_toml, resolve_skill_from_registry_sources,
    sort_registry_specs_by_priority, RegistrySource,
};
use eden_skills_core::safety::{analyze_skills, persist_reports, LicenseStatus, SkillSafetyReport};
use eden_skills_core::source::{sync_sources_async, sync_sources_async_with_reactor, SyncSummary};
use eden_skills_core::source_format::{
    derive_skill_id_from_source_repo, detect_install_source, DetectedInstallSource,
    UrlInstallSource,
};
use eden_skills_core::verify::{verify_config_state, VerifyIssue};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CommandOptions {
    pub strict: bool,
    pub json: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DoctorFinding {
    code: String,
    severity: String,
    skill_id: String,
    target_path: String,
    message: String,
    remediation: String,
}

#[derive(Debug, Clone)]
pub struct UpdateRequest {
    pub config_path: String,
    pub concurrency: Option<usize>,
    pub options: CommandOptions,
}

#[derive(Debug, Clone)]
pub struct InstallRequest {
    pub config_path: String,
    pub source: String,
    pub id: Option<String>,
    pub r#ref: Option<String>,
    pub version: Option<String>,
    pub registry: Option<String>,
    pub target: Option<String>,
    pub dry_run: bool,
    pub options: CommandOptions,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RegistrySyncStatus {
    Cloned,
    Updated,
    Skipped,
    Failed,
}

impl RegistrySyncStatus {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Cloned => "cloned",
            Self::Updated => "updated",
            Self::Skipped => "skipped",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone)]
struct RegistrySyncTask {
    name: String,
    url: String,
    local_dir: PathBuf,
}

#[derive(Debug, Clone)]
struct RegistrySyncResult {
    name: String,
    status: RegistrySyncStatus,
    url: String,
    detail: Option<String>,
}

const REGISTRY_SYNC_MARKER_FILE: &str = ".eden-last-sync";
const REGISTRY_STALE_THRESHOLD_SECS: u64 = 7 * 24 * 60 * 60;

fn resolve_config_path(config_path: &str) -> Result<PathBuf, EdenError> {
    let cwd = std::env::current_dir().map_err(EdenError::Io)?;
    resolve_path_string(config_path, &cwd)
}

fn resolve_effective_reactor_concurrency(
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

pub fn plan(config_path: &str, options: CommandOptions) -> Result<(), EdenError> {
    let config_path_buf = resolve_config_path(config_path)?;
    let config_path = config_path_buf.as_path();
    let loaded = load_from_file(
        config_path,
        LoadOptions {
            strict: options.strict,
        },
    )?;
    for warning in loaded.warnings {
        eprintln!("warning: {warning}");
    }

    let config_dir = config_dir_from_path(config_path);
    let plan = build_plan(&loaded.config, &config_dir)?;
    if options.json {
        print_plan_json(&plan)?;
    } else {
        print_plan_text(&plan);
    }
    Ok(())
}

pub async fn update_async(req: UpdateRequest) -> Result<(), EdenError> {
    let config_path_buf = resolve_config_path(&req.config_path)?;
    let config_path = config_path_buf.as_path();
    let loaded = load_from_file(
        config_path,
        LoadOptions {
            strict: req.options.strict,
        },
    )?;
    for warning in loaded.warnings {
        eprintln!("warning: {warning}");
    }

    let raw_toml = fs::read_to_string(config_path)?;
    let registry_specs = sort_registry_specs_by_priority(
        &parse_registry_specs_from_toml(&raw_toml).map_err(EdenError::from)?,
    );
    if registry_specs.is_empty() {
        eprintln!("warning: no registries configured; skipping update");
        return Ok(());
    }

    let concurrency = resolve_effective_reactor_concurrency(
        req.concurrency,
        loaded.config.reactor.concurrency,
        "update.concurrency",
    )?;

    let config_dir = config_dir_from_path(config_path);
    let storage_root = resolve_path_string(&loaded.config.storage_root, &config_dir)?;
    let registries_root = storage_root.join("registries");
    tokio::fs::create_dir_all(&registries_root).await?;

    let tasks = registry_specs
        .into_iter()
        .map(|spec| {
            let name = spec.name;
            RegistrySyncTask {
                name: name.clone(),
                url: spec.url,
                local_dir: registries_root.join(name),
            }
        })
        .collect::<Vec<_>>();

    let reactor = SkillReactor::new(concurrency).map_err(EdenError::from)?;
    let started = Instant::now();
    let outcomes = reactor
        .run_phase_a(tasks, move |task| {
            let reactor = reactor;
            async move { sync_registry_task(task, reactor).await }
        })
        .await
        .map_err(EdenError::from)?;
    let elapsed_ms = started.elapsed().as_millis() as u64;

    let mut results = Vec::new();
    for outcome in outcomes {
        match outcome.result {
            Ok(result) | Err(result) => results.push(result),
        }
    }
    results.sort_by(|left, right| left.name.cmp(&right.name));

    let failed_count = results
        .iter()
        .filter(|result| matches!(result.status, RegistrySyncStatus::Failed))
        .count();

    if req.options.json {
        let payload = serde_json::json!({
            "registries": results.iter().map(|result| {
                serde_json::json!({
                    "name": result.name,
                    "status": result.status.as_str(),
                    "url": result.url,
                    "detail": result.detail,
                })
            }).collect::<Vec<_>>(),
            "failed": failed_count,
            "elapsed_ms": elapsed_ms,
        });
        let encoded = serde_json::to_string_pretty(&payload)
            .map_err(|err| EdenError::Runtime(format!("failed to encode update json: {err}")))?;
        println!("{encoded}");
    } else {
        let status_fragments = results
            .iter()
            .map(|result| format!("{}={}", result.name, result.status.as_str()))
            .collect::<Vec<_>>()
            .join(" ");
        let elapsed_seconds = elapsed_ms as f64 / 1000.0;
        println!(
            "registry sync: {status_fragments} ({failed_count} failed) [{elapsed_seconds:.1}s]"
        );
        for result in results
            .iter()
            .filter(|result| matches!(result.status, RegistrySyncStatus::Failed))
        {
            if let Some(detail) = &result.detail {
                eprintln!("warning: registry `{}` failed: {detail}", result.name);
            }
        }
    }

    Ok(())
}

pub async fn install_async(req: InstallRequest) -> Result<(), EdenError> {
    let config_path_buf = resolve_config_path(&req.config_path)?;
    let config_path = config_path_buf.as_path();
    ensure_install_config_exists(config_path)?;

    let cwd = std::env::current_dir().map_err(EdenError::Io)?;
    let detected_source = detect_install_source(&req.source, &cwd)?;
    match detected_source {
        DetectedInstallSource::RegistryName(skill_name) => {
            install_registry_mode_async(&req, config_path, &skill_name).await
        }
        DetectedInstallSource::Url(url_source) => {
            install_url_mode_async(&req, config_path, &url_source).await
        }
    }
}

fn ensure_install_config_exists(config_path: &Path) -> Result<(), EdenError> {
    if config_path.exists() {
        return Ok(());
    }

    let parent = config_path.parent().unwrap_or(Path::new("."));
    if !parent.exists() {
        return Err(EdenError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!(
                "config parent directory does not exist: {}",
                parent.display()
            ),
        )));
    }

    fs::write(config_path, default_config_template())?;
    println!("Created config at {}", config_path.display());
    Ok(())
}

async fn install_registry_mode_async(
    req: &InstallRequest,
    config_path: &Path,
    skill_name: &str,
) -> Result<(), EdenError> {
    if req.id.is_some() || req.r#ref.is_some() {
        return Err(EdenError::InvalidArguments(
            "--id/--ref are only supported for URL-mode install sources".to_string(),
        ));
    }

    let loaded = load_from_file(
        config_path,
        LoadOptions {
            strict: req.options.strict,
        },
    )?;
    for warning in loaded.warnings {
        eprintln!("warning: {warning}");
    }
    let config_dir = config_dir_from_path(config_path);

    let mut config = loaded.config;
    let requested_constraint = req.version.clone().unwrap_or_else(|| "*".to_string());
    upsert_mode_b_skill(
        &mut config,
        skill_name,
        &requested_constraint,
        req.registry.as_deref(),
        req.target.as_deref(),
    )?;
    validate_config(&config, &config_dir)?;

    let mut single_skill_config = select_single_skill_config(&config, skill_name)?;

    // Validate resolvability before sync/install, producing actionable errors when cache is missing.
    single_skill_config =
        resolve_registry_mode_skills_for_execution(config_path, &single_skill_config, &config_dir)?;

    if req.dry_run {
        print_install_dry_run(
            req.options.json,
            &single_skill_config,
            skill_name,
            &requested_constraint,
            &config_dir,
        )?;
        return Ok(());
    }

    write_normalized_config(config_path, &config)?;

    let sync_summary = sync_sources_async(&single_skill_config, &config_dir).await?;
    print_source_sync_summary(&sync_summary);
    if let Some(err) = source_sync_failure_error(&sync_summary) {
        return Err(err);
    }

    execute_install_plan(&single_skill_config, &config_dir, req.options.strict)?;
    print_install_success(req.options.json, skill_name, &requested_constraint)?;
    Ok(())
}

async fn install_url_mode_async(
    req: &InstallRequest,
    config_path: &Path,
    url_source: &UrlInstallSource,
) -> Result<(), EdenError> {
    let loaded = load_from_file(
        config_path,
        LoadOptions {
            strict: req.options.strict,
        },
    )?;
    for warning in loaded.warnings {
        eprintln!("warning: {warning}");
    }
    let config_dir = config_dir_from_path(config_path);

    let mut config = loaded.config;
    let skill_id = req
        .id
        .clone()
        .map(Ok)
        .unwrap_or_else(|| derive_skill_id_from_source_repo(&url_source.repo))?;
    let source_ref = req
        .r#ref
        .clone()
        .or_else(|| url_source.reference.clone())
        .unwrap_or_else(|| "main".to_string());
    let source_subpath = url_source
        .subpath
        .clone()
        .unwrap_or_else(|| ".".to_string());

    upsert_mode_a_skill(
        &mut config,
        &skill_id,
        &url_source.repo,
        &source_subpath,
        &source_ref,
        req.target.as_deref(),
    )?;
    validate_config(&config, &config_dir)?;
    let single_skill_config = select_single_skill_config(&config, &skill_id)?;

    if req.dry_run {
        print_install_dry_run(
            req.options.json,
            &single_skill_config,
            &skill_id,
            &source_ref,
            &config_dir,
        )?;
        return Ok(());
    }

    write_normalized_config(config_path, &config)?;

    if url_source.is_local {
        install_local_source_skill(&single_skill_config, &config_dir, req.options.strict)?;
    } else {
        let sync_summary = sync_sources_async(&single_skill_config, &config_dir).await?;
        print_source_sync_summary(&sync_summary);
        if let Some(err) = source_sync_failure_error(&sync_summary) {
            return Err(err);
        }
        execute_install_plan(&single_skill_config, &config_dir, req.options.strict)?;
    }

    print_install_success(req.options.json, &skill_id, &source_ref)?;
    Ok(())
}

fn select_single_skill_config(config: &Config, skill_id: &str) -> Result<Config, EdenError> {
    let single = Config {
        version: config.version,
        storage_root: config.storage_root.clone(),
        reactor: config.reactor,
        skills: config
            .skills
            .iter()
            .find(|skill| skill.id == skill_id)
            .cloned()
            .into_iter()
            .collect(),
    };
    if single.skills.is_empty() {
        return Err(EdenError::Runtime(format!(
            "failed to select installed skill `{skill_id}`"
        )));
    }
    Ok(single)
}

fn print_install_dry_run(
    json_mode: bool,
    single_skill_config: &Config,
    skill_id: &str,
    version_or_ref: &str,
    config_dir: &Path,
) -> Result<(), EdenError> {
    let resolved_skill = single_skill_config
        .skills
        .first()
        .ok_or_else(|| EdenError::Runtime("resolved install skill is missing".to_string()))?;
    let resolved_targets = resolved_skill
        .targets
        .iter()
        .map(|target| {
            let resolved_path = resolve_target_path(target, config_dir)
                .map(|path| path.display().to_string())
                .unwrap_or_else(|err| format!("ERROR: {err}"));
            serde_json::json!({
                "agent": agent_kind_label(&target.agent),
                "environment": target.environment,
                "path": resolved_path,
            })
        })
        .collect::<Vec<_>>();

    if json_mode {
        let payload = serde_json::json!({
            "skill": skill_id,
            "version": version_or_ref,
            "dry_run": true,
            "resolved": {
                "repo": resolved_skill.source.repo.clone(),
                "ref": resolved_skill.source.r#ref.clone(),
                "subpath": resolved_skill.source.subpath.clone(),
            },
            "targets": resolved_targets,
        });
        let encoded = serde_json::to_string_pretty(&payload)
            .map_err(|err| EdenError::Runtime(format!("failed to encode install json: {err}")))?;
        println!("{encoded}");
    } else {
        println!(
            "install dry-run: skill={} version={} repo={} ref={} subpath={}",
            skill_id,
            version_or_ref,
            resolved_skill.source.repo,
            resolved_skill.source.r#ref,
            resolved_skill.source.subpath
        );
        for target in &resolved_targets {
            println!(
                "  target agent={} environment={} path={}",
                target["agent"].as_str().unwrap_or("unknown"),
                target["environment"].as_str().unwrap_or("unknown"),
                target["path"].as_str().unwrap_or("unknown")
            );
        }
    }
    Ok(())
}

fn execute_install_plan(
    single_skill_config: &Config,
    config_dir: &Path,
    strict: bool,
) -> Result<(), EdenError> {
    let plan = build_plan(single_skill_config, config_dir)?;
    let mut conflicts = 0usize;
    for item in &plan {
        match item.action {
            Action::Create | Action::Update => apply_plan_item(item)?,
            Action::Conflict => conflicts += 1,
            Action::Noop => {}
        }
    }
    if strict && conflicts > 0 {
        return Err(EdenError::Conflict(format!(
            "strict mode blocked install: {conflicts} conflict entries"
        )));
    }
    Ok(())
}

fn install_local_source_skill(
    single_skill_config: &Config,
    config_dir: &Path,
    strict: bool,
) -> Result<(), EdenError> {
    let skill = single_skill_config
        .skills
        .first()
        .ok_or_else(|| EdenError::Runtime("local install skill is missing".to_string()))?;
    let source_root = resolve_path_string(&skill.source.repo, config_dir)?;
    let source_path = normalize_lexical(&source_root.join(&skill.source.subpath));
    if !source_path.exists() {
        return Err(EdenError::Runtime(format!(
            "source path missing for skill `{}`: {}",
            skill.id,
            source_path.display()
        )));
    }

    let mut conflicts = 0usize;
    for target in &skill.targets {
        let target_root = resolve_target_path(target, config_dir)?;
        let target_path = normalize_lexical(&target_root.join(&skill.id));
        match fs::symlink_metadata(&target_path) {
            Ok(metadata)
                if matches!(skill.install.mode, InstallMode::Symlink)
                    && !metadata.file_type().is_symlink() =>
            {
                conflicts += 1;
                continue;
            }
            Ok(metadata)
                if matches!(skill.install.mode, InstallMode::Copy)
                    && metadata.file_type().is_symlink() =>
            {
                conflicts += 1;
                continue;
            }
            Ok(_) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => return Err(EdenError::Io(err)),
        }

        let item = PlanItem {
            skill_id: skill.id.clone(),
            source_path: source_path.display().to_string(),
            target_path: target_path.display().to_string(),
            install_mode: skill.install.mode,
            action: Action::Update,
            reasons: vec![],
        };
        apply_plan_item(&item)?;
    }

    if strict && conflicts > 0 {
        return Err(EdenError::Conflict(format!(
            "strict mode blocked install: {conflicts} conflict entries"
        )));
    }

    Ok(())
}

fn print_install_success(
    json_mode: bool,
    skill_id: &str,
    version_or_ref: &str,
) -> Result<(), EdenError> {
    if json_mode {
        let payload = serde_json::json!({
            "skill": skill_id,
            "version": version_or_ref,
            "status": "installed",
        });
        let encoded = serde_json::to_string_pretty(&payload)
            .map_err(|err| EdenError::Runtime(format!("failed to encode install json: {err}")))?;
        println!("{encoded}");
    } else {
        println!("install: skill={} status=installed", skill_id);
    }
    Ok(())
}

async fn sync_registry_task(
    task: RegistrySyncTask,
    reactor: SkillReactor,
) -> Result<RegistrySyncResult, RegistrySyncResult> {
    let failed_name = task.name.clone();
    let failed_url = task.url.clone();

    let task_label = format!("sync registry `{}`", task.name);
    match reactor
        .run_blocking(&task_label, move || sync_registry_task_blocking(task))
        .await
    {
        Ok(Ok(result)) => Ok(result),
        Ok(Err(result)) => Err(result),
        Err(err) => Err(RegistrySyncResult {
            name: failed_name,
            status: RegistrySyncStatus::Failed,
            url: failed_url,
            detail: Some(err.to_string()),
        }),
    }
}

fn sync_registry_task_blocking(
    task: RegistrySyncTask,
) -> Result<Result<RegistrySyncResult, RegistrySyncResult>, EdenError> {
    let failed = |detail: String| RegistrySyncResult {
        name: task.name.clone(),
        status: RegistrySyncStatus::Failed,
        url: task.url.clone(),
        detail: Some(detail),
    };

    if let Some(parent) = task.local_dir.parent() {
        fs::create_dir_all(parent)?;
    }

    let git_dir = task.local_dir.join(".git");
    if !git_dir.exists() {
        let clone_result = run_git_command(
            Command::new("git")
                .arg("clone")
                .arg("--depth")
                .arg("1")
                .arg(&task.url)
                .arg(&task.local_dir),
            &format!("clone registry `{}`", task.name),
        );
        return Ok(match clone_result {
            Ok(_) => {
                write_registry_sync_marker(&task.local_dir)?;
                Ok(RegistrySyncResult {
                    name: task.name,
                    status: RegistrySyncStatus::Cloned,
                    url: task.url,
                    detail: None,
                })
            }
            Err(detail) => Err(failed(detail)),
        });
    }

    let head_before = read_head_sha(&task.local_dir);
    let fetch_result = run_git_command(
        Command::new("git")
            .arg("-C")
            .arg(&task.local_dir)
            .arg("fetch")
            .arg("--depth")
            .arg("1")
            .arg("origin"),
        &format!("fetch registry `{}`", task.name),
    );
    if let Err(detail) = fetch_result {
        return Ok(Err(failed(detail)));
    }

    let reset_result = run_git_command(
        Command::new("git")
            .arg("-C")
            .arg(&task.local_dir)
            .arg("reset")
            .arg("--hard")
            .arg("FETCH_HEAD"),
        &format!("reset registry `{}`", task.name),
    );
    if let Err(detail) = reset_result {
        return Ok(Err(failed(detail)));
    }

    let head_after = read_head_sha(&task.local_dir);
    let status = if head_before.is_some() && head_before == head_after {
        RegistrySyncStatus::Skipped
    } else {
        RegistrySyncStatus::Updated
    };
    write_registry_sync_marker(&task.local_dir)?;
    Ok(Ok(RegistrySyncResult {
        name: task.name,
        status,
        url: task.url,
        detail: None,
    }))
}

fn write_registry_sync_marker(registry_dir: &Path) -> Result<(), EdenError> {
    let now_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| EdenError::Runtime(format!("failed to get system time: {err}")))?
        .as_secs()
        .to_string();
    fs::write(registry_dir.join(REGISTRY_SYNC_MARKER_FILE), now_epoch)?;
    Ok(())
}

fn block_on_command_future<F>(future: F) -> Result<(), EdenError>
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

pub fn apply(config_path: &str, options: CommandOptions) -> Result<(), EdenError> {
    block_on_command_future(apply_async(config_path, options, None))
}

pub async fn apply_async(
    config_path: &str,
    options: CommandOptions,
    concurrency_override: Option<usize>,
) -> Result<(), EdenError> {
    let config_path_buf = resolve_config_path(config_path)?;
    let config_path = config_path_buf.as_path();
    let loaded = load_from_file(
        config_path,
        LoadOptions {
            strict: options.strict,
        },
    )?;
    let concurrency = resolve_effective_reactor_concurrency(
        concurrency_override,
        loaded.config.reactor.concurrency,
        "apply.concurrency",
    )?;
    let reactor = SkillReactor::new(concurrency).map_err(EdenError::from)?;
    let config_dir = config_dir_from_path(config_path);
    let execution_config =
        resolve_registry_mode_skills_for_execution(config_path, &loaded.config, &config_dir)?;
    let sync_summary =
        sync_sources_async_with_reactor(&execution_config, &config_dir, reactor).await?;
    print_source_sync_summary(&sync_summary);
    let safety_reports = analyze_skills(&execution_config, &config_dir)?;
    persist_reports(&safety_reports)?;
    print_safety_summary(&safety_reports);
    if let Some(err) = source_sync_failure_error(&sync_summary) {
        return Err(err);
    }

    let no_exec_skill_ids = no_exec_skill_ids(&safety_reports);
    let plan = build_plan(&execution_config, &config_dir)?;

    let mut created = 0usize;
    let mut updated = 0usize;
    let mut noops = 0usize;
    let mut conflicts = 0usize;
    let mut skipped_no_exec = 0usize;

    for item in &plan {
        match item.action {
            Action::Create => {
                if no_exec_skill_ids.contains(item.skill_id.as_str()) {
                    skipped_no_exec += 1;
                    continue;
                }
                apply_plan_item(item)?;
                created += 1;
            }
            Action::Update => {
                if no_exec_skill_ids.contains(item.skill_id.as_str()) {
                    skipped_no_exec += 1;
                    continue;
                }
                apply_plan_item(item)?;
                updated += 1;
            }
            Action::Noop => {
                noops += 1;
            }
            Action::Conflict => {
                if no_exec_skill_ids.contains(item.skill_id.as_str()) {
                    skipped_no_exec += 1;
                    continue;
                }
                conflicts += 1;
            }
        }
    }

    println!(
        "apply summary: create={created} update={updated} noop={noops} conflict={conflicts} skipped_no_exec={skipped_no_exec}"
    );

    if options.strict && conflicts > 0 {
        return Err(EdenError::Conflict(format!(
            "strict mode blocked apply: {conflicts} conflict entries"
        )));
    }

    let verify_issues = verify_config_state(&execution_config, &config_dir)?;
    if !verify_issues.is_empty() {
        return Err(EdenError::Runtime(format!(
            "post-apply verification failed with {} issue(s); first: [{}] {} {}",
            verify_issues.len(),
            verify_issues[0].check,
            verify_issues[0].skill_id,
            verify_issues[0].message
        )));
    }

    println!("apply verification: ok");
    Ok(())
}

pub fn doctor(config_path: &str, options: CommandOptions) -> Result<(), EdenError> {
    let config_path_buf = resolve_config_path(config_path)?;
    let config_path = config_path_buf.as_path();
    let loaded = load_from_file(
        config_path,
        LoadOptions {
            strict: options.strict,
        },
    )?;
    let config_dir = config_dir_from_path(config_path);
    let plan = build_plan(&loaded.config, &config_dir)?;
    let verify_issues = verify_config_state(&loaded.config, &config_dir)?;
    let safety_reports = analyze_skills(&loaded.config, &config_dir)?;
    let mut findings = collect_doctor_findings(&plan, &verify_issues, &safety_reports);
    findings.extend(collect_phase2_doctor_findings(
        config_path,
        &loaded.config,
        &config_dir,
    )?);

    if findings.is_empty() {
        println!("doctor: no issues detected");
        return Ok(());
    }

    if options.json {
        print_doctor_json(&findings)?;
    } else {
        print_doctor_text(&findings);
    }

    if options.strict {
        return Err(EdenError::Conflict(format!(
            "doctor found {} issue(s) in strict mode",
            findings.len()
        )));
    }
    Ok(())
}

fn collect_doctor_findings(
    plan: &[PlanItem],
    verify_issues: &[VerifyIssue],
    safety_reports: &[SkillSafetyReport],
) -> Vec<DoctorFinding> {
    let mut findings = Vec::new();

    for item in plan {
        if !matches!(item.action, Action::Conflict) {
            continue;
        }
        findings.extend(plan_conflict_to_findings(item));
    }

    for issue in verify_issues {
        findings.push(verify_issue_to_finding(issue));
    }

    findings.extend(safety_reports.iter().flat_map(safety_report_to_findings));

    findings
}

fn collect_phase2_doctor_findings(
    config_path: &Path,
    config: &Config,
    config_dir: &Path,
) -> Result<Vec<DoctorFinding>, EdenError> {
    let mut findings = Vec::new();
    findings.extend(collect_registry_stale_findings(
        config_path,
        config,
        config_dir,
    )?);
    findings.extend(collect_adapter_health_findings(config));
    Ok(findings)
}

fn collect_registry_stale_findings(
    config_path: &Path,
    config: &Config,
    config_dir: &Path,
) -> Result<Vec<DoctorFinding>, EdenError> {
    let raw_toml = fs::read_to_string(config_path)?;
    let registry_specs = sort_registry_specs_by_priority(
        &parse_registry_specs_from_toml(&raw_toml).map_err(EdenError::from)?,
    );
    if registry_specs.is_empty() {
        return Ok(Vec::new());
    }

    let storage_root = resolve_path_string(&config.storage_root, config_dir)?;
    let registries_root = storage_root.join("registries");
    let now = SystemTime::now();
    let threshold = Duration::from_secs(REGISTRY_STALE_THRESHOLD_SECS);
    let mut findings = Vec::new();

    for spec in registry_specs {
        let registry_dir = registries_root.join(&spec.name);
        let marker_path = registry_dir.join(REGISTRY_SYNC_MARKER_FILE);
        let stale_reason = if !registry_dir.exists() {
            Some("registry cache is missing".to_string())
        } else if !marker_path.exists() {
            Some("registry sync marker is missing".to_string())
        } else {
            let marker_raw = fs::read_to_string(&marker_path).unwrap_or_default();
            let marker_epoch = marker_raw.trim().parse::<u64>().ok();
            match marker_epoch {
                Some(epoch) => {
                    let last_synced = UNIX_EPOCH + Duration::from_secs(epoch);
                    match now.duration_since(last_synced) {
                        Ok(age) if age > threshold => Some(format!(
                            "registry cache last synced {} day(s) ago",
                            age.as_secs() / (24 * 60 * 60)
                        )),
                        _ => None,
                    }
                }
                None => Some("registry sync marker is invalid".to_string()),
            }
        };

        if let Some(reason) = stale_reason {
            findings.push(DoctorFinding {
                code: "REGISTRY_STALE".to_string(),
                severity: "warning".to_string(),
                skill_id: format!("registry:{}", spec.name),
                target_path: registry_dir.display().to_string(),
                message: format!("registry `{}` is stale: {reason}", spec.name),
                remediation: "Run `eden-skills update` to refresh local registry cache."
                    .to_string(),
            });
        }
    }

    Ok(findings)
}

fn collect_adapter_health_findings(config: &Config) -> Vec<DoctorFinding> {
    let mut findings = Vec::new();
    let docker_bin = doctor_docker_bin();
    for skill in &config.skills {
        for target in &skill.targets {
            let Some(container_name) = target.environment.strip_prefix("docker:") else {
                continue;
            };

            match Command::new(&docker_bin).arg("--version").output() {
                Ok(output) if output.status.success() => {}
                Ok(output) => {
                    findings.push(DoctorFinding {
                        code: "DOCKER_NOT_FOUND".to_string(),
                        severity: "error".to_string(),
                        skill_id: skill.id.clone(),
                        target_path: target
                            .path
                            .clone()
                            .unwrap_or_else(|| target.environment.clone()),
                        message: format!(
                            "docker CLI `{}` is unavailable for target `{}` (status={} stderr=`{}`)",
                            docker_bin,
                            target.environment,
                            output.status,
                            String::from_utf8_lossy(&output.stderr).trim()
                        ),
                        remediation: "Install Docker or ensure `docker` is available in PATH."
                            .to_string(),
                    });
                    continue;
                }
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                    findings.push(DoctorFinding {
                        code: "DOCKER_NOT_FOUND".to_string(),
                        severity: "error".to_string(),
                        skill_id: skill.id.clone(),
                        target_path: target
                            .path
                            .clone()
                            .unwrap_or_else(|| target.environment.clone()),
                        message: format!(
                            "docker CLI `{}` is unavailable for target `{}`: {err}",
                            docker_bin, target.environment
                        ),
                        remediation: "Install Docker or ensure `docker` is available in PATH."
                            .to_string(),
                    });
                    continue;
                }
                Err(err) => {
                    findings.push(DoctorFinding {
                        code: "DOCKER_NOT_FOUND".to_string(),
                        severity: "error".to_string(),
                        skill_id: skill.id.clone(),
                        target_path: target
                            .path
                            .clone()
                            .unwrap_or_else(|| target.environment.clone()),
                        message: format!(
                            "failed to invoke docker CLI `{}` for target `{}`: {err}",
                            docker_bin, target.environment
                        ),
                        remediation: "Install Docker or ensure `docker` is available in PATH."
                            .to_string(),
                    });
                    continue;
                }
            }

            let inspect = Command::new(&docker_bin)
                .args(["inspect", "--format", "{{.State.Running}}", container_name])
                .output();
            match inspect {
                Ok(output)
                    if output.status.success()
                        && String::from_utf8_lossy(&output.stdout).trim() == "true" => {}
                Ok(output) => {
                    findings.push(DoctorFinding {
                        code: "ADAPTER_HEALTH_FAIL".to_string(),
                        severity: "error".to_string(),
                        skill_id: skill.id.clone(),
                        target_path: target
                            .path
                            .clone()
                            .unwrap_or_else(|| target.environment.clone()),
                        message: format!(
                            "docker target `{}` failed health check (status={} stdout=`{}` stderr=`{}`)",
                            target.environment,
                            output.status,
                            String::from_utf8_lossy(&output.stdout).trim(),
                            String::from_utf8_lossy(&output.stderr).trim()
                        ),
                        remediation: format!(
                            "Start the container (`docker start {container_name}`) and retry."
                        ),
                    });
                }
                Err(err) => {
                    findings.push(DoctorFinding {
                        code: "ADAPTER_HEALTH_FAIL".to_string(),
                        severity: "error".to_string(),
                        skill_id: skill.id.clone(),
                        target_path: target
                            .path
                            .clone()
                            .unwrap_or_else(|| target.environment.clone()),
                        message: format!(
                            "docker health check invocation failed for target `{}`: {err}",
                            target.environment
                        ),
                        remediation: format!(
                            "Verify Docker daemon access and container `{container_name}` state."
                        ),
                    });
                }
            }
        }
    }
    findings
}

fn doctor_docker_bin() -> String {
    std::env::var("EDEN_SKILLS_DOCKER_BIN")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "docker".to_string())
}

fn plan_conflict_to_findings(item: &PlanItem) -> Vec<DoctorFinding> {
    item.reasons
        .iter()
        .map(|reason| {
            let (code, severity, remediation) = map_plan_reason(reason);
            DoctorFinding {
                code: code.to_string(),
                severity: severity.to_string(),
                skill_id: item.skill_id.clone(),
                target_path: item.target_path.clone(),
                message: reason.clone(),
                remediation: remediation.to_string(),
            }
        })
        .collect()
}

fn verify_issue_to_finding(issue: &VerifyIssue) -> DoctorFinding {
    let (code, severity, remediation) = map_verify_issue(issue);
    DoctorFinding {
        code: code.to_string(),
        severity: severity.to_string(),
        skill_id: issue.skill_id.clone(),
        target_path: issue.target_path.clone(),
        message: issue.message.clone(),
        remediation: remediation.to_string(),
    }
}

fn safety_report_to_findings(report: &SkillSafetyReport) -> Vec<DoctorFinding> {
    let mut findings = Vec::new();

    if report.no_exec_metadata_only {
        findings.push(DoctorFinding {
            code: "NO_EXEC_METADATA_ONLY".to_string(),
            severity: "warning".to_string(),
            skill_id: report.skill_id.clone(),
            target_path: report.source_path.display().to_string(),
            message: "install mutations are disabled by no_exec_metadata_only".to_string(),
            remediation: "Set `safety.no_exec_metadata_only = false` to re-enable apply/repair target mutations."
                .to_string(),
        });
    }

    match report.license_status {
        LicenseStatus::Permissive => {}
        LicenseStatus::NonPermissive => findings.push(DoctorFinding {
            code: "LICENSE_NON_PERMISSIVE".to_string(),
            severity: "warning".to_string(),
            skill_id: report.skill_id.clone(),
            target_path: report.source_path.display().to_string(),
            message: "repository license is not detected as permissive".to_string(),
            remediation: "Review license terms or switch this skill to metadata-only mode."
                .to_string(),
        }),
        LicenseStatus::Unknown => findings.push(DoctorFinding {
            code: "LICENSE_UNKNOWN".to_string(),
            severity: "warning".to_string(),
            skill_id: report.skill_id.clone(),
            target_path: report.source_path.display().to_string(),
            message: "repository license could not be determined".to_string(),
            remediation: "Add an explicit license file upstream, or use metadata-only mode."
                .to_string(),
        }),
    }

    if !report.risk_labels.is_empty() {
        findings.push(DoctorFinding {
            code: "RISK_REVIEW_REQUIRED".to_string(),
            severity: "warning".to_string(),
            skill_id: report.skill_id.clone(),
            target_path: report.source_path.display().to_string(),
            message: format!("risk labels detected: {}", report.risk_labels.join(",")),
            remediation: "Review flagged files before enabling execution in agent workflows."
                .to_string(),
        });
    }

    findings
}

fn map_plan_reason(reason: &str) -> (&'static str, &'static str, &'static str) {
    match reason {
        "source path does not exist" => (
            "SOURCE_MISSING",
            "error",
            "Run `eden-skills apply` to sync sources or correct storage/source settings.",
        ),
        "target exists but is not a symlink" => (
            "TARGET_NOT_SYMLINK",
            "error",
            "Remove/rename the conflicting target, or set `install.mode = \"copy\"`.",
        ),
        "target is a symlink but install mode is copy" => (
            "TARGET_MODE_MISMATCH",
            "error",
            "Remove the symlink target and re-run `eden-skills apply` in copy mode.",
        ),
        _ => (
            "PLAN_CONFLICT",
            "error",
            "Inspect plan output and align local state with config.",
        ),
    }
}

fn map_verify_issue(issue: &VerifyIssue) -> (&'static str, &'static str, &'static str) {
    match issue.check.as_str() {
        "path-exists" => (
            "TARGET_PATH_MISSING",
            "error",
            "Run `eden-skills apply` or `eden-skills repair` to recreate target paths.",
        ),
        "is-symlink" => {
            if issue.message.contains("not a symlink") {
                (
                    "TARGET_NOT_SYMLINK",
                    "error",
                    "Replace target with a symlink or switch install mode to copy.",
                )
            } else {
                (
                    "BROKEN_SYMLINK",
                    "error",
                    "Run `eden-skills repair` to recreate a valid symlink target.",
                )
            }
        }
        "target-resolves" => {
            if issue.message.contains("resolves to") {
                (
                    "TARGET_RESOLVE_MISMATCH",
                    "error",
                    "Run `eden-skills repair` to relink target to the configured source.",
                )
            } else {
                (
                    "BROKEN_SYMLINK",
                    "error",
                    "Run `eden-skills repair` to rebuild the unreadable/missing symlink.",
                )
            }
        }
        "content-present" => {
            if issue.message.contains("typically for copy mode") {
                (
                    "VERIFY_CHECK_MISMATCH",
                    "warning",
                    "Adjust `verify.checks` to match the configured install mode.",
                )
            } else {
                (
                    "TARGET_CONTENT_MISSING",
                    "error",
                    "Run `eden-skills apply` or `eden-skills repair` to restore copied content.",
                )
            }
        }
        _ => (
            "VERIFY_CHECK_FAILED",
            "error",
            "Review `verify.checks` and local target state.",
        ),
    }
}

fn print_doctor_text(findings: &[DoctorFinding]) {
    println!("doctor: detected {} issue(s)", findings.len());
    for finding in findings {
        println!(
            "  code={} severity={} skill={} target={} message={} remediation={}",
            finding.code,
            finding.severity,
            finding.skill_id,
            finding.target_path,
            finding.message,
            finding.remediation
        );
    }
}

fn print_doctor_json(findings: &[DoctorFinding]) -> Result<(), EdenError> {
    let error_count = findings.iter().filter(|f| f.severity == "error").count();
    let warning_count = findings.iter().filter(|f| f.severity == "warning").count();

    let payload = serde_json::json!({
        "summary": {
            "total": findings.len(),
            "error": error_count,
            "warning": warning_count,
        },
        "findings": findings
            .iter()
            .map(|f| {
                serde_json::json!({
                    "code": f.code,
                    "severity": f.severity,
                    "skill_id": f.skill_id,
                    "target_path": f.target_path,
                    "message": f.message,
                    "remediation": f.remediation,
                })
            })
            .collect::<Vec<_>>(),
    });

    let encoded = serde_json::to_string_pretty(&payload)
        .map_err(|err| EdenError::Runtime(format!("failed to serialize doctor json: {err}")))?;
    println!("{encoded}");
    Ok(())
}

pub fn repair(config_path: &str, options: CommandOptions) -> Result<(), EdenError> {
    block_on_command_future(repair_async(config_path, options, None))
}

pub async fn repair_async(
    config_path: &str,
    options: CommandOptions,
    concurrency_override: Option<usize>,
) -> Result<(), EdenError> {
    let config_path_buf = resolve_config_path(config_path)?;
    let config_path = config_path_buf.as_path();
    let loaded = load_from_file(
        config_path,
        LoadOptions {
            strict: options.strict,
        },
    )?;
    let concurrency = resolve_effective_reactor_concurrency(
        concurrency_override,
        loaded.config.reactor.concurrency,
        "repair.concurrency",
    )?;
    let reactor = SkillReactor::new(concurrency).map_err(EdenError::from)?;
    let config_dir = config_dir_from_path(config_path);
    let execution_config =
        resolve_registry_mode_skills_for_execution(config_path, &loaded.config, &config_dir)?;
    let sync_summary =
        sync_sources_async_with_reactor(&execution_config, &config_dir, reactor).await?;
    print_source_sync_summary(&sync_summary);
    let safety_reports = analyze_skills(&execution_config, &config_dir)?;
    persist_reports(&safety_reports)?;
    print_safety_summary(&safety_reports);
    if let Some(err) = source_sync_failure_error(&sync_summary) {
        return Err(err);
    }

    let no_exec_skill_ids = no_exec_skill_ids(&safety_reports);
    let plan = build_plan(&execution_config, &config_dir)?;

    let mut repaired = 0usize;
    let mut skipped_conflicts = 0usize;
    let mut skipped_no_exec = 0usize;

    for item in &plan {
        match item.action {
            Action::Create | Action::Update => {
                if no_exec_skill_ids.contains(item.skill_id.as_str()) {
                    skipped_no_exec += 1;
                    continue;
                }
                apply_plan_item(item)?;
                repaired += 1;
            }
            Action::Conflict => {
                if no_exec_skill_ids.contains(item.skill_id.as_str()) {
                    skipped_no_exec += 1;
                    continue;
                }
                skipped_conflicts += 1;
            }
            Action::Noop => {}
        }
    }

    println!(
        "repair summary: repaired={repaired} skipped_conflicts={skipped_conflicts} skipped_no_exec={skipped_no_exec}"
    );

    if options.strict && skipped_conflicts > 0 {
        return Err(EdenError::Conflict(format!(
            "repair skipped {skipped_conflicts} conflict entries in strict mode"
        )));
    }

    let verify_issues = verify_config_state(&execution_config, &config_dir)?;
    if !verify_issues.is_empty() {
        return Err(EdenError::Runtime(format!(
            "post-repair verification failed with {} issue(s); first: [{}] {} {}",
            verify_issues.len(),
            verify_issues[0].check,
            verify_issues[0].skill_id,
            verify_issues[0].message
        )));
    }

    println!("repair verification: ok");
    Ok(())
}

fn no_exec_skill_ids(reports: &[SkillSafetyReport]) -> HashSet<&str> {
    reports
        .iter()
        .filter(|r| r.no_exec_metadata_only)
        .map(|r| r.skill_id.as_str())
        .collect()
}

fn print_source_sync_summary(summary: &SyncSummary) {
    println!(
        "source sync: cloned={} updated={} skipped={} failed={}",
        summary.cloned, summary.updated, summary.skipped, summary.failed
    );
}

fn source_sync_failure_error(summary: &SyncSummary) -> Option<EdenError> {
    if summary.failed == 0 {
        return None;
    }

    let details = summary
        .failures
        .iter()
        .map(|failure| {
            format!(
                "skill={} stage={} repo_dir={} detail={}",
                failure.skill_id,
                failure.stage.as_str(),
                failure.repo_dir,
                failure.detail
            )
        })
        .collect::<Vec<_>>()
        .join("; ");

    Some(EdenError::Runtime(format!(
        "source sync failed for {} skill(s): {details}",
        summary.failed
    )))
}

fn resolve_registry_mode_skills_for_execution(
    config_path: &Path,
    config: &Config,
    config_dir: &Path,
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
            validate_registry_manifest_for_resolution(source)?;
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

fn validate_registry_manifest_for_resolution(source: &RegistrySource) -> Result<(), EdenError> {
    if !source.root.exists() {
        return Ok(());
    }

    let manifest_path = source.root.join("manifest.toml");
    if !manifest_path.exists() {
        eprintln!(
            "warning: registry `{}` is missing manifest.toml; assuming format_version = 1",
            source.name
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
        eprintln!(
            "warning: registry `{}` manifest format_version={} (expected 1); continuing",
            source.name, format_version
        );
    }
    Ok(())
}

fn upsert_mode_b_skill(
    config: &mut Config,
    skill_name: &str,
    version_constraint: &str,
    registry: Option<&str>,
    target_spec: Option<&str>,
) -> Result<(), EdenError> {
    let target_override = target_spec.map(parse_install_target_spec).transpose()?;
    if let Some(skill) = config
        .skills
        .iter_mut()
        .find(|skill| skill.id == skill_name)
    {
        skill.source = SourceConfig {
            repo: encode_registry_mode_repo(registry),
            subpath: ".".to_string(),
            r#ref: version_constraint.to_string(),
        };
        if let Some(target) = target_override {
            skill.targets = vec![target];
        }
        return Ok(());
    }

    let targets = if let Some(target) = target_override {
        vec![target]
    } else {
        vec![default_install_target()]
    };

    config.skills.push(SkillConfig {
        id: skill_name.to_string(),
        source: SourceConfig {
            repo: encode_registry_mode_repo(registry),
            subpath: ".".to_string(),
            r#ref: version_constraint.to_string(),
        },
        install: eden_skills_core::config::InstallConfig {
            mode: InstallMode::Symlink,
        },
        targets,
        verify: eden_skills_core::config::VerifyConfig {
            enabled: true,
            checks: default_verify_checks_for_mode(InstallMode::Symlink),
        },
        safety: eden_skills_core::config::SafetyConfig {
            no_exec_metadata_only: false,
        },
    });
    Ok(())
}

fn upsert_mode_a_skill(
    config: &mut Config,
    skill_id: &str,
    repo: &str,
    subpath: &str,
    reference: &str,
    target_spec: Option<&str>,
) -> Result<(), EdenError> {
    let target_override = target_spec.map(parse_install_target_spec).transpose()?;
    if let Some(skill) = config.skills.iter_mut().find(|skill| skill.id == skill_id) {
        skill.source = SourceConfig {
            repo: repo.to_string(),
            subpath: subpath.to_string(),
            r#ref: reference.to_string(),
        };
        if let Some(target) = target_override {
            skill.targets = vec![target];
        }
        return Ok(());
    }

    let targets = if let Some(target) = target_override {
        vec![target]
    } else {
        vec![default_install_target()]
    };

    config.skills.push(SkillConfig {
        id: skill_id.to_string(),
        source: SourceConfig {
            repo: repo.to_string(),
            subpath: subpath.to_string(),
            r#ref: reference.to_string(),
        },
        install: eden_skills_core::config::InstallConfig {
            mode: InstallMode::Symlink,
        },
        targets,
        verify: eden_skills_core::config::VerifyConfig {
            enabled: true,
            checks: default_verify_checks_for_mode(InstallMode::Symlink),
        },
        safety: eden_skills_core::config::SafetyConfig {
            no_exec_metadata_only: false,
        },
    });
    Ok(())
}

fn default_install_target() -> TargetConfig {
    TargetConfig {
        agent: AgentKind::ClaudeCode,
        expected_path: None,
        path: None,
        environment: "local".to_string(),
    }
}

fn parse_install_target_spec(spec: &str) -> Result<TargetConfig, EdenError> {
    if spec == "local" {
        return Ok(default_install_target());
    }
    if let Some(container_name) = spec.strip_prefix("docker:") {
        if container_name.trim().is_empty() {
            return Err(EdenError::InvalidArguments(
                "invalid --target `docker:`: container name is required".to_string(),
            ));
        }
        return Ok(TargetConfig {
            agent: AgentKind::ClaudeCode,
            expected_path: None,
            path: None,
            environment: spec.to_string(),
        });
    }
    Err(EdenError::InvalidArguments(format!(
        "invalid --target `{spec}` (expected `local` or `docker:<container>`)"
    )))
}

fn run_git_command(command: &mut Command, context: &str) -> Result<String, String> {
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

fn read_head_sha(repo_dir: &Path) -> Option<String> {
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

fn print_safety_summary(reports: &[SkillSafetyReport]) {
    let permissive = reports
        .iter()
        .filter(|r| matches!(r.license_status, LicenseStatus::Permissive))
        .count();
    let non_permissive = reports
        .iter()
        .filter(|r| matches!(r.license_status, LicenseStatus::NonPermissive))
        .count();
    let unknown = reports
        .iter()
        .filter(|r| matches!(r.license_status, LicenseStatus::Unknown))
        .count();
    let risk_labeled = reports.iter().filter(|r| !r.risk_labels.is_empty()).count();
    let no_exec = reports.iter().filter(|r| r.no_exec_metadata_only).count();

    println!(
        "safety summary: permissive={permissive} non_permissive={non_permissive} unknown={unknown} risk_labeled={risk_labeled} no_exec={no_exec}"
    );
}

fn print_plan_text(items: &[PlanItem]) {
    if items.is_empty() {
        println!("plan: 0 actions");
        return;
    }

    for item in items {
        println!(
            "{} {} {} -> {} ({})",
            action_label(item.action),
            item.skill_id,
            item.source_path,
            item.target_path,
            item.install_mode.as_str()
        );
        for reason in &item.reasons {
            println!("  reason: {reason}");
        }
    }
}

fn print_plan_json(items: &[PlanItem]) -> Result<(), EdenError> {
    let payload = serde_json::to_string_pretty(items)
        .map_err(|err| EdenError::Runtime(format!("failed to serialize plan as json: {err}")))?;
    println!("{payload}");
    Ok(())
}

fn action_label(action: Action) -> &'static str {
    match action {
        Action::Create => "create",
        Action::Update => "update",
        Action::Noop => "noop",
        Action::Conflict => "conflict",
    }
}

pub fn init(config_path: &str, force: bool) -> Result<(), EdenError> {
    let config_path = resolve_config_path(config_path)?;
    if config_path.exists() && !force {
        return Err(EdenError::Conflict(format!(
            "config already exists: {} (use --force to overwrite)",
            config_path.display()
        )));
    }

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&config_path, default_config_template())?;
    println!("init: wrote {}", config_path.display());
    Ok(())
}

fn default_config_template() -> String {
    // Keep this template valid and deterministic.
    [
        "version = 1",
        "",
        "[storage]",
        "root = \"~/.local/share/eden-skills/repos\"",
        "",
    ]
    .join("\n")
}

pub fn list(config_path: &str, options: CommandOptions) -> Result<(), EdenError> {
    let config_path_buf = resolve_config_path(config_path)?;
    let config_path = config_path_buf.as_path();
    let loaded = load_from_file(
        config_path,
        LoadOptions {
            strict: options.strict,
        },
    )?;
    for warning in loaded.warnings {
        eprintln!("warning: {warning}");
    }

    let config_dir = config_dir_from_path(config_path);
    let skills = &loaded.config.skills;

    if options.json {
        let payload = serde_json::json!({
            "count": skills.len(),
            "skills": skills.iter().map(|skill| {
                let targets = skill.targets.iter().map(|target| {
                    let resolved = resolve_target_path(target, &config_dir)
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|err| format!("ERROR: {err}"));

                    serde_json::json!({
                        "agent": agent_kind_label(&target.agent),
                        "path": resolved,
                    })
                }).collect::<Vec<_>>();

                serde_json::json!({
                    "id": skill.id,
                    "source": {
                        "repo": skill.source.repo,
                        "ref": skill.source.r#ref,
                        "subpath": skill.source.subpath,
                    },
                    "install": {
                        "mode": skill.install.mode.as_str(),
                    },
                    "verify": {
                        "enabled": skill.verify.enabled,
                        "checks": skill.verify.checks,
                    },
                    "targets": targets,
                })
            }).collect::<Vec<_>>(),
        });

        let encoded = serde_json::to_string_pretty(&payload)
            .map_err(|err| EdenError::Runtime(format!("failed to serialize list json: {err}")))?;
        println!("{encoded}");
        return Ok(());
    }

    println!("list: {} skill(s)", skills.len());
    for skill in skills {
        println!(
            "skill id={} mode={} repo={} ref={} subpath={}",
            skill.id,
            skill.install.mode.as_str(),
            skill.source.repo,
            skill.source.r#ref,
            skill.source.subpath
        );
        println!(
            "  verify enabled={} checks={}",
            skill.verify.enabled,
            skill.verify.checks.join(",")
        );
        for target in &skill.targets {
            let resolved = resolve_target_path(target, &config_dir)
                .map(|p| p.display().to_string())
                .unwrap_or_else(|err| format!("ERROR: {err}"));
            println!(
                "  target agent={} path={}",
                agent_kind_label(&target.agent),
                resolved
            );
        }
    }

    Ok(())
}

#[derive(Debug, Clone)]
pub struct AddRequest {
    pub config_path: String,
    pub id: String,
    pub repo: String,
    pub r#ref: String,
    pub subpath: String,
    pub mode: InstallMode,
    pub target_specs: Vec<String>,
    pub verify_enabled: Option<bool>,
    pub verify_checks: Option<Vec<String>>,
    pub no_exec_metadata_only: Option<bool>,
    pub options: CommandOptions,
}

pub fn add(req: AddRequest) -> Result<(), EdenError> {
    let config_path_buf = resolve_config_path(&req.config_path)?;
    let config_path = config_path_buf.as_path();
    let loaded = load_from_file(
        config_path,
        LoadOptions {
            strict: req.options.strict,
        },
    )?;
    for warning in loaded.warnings {
        eprintln!("warning: {warning}");
    }

    let config_dir = config_dir_from_path(config_path);
    let mut config = loaded.config;

    if config.skills.iter().any(|s| s.id == req.id) {
        return Err(EdenError::InvalidArguments(format!(
            "skill id already exists: `{}`",
            req.id
        )));
    }

    let targets = parse_target_specs(&req.target_specs)?;
    let enabled = req.verify_enabled.unwrap_or(true);
    let checks = req
        .verify_checks
        .clone()
        .unwrap_or_else(|| default_verify_checks_for_mode(req.mode));

    let skill = SkillConfig {
        id: req.id.clone(),
        source: eden_skills_core::config::SourceConfig {
            repo: req.repo.clone(),
            subpath: req.subpath.clone(),
            r#ref: req.r#ref.clone(),
        },
        install: eden_skills_core::config::InstallConfig { mode: req.mode },
        targets,
        verify: eden_skills_core::config::VerifyConfig { enabled, checks },
        safety: eden_skills_core::config::SafetyConfig {
            no_exec_metadata_only: req.no_exec_metadata_only.unwrap_or(false),
        },
    };

    config.skills.push(skill);

    validate_config(&config, &config_dir)?;
    write_normalized_config(config_path, &config)?;

    if req.options.json {
        let payload = serde_json::json!({
            "action": "add",
            "config_path": config_path.display().to_string(),
            "skill_id": req.id,
        });
        let encoded = serde_json::to_string_pretty(&payload)
            .map_err(|err| EdenError::Runtime(format!("failed to serialize add json: {err}")))?;
        println!("{encoded}");
        return Ok(());
    }

    println!("add: wrote {}", config_path.display());
    Ok(())
}

pub fn remove(config_path: &str, skill_id: &str, options: CommandOptions) -> Result<(), EdenError> {
    block_on_command_future(remove_async(config_path, skill_id, options))
}

pub async fn remove_async(
    config_path: &str,
    skill_id: &str,
    options: CommandOptions,
) -> Result<(), EdenError> {
    let config_path_buf = resolve_config_path(config_path)?;
    let config_path = config_path_buf.as_path();
    let loaded = load_from_file(
        config_path,
        LoadOptions {
            strict: options.strict,
        },
    )?;
    for warning in loaded.warnings {
        eprintln!("warning: {warning}");
    }

    let config_dir = config_dir_from_path(config_path);
    let mut config = loaded.config;

    let Some(idx) = config.skills.iter().position(|s| s.id == skill_id) else {
        return Err(EdenError::InvalidArguments(format!(
            "unknown skill id: `{skill_id}`"
        )));
    };

    let removed_skill = config.skills[idx].clone();
    uninstall_skill_targets(&removed_skill, &config_dir).await?;
    config.skills.remove(idx);
    validate_config(&config, &config_dir)?;
    write_normalized_config(config_path, &config)?;

    if options.json {
        let payload = serde_json::json!({
            "action": "remove",
            "config_path": config_path.display().to_string(),
            "skill_id": skill_id,
        });
        let encoded = serde_json::to_string_pretty(&payload)
            .map_err(|err| EdenError::Runtime(format!("failed to serialize remove json: {err}")))?;
        println!("{encoded}");
        return Ok(());
    }

    println!("remove: wrote {}", config_path.display());
    Ok(())
}

async fn uninstall_skill_targets(skill: &SkillConfig, config_dir: &Path) -> Result<(), EdenError> {
    for target in &skill.targets {
        let target_root = resolve_target_path(target, config_dir)?;
        let installed_target = normalize_lexical(&target_root.join(&skill.id));
        let adapter = create_adapter(&target.environment).map_err(EdenError::from)?;
        adapter
            .uninstall(&installed_target)
            .await
            .map_err(EdenError::from)?;
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub struct SetRequest {
    pub config_path: String,
    pub skill_id: String,
    pub repo: Option<String>,
    pub r#ref: Option<String>,
    pub subpath: Option<String>,
    pub mode: Option<InstallMode>,
    pub verify_enabled: Option<bool>,
    pub verify_checks: Option<Vec<String>>,
    pub target_specs: Option<Vec<String>>,
    pub no_exec_metadata_only: Option<bool>,
    pub options: CommandOptions,
}

pub fn set(req: SetRequest) -> Result<(), EdenError> {
    let has_any_mutation = req.repo.is_some()
        || req.r#ref.is_some()
        || req.subpath.is_some()
        || req.mode.is_some()
        || req.verify_enabled.is_some()
        || req.verify_checks.is_some()
        || req.target_specs.is_some()
        || req.no_exec_metadata_only.is_some();
    if !has_any_mutation {
        return Err(EdenError::InvalidArguments(
            "set requires at least one mutation flag".to_string(),
        ));
    }

    let config_path_buf = resolve_config_path(&req.config_path)?;
    let config_path = config_path_buf.as_path();
    let loaded = load_from_file(
        config_path,
        LoadOptions {
            strict: req.options.strict,
        },
    )?;
    for warning in loaded.warnings {
        eprintln!("warning: {warning}");
    }

    let config_dir = config_dir_from_path(config_path);
    let mut config = loaded.config;

    let Some(skill) = config.skills.iter_mut().find(|s| s.id == req.skill_id) else {
        return Err(EdenError::InvalidArguments(format!(
            "unknown skill id: `{}`",
            req.skill_id
        )));
    };

    if let Some(repo) = req.repo {
        skill.source.repo = repo;
    }
    if let Some(r#ref) = req.r#ref {
        skill.source.r#ref = r#ref;
    }
    if let Some(subpath) = req.subpath {
        skill.source.subpath = subpath;
    }
    if let Some(mode) = req.mode {
        skill.install.mode = mode;
    }
    if let Some(enabled) = req.verify_enabled {
        skill.verify.enabled = enabled;
    }
    if let Some(checks) = req.verify_checks {
        skill.verify.checks = checks;
    }
    if let Some(target_specs) = req.target_specs {
        skill.targets = parse_target_specs(&target_specs)?;
    }
    if let Some(flag) = req.no_exec_metadata_only {
        skill.safety.no_exec_metadata_only = flag;
    }

    validate_config(&config, &config_dir)?;
    write_normalized_config(config_path, &config)?;

    if req.options.json {
        let payload = serde_json::json!({
            "action": "set",
            "config_path": config_path.display().to_string(),
            "skill_id": req.skill_id,
        });
        let encoded = serde_json::to_string_pretty(&payload)
            .map_err(|err| EdenError::Runtime(format!("failed to serialize set json: {err}")))?;
        println!("{encoded}");
        return Ok(());
    }

    println!("set: wrote {}", config_path.display());
    Ok(())
}

fn agent_kind_label(agent: &AgentKind) -> &'static str {
    match agent {
        AgentKind::ClaudeCode => "claude-code",
        AgentKind::Cursor => "cursor",
        AgentKind::Custom => "custom",
    }
}

fn parse_target_specs(specs: &[String]) -> Result<Vec<TargetConfig>, EdenError> {
    let mut targets = Vec::with_capacity(specs.len());
    for spec in specs {
        match spec.as_str() {
            "claude-code" => targets.push(TargetConfig {
                agent: AgentKind::ClaudeCode,
                expected_path: None,
                path: None,
                environment: "local".to_string(),
            }),
            "cursor" => targets.push(TargetConfig {
                agent: AgentKind::Cursor,
                expected_path: None,
                path: None,
                environment: "local".to_string(),
            }),
            _ => {
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
                    "invalid target spec `{spec}` (expected `claude-code`, `cursor`, or `custom:<path>`)"
                )));
            }
        }
    }
    Ok(targets)
}

fn write_normalized_config(path: &Path, config: &Config) -> Result<(), EdenError> {
    let registries = read_existing_registries(path)?;
    let toml = normalized_config_toml(config, registries.as_ref());
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, toml)?;
    Ok(())
}

pub fn config_export(config_path: &str, options: CommandOptions) -> Result<(), EdenError> {
    let config_path_buf = resolve_config_path(config_path)?;
    let config_path = config_path_buf.as_path();
    let loaded = load_from_file(
        config_path,
        LoadOptions {
            strict: options.strict,
        },
    )?;
    for warning in loaded.warnings {
        eprintln!("warning: {warning}");
    }

    let registries = read_existing_registries(config_path)?;
    let toml = normalized_config_toml(&loaded.config, registries.as_ref());

    if options.json {
        let payload = serde_json::json!({
            "format": "toml",
            "toml": toml,
        });
        let encoded = serde_json::to_string_pretty(&payload).map_err(|err| {
            EdenError::Runtime(format!("failed to serialize config export json: {err}"))
        })?;
        println!("{encoded}");
        return Ok(());
    }

    print!("{toml}");
    Ok(())
}

pub fn config_import(
    from_path: &str,
    config_path: &str,
    dry_run: bool,
    options: CommandOptions,
) -> Result<(), EdenError> {
    let cwd = std::env::current_dir().map_err(EdenError::Io)?;
    let from_path = resolve_path_string(from_path, &cwd)?;
    let loaded = load_from_file(
        &from_path,
        LoadOptions {
            strict: options.strict,
        },
    )?;
    for warning in loaded.warnings {
        eprintln!("warning: {warning}");
    }

    let registries = read_existing_registries(&from_path)?;
    let toml = normalized_config_toml(&loaded.config, registries.as_ref());

    if dry_run {
        print!("{toml}");
        return Ok(());
    }

    let dest_path = resolve_config_path(config_path)?;
    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&dest_path, toml)?;
    println!("config import: wrote {}", dest_path.display());
    Ok(())
}

fn normalized_config_toml(
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

#[derive(Debug, Clone, serde::Deserialize)]
struct ExistingRegistryFile {
    #[serde(default)]
    registries: BTreeMap<String, ExistingRegistryConfig>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct ExistingRegistryConfig {
    url: String,
    priority: Option<i64>,
    auto_update: Option<bool>,
}

fn read_existing_registries(
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

fn apply_plan_item(item: &PlanItem) -> Result<(), EdenError> {
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
            std::os::windows::fs::symlink_dir(source_path, target_path)
                .map_err(|err| map_windows_symlink_error(err, source_path, target_path))?;
        } else {
            std::os::windows::fs::symlink_file(source_path, target_path)
                .map_err(|err| map_windows_symlink_error(err, source_path, target_path))?;
        }
    }

    Ok(())
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

fn ensure_parent_dir(path: &Path) -> Result<(), EdenError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

fn remove_path(path: &Path) -> Result<(), EdenError> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
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
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => {
            fs::remove_dir(path)?;
            Ok(())
        }
        Err(err) => Err(EdenError::Io(err)),
    }
}

fn copy_recursively(source: &Path, target: &Path) -> Result<(), EdenError> {
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
