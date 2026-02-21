use std::fs;
use std::io::ErrorKind;
use std::io::Read;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::config::{Config, InstallMode};
use crate::error::EdenError;
use crate::paths::{normalize_lexical, resolve_path_string, resolve_target_path};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    Create,
    Update,
    Noop,
    Conflict,
    Remove,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlanItem {
    pub skill_id: String,
    pub source_path: String,
    pub target_path: String,
    pub install_mode: InstallMode,
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
            let target_root = resolve_target_path(target, config_dir)?;
            let target_path = normalize_lexical(&target_root.join(&skill.id));
            let (action, reasons) =
                determine_action(skill.install.mode, &target_path, &source_path)
                    .map_err(EdenError::Io)?;

            items.push(PlanItem {
                skill_id: skill.id.clone(),
                source_path: source_path.display().to_string(),
                target_path: target_path.display().to_string(),
                install_mode: skill.install.mode,
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
    if !source_path.exists() {
        return Ok((
            Action::Conflict,
            vec!["source path does not exist".to_string()],
        ));
    }

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
            let expected = normalize_lexical(source_path);
            let current_cmp = fs::canonicalize(&current).unwrap_or(current);
            let expected_cmp = fs::canonicalize(&expected).unwrap_or(expected);
            if current_cmp == expected_cmp {
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

            match copy_content_equal(source_path, target_path) {
                Ok(true) => Ok((
                    Action::Noop,
                    vec!["target content matches source".to_string()],
                )),
                Ok(false) => Ok((
                    Action::Update,
                    vec!["target content differs from source".to_string()],
                )),
                Err(err) => Ok((
                    Action::Conflict,
                    vec![format!(
                        "copy comparison failed: {}",
                        copy_compare_error_cause(&err)
                    )],
                )),
            }
        }
    }
}

#[derive(Debug)]
enum CopyCompareError {
    SymlinkInTree,
    Io(ErrorKind),
}

fn copy_compare_error_cause(err: &CopyCompareError) -> &'static str {
    match err {
        CopyCompareError::SymlinkInTree => "symlink in tree",
        CopyCompareError::Io(ErrorKind::PermissionDenied) => "permission denied",
        CopyCompareError::Io(ErrorKind::NotFound) => "not found",
        CopyCompareError::Io(_) => "io error",
    }
}

fn copy_content_equal(source: &Path, target: &Path) -> Result<bool, CopyCompareError> {
    // Copy-mode comparisons must not follow symlinks (both for safety and for determinism).
    let source_meta = fs::symlink_metadata(source).map_err(|e| CopyCompareError::Io(e.kind()))?;
    let target_meta = fs::symlink_metadata(target).map_err(|e| CopyCompareError::Io(e.kind()))?;
    if source_meta.file_type().is_symlink() || target_meta.file_type().is_symlink() {
        return Err(CopyCompareError::SymlinkInTree);
    }

    if source_meta.is_file() != target_meta.is_file()
        || source_meta.is_dir() != target_meta.is_dir()
    {
        return Ok(false);
    }

    if source_meta.is_file() {
        if source_meta.len() != target_meta.len() {
            return Ok(false);
        }
        return file_content_equal_streaming(source, target);
    }

    let source_entries = read_dir_entry_names_no_symlink(source)?;
    let target_entries = read_dir_entry_names_no_symlink(target)?;

    if source_entries != target_entries {
        return Ok(false);
    }

    for name in source_entries {
        let source_child = source.join(&name);
        let target_child = target.join(&name);

        if !copy_content_equal(&source_child, &target_child)? {
            return Ok(false);
        }
    }

    Ok(true)
}

fn read_dir_entry_names_no_symlink(
    path: &Path,
) -> Result<Vec<std::ffi::OsString>, CopyCompareError> {
    let mut names = Vec::new();
    for entry in fs::read_dir(path).map_err(|e| CopyCompareError::Io(e.kind()))? {
        let entry = entry.map_err(|e| CopyCompareError::Io(e.kind()))?;
        let file_type = entry
            .file_type()
            .map_err(|e| CopyCompareError::Io(e.kind()))?;
        if file_type.is_symlink() {
            return Err(CopyCompareError::SymlinkInTree);
        }
        names.push(entry.file_name());
    }
    names.sort();
    Ok(names)
}

fn file_content_equal_streaming(source: &Path, target: &Path) -> Result<bool, CopyCompareError> {
    let mut source_file = fs::File::open(source).map_err(|e| CopyCompareError::Io(e.kind()))?;
    let mut target_file = fs::File::open(target).map_err(|e| CopyCompareError::Io(e.kind()))?;

    let mut source_buf = vec![0u8; 64 * 1024];
    let mut target_buf = vec![0u8; 64 * 1024];

    loop {
        let n1 = source_file
            .read(&mut source_buf)
            .map_err(|e| CopyCompareError::Io(e.kind()))?;
        let n2 = target_file
            .read(&mut target_buf)
            .map_err(|e| CopyCompareError::Io(e.kind()))?;
        if n1 != n2 {
            return Ok(false);
        }
        if n1 == 0 {
            return Ok(true);
        }
        if source_buf[..n1] != target_buf[..n2] {
            return Ok(false);
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
