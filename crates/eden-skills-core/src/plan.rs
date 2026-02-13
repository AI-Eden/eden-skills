use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::config::{Config, InstallMode};
use crate::error::EdenError;
use crate::paths::{normalize_lexical, resolve_path_string, resolve_target_path};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Create,
    Update,
    Noop,
    Conflict,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanItem {
    pub skill_id: String,
    pub source_path: String,
    pub target_path: String,
    pub install_mode: String,
    pub action: Action,
    pub reasons: Vec<String>,
}

pub fn build_plan(config: &Config, config_dir: &Path) -> Result<Vec<PlanItem>, EdenError> {
    let storage_root = resolve_path_string(&config.storage_root, config_dir)?;
    let mut items = Vec::new();

    for skill in &config.skills {
        let source_repo_root = storage_root.join(&skill.id);
        let source_path = normalize_lexical(&source_repo_root.join(&skill.source.subpath));

        for target in &skill.targets {
            let target_path = resolve_target_path(target, config_dir)?;
            let (action, reasons) =
                determine_action(skill.install.mode, &target_path, &source_path)
                    .map_err(EdenError::Io)?;

            items.push(PlanItem {
                skill_id: skill.id.clone(),
                source_path: source_path.display().to_string(),
                target_path: target_path.display().to_string(),
                install_mode: skill.install.mode.as_str().to_string(),
                action,
                reasons,
            });
        }
    }

    Ok(items)
}

fn determine_action(
    install_mode: InstallMode,
    target_path: &Path,
    source_path: &Path,
) -> Result<(Action, Vec<String>), std::io::Error> {
    let metadata = match fs::symlink_metadata(target_path) {
        Ok(metadata) => metadata,
        Err(err) if err.kind() == ErrorKind::NotFound => {
            return Ok((
                Action::Create,
                vec!["target path does not exist".to_string()],
            ));
        }
        Err(err) => return Err(err),
    };

    match install_mode {
        InstallMode::Symlink => {
            if !metadata.file_type().is_symlink() {
                return Ok((
                    Action::Conflict,
                    vec!["target exists but is not a symlink".to_string()],
                ));
            }

            let current = read_symlink_target(target_path)?;
            if current == normalize_lexical(source_path) {
                Ok((
                    Action::Noop,
                    vec!["target already points to source".to_string()],
                ))
            } else {
                Ok((
                    Action::Update,
                    vec!["symlink points to a different source".to_string()],
                ))
            }
        }
        InstallMode::Copy => {
            if metadata.file_type().is_symlink() {
                return Ok((
                    Action::Conflict,
                    vec!["target is a symlink but install mode is copy".to_string()],
                ));
            }
            Ok((
                Action::Noop,
                vec!["target exists; copy diff check not implemented yet".to_string()],
            ))
        }
    }
}

fn read_symlink_target(target_path: &Path) -> Result<PathBuf, std::io::Error> {
    let raw_target = fs::read_link(target_path)?;
    let resolved = if raw_target.is_absolute() {
        raw_target
    } else {
        let parent = target_path.parent().unwrap_or(Path::new("."));
        parent.join(raw_target)
    };
    Ok(normalize_lexical(&resolved))
}
