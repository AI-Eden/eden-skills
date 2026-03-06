//! Reconciliation plan computation.
//!
//! Compares declared skill targets against the current filesystem state
//! to produce a list of [`PlanItem`]s, each carrying an [`Action`]:
//! `Create`, `Update`, `Noop`, `Conflict`, or `Remove`.

use std::fs;
use std::io::ErrorKind;
use std::io::Read;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::config::{Config, InstallMode};
use crate::error::EdenError;
use crate::paths::{normalize_lexical, resolve_path_string, resolve_target_path};
use crate::source::resolve_skill_source_path;

/// The reconciliation action determined for a single skill target.
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

/// Build a reconciliation plan by comparing config targets to filesystem state.
///
/// For each skill × target pair, determines whether the target needs to
/// be created, updated, left alone (noop), or flagged as a conflict.
///
/// # Errors
///
/// Returns [`EdenError`] on path resolution or I/O failures during
/// filesystem inspection.
pub fn build_plan(config: &Config, config_dir: &Path) -> Result<Vec<PlanItem>, EdenError> {
    let storage_root = resolve_path_string(&config.storage_root, config_dir)?;
    let mut items = Vec::new();

    for skill in &config.skills {
        let source_path = resolve_skill_source_path(&storage_root, skill);

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
            if !is_symlink_or_junction(&metadata, target_path) {
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
            if is_symlink_or_junction(&metadata, target_path) {
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

fn is_symlink_or_junction(metadata: &fs::Metadata, path: &Path) -> bool {
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

fn file_type_is_symlink_or_junction(file_type: &fs::FileType, path: &Path) -> bool {
    #[cfg(windows)]
    {
        file_type.is_symlink() || junction::exists(path).unwrap_or(false)
    }

    #[cfg(not(windows))]
    {
        let _ = path;
        file_type.is_symlink()
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
    if is_symlink_or_junction(&source_meta, source) || is_symlink_or_junction(&target_meta, target)
    {
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
        if file_metadata_matches_fast_path(&source_meta, &target_meta) {
            return Ok(true);
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

fn file_metadata_matches_fast_path(source_meta: &fs::Metadata, target_meta: &fs::Metadata) -> bool {
    match (source_meta.modified(), target_meta.modified()) {
        (Ok(source_modified), Ok(target_modified)) => source_modified == target_modified,
        _ => false,
    }
}

fn read_dir_entry_names_no_symlink(
    path: &Path,
) -> Result<Vec<std::ffi::OsString>, CopyCompareError> {
    let mut names = Vec::new();
    for entry in fs::read_dir(path).map_err(|e| CopyCompareError::Io(e.kind()))? {
        let entry = entry.map_err(|e| CopyCompareError::Io(e.kind()))?;
        let entry_path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|e| CopyCompareError::Io(e.kind()))?;
        if file_type_is_symlink_or_junction(&file_type, &entry_path) {
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
    #[cfg(windows)]
    let raw_target = if junction::exists(target_path).unwrap_or(false) {
        junction::get_target(target_path)?
    } else {
        fs::read_link(target_path)?
    };

    #[cfg(not(windows))]
    let raw_target = fs::read_link(target_path)?;

    let resolved = if raw_target.is_absolute() {
        raw_target
    } else {
        let parent = target_path.parent().unwrap_or(Path::new("."));
        parent.join(raw_target)
    };
    Ok(normalize_lexical(&resolved))
}
