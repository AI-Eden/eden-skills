//! Target adapter abstraction for local and Docker environments.
//!
//! [`TargetAdapter`] defines the install/uninstall/health-check interface.
//! [`LocalAdapter`] operates on the host filesystem (symlink or copy).
//! [`DockerAdapter`] proxies operations into a running Docker container
//! via `docker exec` / `docker cp`. Both are `Send + Sync` to enable
//! concurrent spawning from the reactor.

mod docker;
mod local;
mod manifest;

use std::path::Path;

use async_trait::async_trait;

use crate::config::InstallMode;
pub use crate::error::AdapterError;

pub use self::docker::DockerAdapter;
pub use self::local::LocalAdapter;
pub use self::manifest::{
    read_managed_manifest, write_managed_manifest, ManagedManifestReadResult,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdapterEnvironment {
    Local,
    Docker { container_name: String },
}

pub fn parse_environment(environment: &str) -> Result<AdapterEnvironment, AdapterError> {
    if environment == "local" {
        return Ok(AdapterEnvironment::Local);
    }

    if let Some(container_name) = environment.strip_prefix("docker:") {
        if container_name.trim().is_empty() {
            return Err(AdapterError::Config {
                detail: "invalid environment `docker:`: container name must not be empty"
                    .to_string(),
            });
        }
        return Ok(AdapterEnvironment::Docker {
            container_name: container_name.to_string(),
        });
    }

    Err(AdapterError::Config {
        detail: format!(
            "invalid environment `{environment}`: expected `local` or `docker:<container>`"
        ),
    })
}

pub fn create_adapter(environment: &str) -> Result<Box<dyn TargetAdapter>, AdapterError> {
    match parse_environment(environment)? {
        AdapterEnvironment::Local => Ok(Box::new(LocalAdapter::new())),
        AdapterEnvironment::Docker { container_name } => {
            Ok(Box::new(DockerAdapter::new(container_name)?))
        }
    }
}

/// Abstract interface for skill installation targets.
///
/// `Send + Sync` bounds are required so the reactor can spawn adapter
/// calls across threads.
#[async_trait]
pub trait TargetAdapter: Send + Sync {
    fn adapter_type(&self) -> &str;

    async fn health_check(&self) -> Result<(), AdapterError>;

    async fn path_exists(&self, path: &Path) -> Result<bool, AdapterError>;

    async fn install(
        &self,
        source: &Path,
        target: &Path,
        mode: InstallMode,
    ) -> Result<(), AdapterError>;

    async fn uninstall(&self, target: &Path) -> Result<(), AdapterError>;

    async fn exec(&self, cmd: &str) -> Result<String, AdapterError>;
}

pub(crate) fn shell_escape_double_quoted(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\"', "\\\"")
}
