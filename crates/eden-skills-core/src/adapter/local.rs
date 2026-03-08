//! Local filesystem adapter for symlink and copy install modes.

use std::path::Path;

use async_trait::async_trait;
use tokio::process::Command;

use crate::config::InstallMode;
use crate::error::AdapterError;

use super::TargetAdapter;

/// Adapter that installs skills on the host filesystem via symlink or copy.
#[derive(Debug, Clone, Copy, Default)]
pub struct LocalAdapter;

impl LocalAdapter {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl TargetAdapter for LocalAdapter {
    fn adapter_type(&self) -> &str {
        "local"
    }

    async fn health_check(&self) -> Result<(), AdapterError> {
        Ok(())
    }

    async fn path_exists(&self, path: &Path) -> Result<bool, AdapterError> {
        match tokio::fs::symlink_metadata(path).await {
            Ok(_) => Ok(true),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(err) => Err(AdapterError::Io(err)),
        }
    }

    async fn install(
        &self,
        source: &Path,
        target: &Path,
        mode: InstallMode,
    ) -> Result<(), AdapterError> {
        let source_metadata = tokio::fs::symlink_metadata(source).await?;
        ensure_parent_dir(target).await?;
        remove_existing_path(target).await?;

        match mode {
            InstallMode::Symlink => create_symlink(source, target, source_metadata.is_dir())?,
            InstallMode::Copy => copy_recursively(source, target).await?,
        }

        Ok(())
    }

    async fn uninstall(&self, target: &Path) -> Result<(), AdapterError> {
        remove_existing_path(target).await
    }

    async fn exec(&self, cmd: &str) -> Result<String, AdapterError> {
        #[cfg(unix)]
        let mut command = {
            let mut command = Command::new("sh");
            command.arg("-c").arg(cmd);
            command
        };

        #[cfg(windows)]
        let mut command = {
            let mut command = Command::new("cmd");
            command.arg("/C").arg(cmd);
            command
        };

        let output = command.output().await?;
        if output.status.success() {
            return Ok(String::from_utf8_lossy(&output.stdout).to_string());
        }

        Err(AdapterError::Runtime {
            detail: format!(
                "local command failed: status={} stderr=`{}`",
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        })
    }
}

async fn ensure_parent_dir(path: &Path) -> Result<(), AdapterError> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    Ok(())
}

async fn remove_existing_path(path: &Path) -> Result<(), AdapterError> {
    match tokio::fs::symlink_metadata(path).await {
        Ok(metadata) => {
            if path_is_symlink_or_junction(path, &metadata) {
                remove_symlink_or_junction(path).await?;
                return Ok(());
            }

            if metadata.is_dir() {
                tokio::fs::remove_dir_all(path).await?;
                return Ok(());
            }

            tokio::fs::remove_file(path).await?;
            Ok(())
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(AdapterError::Io(err)),
    }
}

fn path_is_symlink_or_junction(path: &Path, metadata: &std::fs::Metadata) -> bool {
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

#[cfg(not(windows))]
async fn remove_symlink(path: &Path) -> Result<(), AdapterError> {
    tokio::fs::remove_file(path).await?;
    Ok(())
}

#[cfg(not(windows))]
async fn remove_symlink_or_junction(path: &Path) -> Result<(), AdapterError> {
    remove_symlink(path).await
}

#[cfg(windows)]
async fn remove_symlink(path: &Path) -> Result<(), AdapterError> {
    match tokio::fs::remove_file(path).await {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => {
            tokio::fs::remove_dir(path).await?;
            Ok(())
        }
        Err(err) => Err(AdapterError::Io(err)),
    }
}

#[cfg(windows)]
async fn remove_symlink_or_junction(path: &Path) -> Result<(), AdapterError> {
    if junction::exists(path).unwrap_or(false) {
        junction::delete(path).map_err(|err| AdapterError::Runtime {
            detail: format!("failed to delete junction `{}`: {err}", path.display()),
        })?;
        match tokio::fs::remove_dir(path).await {
            Ok(()) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => return Err(AdapterError::Io(err)),
        }
        return Ok(());
    }

    remove_symlink(path).await
}

fn create_symlink(source: &Path, target: &Path, source_is_dir: bool) -> Result<(), AdapterError> {
    #[cfg(unix)]
    {
        let _ = source_is_dir;
        std::os::unix::fs::symlink(source, target)?;
        Ok(())
    }

    #[cfg(windows)]
    {
        if source_is_dir {
            match std::os::windows::fs::symlink_dir(source, target) {
                Ok(()) => {}
                Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => {
                    junction::create(source, target).map_err(|junction_err| {
                        map_windows_symlink_fallback_error(err, junction_err, source, target)
                    })?;
                }
                Err(err) => return Err(map_windows_symlink_error(err, source, target)),
            }
        } else {
            std::os::windows::fs::symlink_file(source, target)
                .map_err(|err| map_windows_symlink_error(err, source, target))?;
        }
        Ok(())
    }
}

#[cfg(windows)]
fn map_windows_symlink_fallback_error(
    symlink_err: std::io::Error,
    junction_err: std::io::Error,
    source: &Path,
    target: &Path,
) -> AdapterError {
    AdapterError::Runtime {
        detail: format!(
            "failed to create symlink `{}` -> `{}`: {}. Enable Developer Mode or run as Administrator if symlink privileges are unavailable; otherwise verify write permissions for the target path. junction fallback failed: {}",
            target.display(),
            source.display(),
            symlink_err,
            junction_err
        ),
    }
}

#[cfg(windows)]
fn map_windows_symlink_error(err: std::io::Error, source: &Path, target: &Path) -> AdapterError {
    if err.kind() == std::io::ErrorKind::PermissionDenied {
        return AdapterError::Runtime {
            detail: format!(
                "failed to create symlink `{}` -> `{}`: {}. Enable Developer Mode or run as Administrator.",
                target.display(),
                source.display(),
                err
            ),
        };
    }
    AdapterError::Io(err)
}

async fn copy_recursively(source: &Path, target: &Path) -> Result<(), AdapterError> {
    let source = source.to_path_buf();
    let target = target.to_path_buf();
    tokio::task::spawn_blocking(move || copy_recursively_blocking(&source, &target))
        .await
        .map_err(|err| AdapterError::Runtime {
            detail: format!("local copy task failed to join: {err}"),
        })?
}

fn copy_recursively_blocking(source: &Path, target: &Path) -> Result<(), AdapterError> {
    let source_metadata = std::fs::symlink_metadata(source)?;
    if source_metadata.is_file() {
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(source, target)?;
        return Ok(());
    }

    std::fs::create_dir_all(target)?;
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let source_child = entry.path();
        let target_child = target.join(entry.file_name());
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            copy_recursively_blocking(&source_child, &target_child)?;
        } else {
            std::fs::copy(&source_child, &target_child)?;
        }
    }

    Ok(())
}
