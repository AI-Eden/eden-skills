use std::fs;
use std::path::{Path, PathBuf};

use crate::config::{Config, InstallMode};
use crate::error::EdenError;
use crate::paths::{normalize_lexical, resolve_path_string, resolve_target_path};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifyIssue {
    pub skill_id: String,
    pub target_path: String,
    pub check: String,
    pub message: String,
}

pub fn verify_config_state(
    config: &Config,
    config_dir: &Path,
) -> Result<Vec<VerifyIssue>, EdenError> {
    let storage_root = resolve_path_string(&config.storage_root, config_dir)?;
    let mut issues = Vec::new();

    for skill in &config.skills {
        if !skill.verify.enabled {
            continue;
        }

        let source_path =
            normalize_lexical(&storage_root.join(&skill.id).join(&skill.source.subpath));
        for target in &skill.targets {
            let target_root = resolve_target_path(target, config_dir)?;
            let target_path = normalize_lexical(&target_root.join(&skill.id));

            for check in &skill.verify.checks {
                run_check(
                    check,
                    skill.id.as_str(),
                    skill.install.mode,
                    &source_path,
                    &target_path,
                    &mut issues,
                )?;
            }
        }
    }

    Ok(issues)
}

fn run_check(
    check: &str,
    skill_id: &str,
    install_mode: InstallMode,
    source_path: &Path,
    target_path: &Path,
    issues: &mut Vec<VerifyIssue>,
) -> Result<(), EdenError> {
    match check {
        "path-exists" => {
            if fs::symlink_metadata(target_path).is_err() {
                issues.push(issue(
                    skill_id,
                    target_path,
                    check,
                    "target path does not exist".to_string(),
                ));
            }
        }
        "is-symlink" => match fs::symlink_metadata(target_path) {
            Ok(metadata) => {
                if !metadata.file_type().is_symlink() {
                    issues.push(issue(
                        skill_id,
                        target_path,
                        check,
                        "target exists but is not a symlink".to_string(),
                    ));
                }
            }
            Err(_) => issues.push(issue(
                skill_id,
                target_path,
                check,
                "target path does not exist".to_string(),
            )),
        },
        "target-resolves" => match fs::read_link(target_path) {
            Ok(current_target) => {
                let resolved = resolve_symlink_target(target_path, &current_target);
                if resolved != normalize_lexical(source_path) {
                    issues.push(issue(
                        skill_id,
                        target_path,
                        check,
                        format!(
                            "symlink resolves to `{}` but expected `{}`",
                            resolved.display(),
                            normalize_lexical(source_path).display()
                        ),
                    ));
                }
            }
            Err(_) => issues.push(issue(
                skill_id,
                target_path,
                check,
                "target symlink is missing or unreadable".to_string(),
            )),
        },
        "content-present" => {
            if fs::symlink_metadata(target_path).is_err() {
                issues.push(issue(
                    skill_id,
                    target_path,
                    check,
                    "target content is missing".to_string(),
                ));
            } else if matches!(install_mode, InstallMode::Symlink) {
                issues.push(issue(
                    skill_id,
                    target_path,
                    check,
                    "content-present check is typically for copy mode".to_string(),
                ));
            }
        }
        unknown => {
            return Err(EdenError::Validation(format!(
                "verify.checks: unsupported check `{unknown}`"
            )));
        }
    }

    Ok(())
}

fn resolve_symlink_target(target_path: &Path, link_target: &Path) -> PathBuf {
    if link_target.is_absolute() {
        normalize_lexical(link_target)
    } else {
        let parent = target_path.parent().unwrap_or(Path::new("."));
        normalize_lexical(&parent.join(link_target))
    }
}

fn issue(skill_id: &str, target_path: &Path, check: &str, message: String) -> VerifyIssue {
    VerifyIssue {
        skill_id: skill_id.to_string(),
        target_path: target_path.display().to_string(),
        check: check.to_string(),
        message,
    }
}
