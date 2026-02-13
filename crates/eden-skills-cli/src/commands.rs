use std::fs;
use std::path::{Path, PathBuf};

use eden_skills_core::config::InstallMode;
use eden_skills_core::config::{
    config_dir_from_path, default_verify_checks_for_mode, load_from_file, validate_config,
    LoadOptions,
};
use eden_skills_core::config::{AgentKind, Config, SkillConfig, TargetConfig};
use eden_skills_core::error::EdenError;
use eden_skills_core::paths::{resolve_path_string, resolve_target_path};
use eden_skills_core::plan::{build_plan, Action, PlanItem};
use eden_skills_core::source::sync_sources;
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

fn resolve_config_path(config_path: &str) -> Result<PathBuf, EdenError> {
    let cwd = std::env::current_dir().map_err(EdenError::Io)?;
    resolve_path_string(config_path, &cwd)
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

pub fn apply(config_path: &str, options: CommandOptions) -> Result<(), EdenError> {
    let config_path_buf = resolve_config_path(config_path)?;
    let config_path = config_path_buf.as_path();
    let loaded = load_from_file(
        config_path,
        LoadOptions {
            strict: options.strict,
        },
    )?;
    let config_dir = config_dir_from_path(config_path);
    let sync_summary = sync_sources(&loaded.config, &config_dir)?;
    println!(
        "source sync: cloned={} updated={} skipped={}",
        sync_summary.cloned, sync_summary.updated, sync_summary.skipped
    );
    let plan = build_plan(&loaded.config, &config_dir)?;

    let mut created = 0usize;
    let mut updated = 0usize;
    let mut noops = 0usize;
    let mut conflicts = 0usize;

    for item in &plan {
        match item.action {
            Action::Create => {
                apply_plan_item(item)?;
                created += 1;
            }
            Action::Update => {
                apply_plan_item(item)?;
                updated += 1;
            }
            Action::Noop => {
                noops += 1;
            }
            Action::Conflict => {
                conflicts += 1;
            }
        }
    }

    println!("apply summary: create={created} update={updated} noop={noops} conflict={conflicts}");

    if options.strict && conflicts > 0 {
        return Err(EdenError::Conflict(format!(
            "strict mode blocked apply: {conflicts} conflict entries"
        )));
    }

    let verify_issues = verify_config_state(&loaded.config, &config_dir)?;
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
    let findings = collect_doctor_findings(&plan, &verify_issues);

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

fn collect_doctor_findings(plan: &[PlanItem], verify_issues: &[VerifyIssue]) -> Vec<DoctorFinding> {
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

    findings
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
    let config_path_buf = resolve_config_path(config_path)?;
    let config_path = config_path_buf.as_path();
    let loaded = load_from_file(
        config_path,
        LoadOptions {
            strict: options.strict,
        },
    )?;
    let config_dir = config_dir_from_path(config_path);
    let sync_summary = sync_sources(&loaded.config, &config_dir)?;
    println!(
        "source sync: cloned={} updated={} skipped={}",
        sync_summary.cloned, sync_summary.updated, sync_summary.skipped
    );
    let plan = build_plan(&loaded.config, &config_dir)?;

    let mut repaired = 0usize;
    let mut skipped_conflicts = 0usize;

    for item in &plan {
        match item.action {
            Action::Create | Action::Update => {
                apply_plan_item(item)?;
                repaired += 1;
            }
            Action::Conflict => {
                skipped_conflicts += 1;
            }
            Action::Noop => {}
        }
    }

    println!("repair summary: repaired={repaired} skipped_conflicts={skipped_conflicts}");

    let verify_issues = verify_config_state(&loaded.config, &config_dir)?;
    if !verify_issues.is_empty() {
        return Err(EdenError::Runtime(format!(
            "post-repair verification failed with {} issue(s); first: [{}] {} {}",
            verify_issues.len(),
            verify_issues[0].check,
            verify_issues[0].skill_id,
            verify_issues[0].message
        )));
    }

    if options.strict && skipped_conflicts > 0 {
        return Err(EdenError::Conflict(format!(
            "repair skipped {skipped_conflicts} conflict entries in strict mode"
        )));
    }

    println!("repair verification: ok");
    Ok(())
}

fn print_plan_text(items: &[PlanItem]) {
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
    // Note: `skills` must be non-empty per `SPEC_SCHEMA.md`.
    [
        "version = 1",
        "",
        "[storage]",
        "root = \"~/.local/share/eden-skills/repos\"",
        "",
        "[[skills]]",
        "id = \"browser-tool\"",
        "",
        "[skills.source]",
        "repo = \"https://github.com/vercel-labs/skills.git\"",
        "subpath = \"packages/browser\"",
        "ref = \"main\"",
        "",
        "[skills.install]",
        "mode = \"symlink\"",
        "",
        "[[skills.targets]]",
        "agent = \"claude-code\"",
        "",
        "[[skills.targets]]",
        "agent = \"cursor\"",
        "",
        "[skills.verify]",
        "enabled = true",
        "checks = [\"path-exists\", \"target-resolves\", \"is-symlink\"]",
        "",
        "[skills.safety]",
        "no_exec_metadata_only = false",
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
            }),
            "cursor" => targets.push(TargetConfig {
                agent: AgentKind::Cursor,
                expected_path: None,
                path: None,
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
    let toml = normalized_config_toml(config);
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

    let toml = normalized_config_toml(&loaded.config);

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

    let toml = normalized_config_toml(&loaded.config);

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

fn normalized_config_toml(config: &Config) -> String {
    let mut out = String::new();

    out.push_str(&format!("version = {}\n\n", config.version));
    out.push_str("[storage]\n");
    out.push_str(&format!(
        "root = \"{}\"\n\n",
        toml_escape_str(&config.storage_root)
    ));

    for skill in &config.skills {
        out.push_str(&normalized_skill_toml(skill));
        out.push('\n');
    }

    out
}

fn normalized_skill_toml(skill: &SkillConfig) -> String {
    let mut out = String::new();

    out.push_str("[[skills]]\n");
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
            std::os::windows::fs::symlink_dir(source_path, target_path)?;
        } else {
            std::os::windows::fs::symlink_file(source_path, target_path)?;
        }
    }

    Ok(())
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
    if metadata.file_type().is_symlink() || metadata.is_file() {
        fs::remove_file(path)?;
        return Ok(());
    }
    if metadata.is_dir() {
        fs::remove_dir_all(path)?;
    }
    Ok(())
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
