//! `.eden-managed` manifest I/O for ownership tracking.

use std::path::{Path, PathBuf};

use crate::error::AdapterError;
use crate::managed::{ManagedManifest, MANAGED_MANIFEST_FILE};
use crate::paths::normalize_lexical;

use super::{parse_environment, AdapterEnvironment};

#[derive(Debug, Clone)]
pub struct ManagedManifestReadResult {
    pub manifest: ManagedManifest,
    pub warning: Option<String>,
}

pub async fn read_managed_manifest(
    environment: &str,
    agent_dir: &Path,
) -> Result<ManagedManifestReadResult, AdapterError> {
    match parse_environment(environment)? {
        AdapterEnvironment::Local => read_local_managed_manifest(agent_dir).await,
        AdapterEnvironment::Docker { container_name } => {
            super::docker::DockerAdapter::new(container_name)?
                .read_managed_manifest(agent_dir)
                .await
        }
    }
}

pub async fn write_managed_manifest(
    environment: &str,
    agent_dir: &Path,
    manifest: &ManagedManifest,
) -> Result<(), AdapterError> {
    match parse_environment(environment)? {
        AdapterEnvironment::Local => write_local_managed_manifest(agent_dir, manifest).await,
        AdapterEnvironment::Docker { container_name } => {
            super::docker::DockerAdapter::new(container_name)?
                .write_managed_manifest(agent_dir, manifest)
                .await
        }
    }
}

pub(super) fn managed_manifest_path(agent_dir: &Path) -> PathBuf {
    normalize_lexical(&agent_dir.join(MANAGED_MANIFEST_FILE))
}

pub(super) fn parse_managed_manifest(
    manifest_path: &Path,
    raw: &str,
    location_label: &str,
) -> Result<ManagedManifestReadResult, AdapterError> {
    match ManagedManifest::parse(raw) {
        Ok(manifest) => Ok(ManagedManifestReadResult {
            manifest,
            warning: None,
        }),
        Err(err) => Ok(ManagedManifestReadResult {
            manifest: ManagedManifest::default(),
            warning: Some(format!(
                "Ignoring invalid `.eden-managed` manifest at `{}` for {location_label}: {err}",
                manifest_path.display()
            )),
        }),
    }
}

pub(super) async fn read_local_managed_manifest(
    agent_dir: &Path,
) -> Result<ManagedManifestReadResult, AdapterError> {
    let manifest_path = managed_manifest_path(agent_dir);
    match tokio::fs::read_to_string(&manifest_path).await {
        Ok(raw) => parse_managed_manifest(&manifest_path, &raw, "local target"),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(ManagedManifestReadResult {
            manifest: ManagedManifest::default(),
            warning: None,
        }),
        Err(err) => Err(AdapterError::Io(err)),
    }
}

pub(super) async fn write_local_managed_manifest(
    agent_dir: &Path,
    manifest: &ManagedManifest,
) -> Result<(), AdapterError> {
    tokio::fs::create_dir_all(agent_dir).await?;
    let manifest_path = managed_manifest_path(agent_dir);
    let encoded = manifest
        .to_pretty_json()
        .map_err(|err| AdapterError::Runtime {
            detail: format!("failed to serialize managed manifest: {err}"),
        })?;
    tokio::fs::write(manifest_path, format!("{encoded}\n")).await?;
    Ok(())
}

pub(super) fn managed_manifest_temp_path() -> PathBuf {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!(
        "eden-skills-managed-{}-{unique}.json",
        std::process::id()
    ))
}
