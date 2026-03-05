//! Health diagnostics via the `doctor` command.
//!
//! Collects findings from plan conflicts, verification issues, safety
//! reports, adapter health checks, and stale registry markers. Renders
//! results as severity-tagged cards in human mode or as a JSON array.

use std::fs;
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use comfy_table::{ColumnConstraint, Width};
use eden_skills_core::config::{config_dir_from_path, Config};
use eden_skills_core::error::EdenError;
use eden_skills_core::paths::resolve_path_string;
use eden_skills_core::plan::{build_plan, Action, PlanItem};
use eden_skills_core::registry::{parse_registry_specs_from_toml, sort_registry_specs_by_priority};
use eden_skills_core::safety::{analyze_skills, LicenseStatus, SkillSafetyReport};
use eden_skills_core::verify::{verify_config_state, VerifyIssue};
use owo_colors::OwoColorize;

use super::common::{
    doctor_docker_bin, load_config_with_context, resolve_config_path, REGISTRY_SYNC_MARKER_FILE,
};
use super::CommandOptions;
use crate::ui::{StatusSymbol, UiContext};

const REGISTRY_STALE_THRESHOLD_SECS: u64 = 7 * 24 * 60 * 60;

#[derive(Debug, Clone)]
pub(crate) struct DoctorFinding {
    pub(crate) code: String,
    pub(crate) severity: String,
    pub(crate) skill_id: String,
    pub(crate) target_path: String,
    pub(crate) message: String,
    pub(crate) remediation: String,
}

/// Diagnose configuration and installation health.
///
/// Collects findings from plan conflicts, verification issues, safety
/// reports, Phase 2 adapter health, and stale registry markers. Outputs
/// severity-tagged cards (human) or a JSON array (`--json`).
///
/// # Errors
///
/// Returns [`EdenError`] on config load failure or plan build errors.
pub fn doctor(config_path: &str, options: CommandOptions) -> Result<(), EdenError> {
    let config_path_buf = resolve_config_path(config_path)?;
    let config_path = config_path_buf.as_path();
    let loaded = load_config_with_context(config_path, options.strict)?;
    let ui = UiContext::from_env(options.json);
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

    if options.json {
        print_doctor_json(&findings)?;
    } else {
        print_doctor_text(&ui, &findings);
    }

    if options.strict && !findings.is_empty() {
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
    config_path: &std::path::Path,
    config: &Config,
    config_dir: &std::path::Path,
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
    config_path: &std::path::Path,
    config: &Config,
    config_dir: &std::path::Path,
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

fn print_doctor_text(ui: &UiContext, findings: &[DoctorFinding]) {
    if findings.is_empty() {
        println!(
            "{}  {} no issues detected",
            ui.action_prefix("Doctor"),
            ui.status_symbol(StatusSymbol::Success)
        );
        return;
    }

    let issue_label = if findings.len() == 1 {
        "issue"
    } else {
        "issues"
    };
    println!(
        "{}  {} {issue_label} detected",
        ui.action_prefix("Doctor"),
        findings.len()
    );
    println!();

    if findings.len() > 3 {
        let mut table = ui.table(&["Sev", "Code", "Skill"]);
        if let Some(column) = table.column_mut(0) {
            column.set_constraint(ColumnConstraint::UpperBoundary(Width::Fixed(5)));
        }
        for finding in findings {
            table.add_row(vec![
                doctor_severity_cell(&finding.severity),
                finding.code.clone(),
                finding.skill_id.clone(),
            ]);
        }
        println!("{table}");
        println!();
    }

    for (index, finding) in findings.iter().enumerate() {
        println!(
            "  {} [{}] {}",
            doctor_severity_symbol(ui, &finding.severity),
            finding.code,
            finding.skill_id
        );
        println!("    {}", doctor_message_with_styled_path(ui, finding));
        println!(
            "    {} {}",
            doctor_remediation_prefix(ui),
            finding.remediation
        );
        if index + 1 < findings.len() {
            println!();
        }
    }
}

fn doctor_severity_symbol(ui: &UiContext, severity: &str) -> String {
    match severity {
        "warning" => ui.status_symbol(StatusSymbol::Warning),
        _ => ui.status_symbol(StatusSymbol::Failure),
    }
}

fn doctor_severity_cell(severity: &str) -> String {
    match severity {
        "warning" => "warn".to_string(),
        _ => "error".to_string(),
    }
}

fn doctor_remediation_prefix(ui: &UiContext) -> String {
    let arrow = "→";
    if ui.colors_enabled() {
        arrow.dimmed().to_string()
    } else {
        arrow.to_string()
    }
}

fn doctor_message_with_styled_path(ui: &UiContext, finding: &DoctorFinding) -> String {
    if finding.target_path.is_empty() {
        return finding.message.clone();
    }
    let styled_path = ui.styled_path(&finding.target_path);
    let abbreviated_target = crate::ui::abbreviate_home_path(&finding.target_path);
    finding
        .message
        .replace(&finding.target_path, &styled_path)
        .replace(&abbreviated_target, &styled_path)
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
