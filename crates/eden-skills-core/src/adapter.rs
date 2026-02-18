use std::ffi::OsString;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use tokio::process::Command;

use crate::config::InstallMode;
pub use crate::error::AdapterError;

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

    async fn exec(&self, cmd: &str) -> Result<String, AdapterError>;
}

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

#[derive(Debug, Clone)]
pub struct DockerAdapter {
    container_name: String,
    docker_bin: PathBuf,
}

impl DockerAdapter {
    pub fn new(container_name: impl Into<String>) -> Result<Self, AdapterError> {
        Self::with_binary(container_name, Path::new("docker"))
    }

    pub fn with_binary(
        container_name: impl Into<String>,
        docker_bin: impl AsRef<Path>,
    ) -> Result<Self, AdapterError> {
        let container_name = container_name.into();
        if container_name.trim().is_empty() {
            return Err(AdapterError::Config {
                detail: "docker container name must not be empty".to_string(),
            });
        }

        let docker_bin = docker_bin.as_ref().to_path_buf();
        ensure_docker_cli_available(&docker_bin)?;

        Ok(Self {
            container_name,
            docker_bin,
        })
    }

    async fn run_docker<I>(
        &self,
        args: I,
        context: &str,
    ) -> Result<std::process::Output, AdapterError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let mut command = Command::new(&self.docker_bin);
        command.args(args);
        let output = command.output().await.map_err(|err| {
            if err.kind() == std::io::ErrorKind::NotFound {
                AdapterError::Config {
                    detail:
                        "Docker CLI not found. Install Docker or ensure `docker` is in your PATH."
                            .to_string(),
                }
            } else {
                AdapterError::Runtime {
                    detail: format!(
                        "docker command failed to start while trying to {context}: {err}"
                    ),
                }
            }
        })?;

        Ok(output)
    }
}

#[async_trait]
impl TargetAdapter for DockerAdapter {
    fn adapter_type(&self) -> &str {
        "docker"
    }

    async fn health_check(&self) -> Result<(), AdapterError> {
        let output = self
            .run_docker(
                vec![
                    OsString::from("inspect"),
                    OsString::from("--format"),
                    OsString::from("{{.State.Running}}"),
                    OsString::from(self.container_name.clone()),
                ],
                "check container health",
            )
            .await?;

        if !output.status.success() {
            return Err(AdapterError::Runtime {
                detail: format!(
                    "docker inspect failed for container `{}`: status={} stderr=`{}`",
                    self.container_name,
                    output.status,
                    String::from_utf8_lossy(&output.stderr).trim()
                ),
            });
        }

        let running = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if running == "true" {
            return Ok(());
        }

        Err(AdapterError::Runtime {
            detail: format!(
                "Container `{}` is not running. Start it with `docker start {}`.",
                self.container_name, self.container_name
            ),
        })
    }

    async fn path_exists(&self, path: &Path) -> Result<bool, AdapterError> {
        let command = format!(
            "test -e \"{}\"",
            shell_escape_double_quoted(&path.display().to_string())
        );
        let output = self
            .run_docker(
                vec![
                    OsString::from("exec"),
                    OsString::from(self.container_name.clone()),
                    OsString::from("sh"),
                    OsString::from("-c"),
                    OsString::from(command),
                ],
                "check path existence in container",
            )
            .await?;

        if output.status.success() {
            return Ok(true);
        }

        if output.status.code() == Some(1) {
            return Ok(false);
        }

        Err(AdapterError::Runtime {
            detail: format!(
                "docker exec path check failed in container `{}`: status={} stderr=`{}`",
                self.container_name,
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        })
    }

    async fn install(
        &self,
        source: &Path,
        target: &Path,
        _mode: InstallMode,
    ) -> Result<(), AdapterError> {
        self.health_check().await?;
        let source_metadata = tokio::fs::symlink_metadata(source).await?;

        let source_arg = if source_metadata.is_dir() {
            format!("{}/.", source.display())
        } else {
            source.display().to_string()
        };
        let target_arg = format!("{}:{}", self.container_name, target.display());

        let output = self
            .run_docker(
                vec![
                    OsString::from("cp"),
                    OsString::from(source_arg),
                    OsString::from(target_arg),
                ],
                "copy files into container",
            )
            .await?;
        if !output.status.success() {
            return Err(AdapterError::Runtime {
                detail: format!(
                    "docker cp failed for container `{}`: status={} stderr=`{}`",
                    self.container_name,
                    output.status,
                    String::from_utf8_lossy(&output.stderr).trim()
                ),
            });
        }

        if !self.path_exists(target).await? {
            return Err(AdapterError::Runtime {
                detail: format!(
                    "docker install verification failed: target `{}` does not exist in container `{}`",
                    target.display(),
                    self.container_name
                ),
            });
        }

        Ok(())
    }

    async fn exec(&self, cmd: &str) -> Result<String, AdapterError> {
        self.health_check().await?;
        let output = self
            .run_docker(
                vec![
                    OsString::from("exec"),
                    OsString::from(self.container_name.clone()),
                    OsString::from("sh"),
                    OsString::from("-c"),
                    OsString::from(cmd),
                ],
                "execute command in container",
            )
            .await?;

        if output.status.success() {
            return Ok(String::from_utf8_lossy(&output.stdout).to_string());
        }

        Err(AdapterError::Runtime {
            detail: format!(
                "docker exec failed in container `{}`: status={} stderr=`{}` stdout=`{}`",
                self.container_name,
                output.status,
                String::from_utf8_lossy(&output.stderr).trim(),
                String::from_utf8_lossy(&output.stdout).trim()
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
            if metadata.file_type().is_symlink() {
                remove_symlink(path).await?;
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

#[cfg(not(windows))]
async fn remove_symlink(path: &Path) -> Result<(), AdapterError> {
    tokio::fs::remove_file(path).await?;
    Ok(())
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
            std::os::windows::fs::symlink_dir(source, target)?;
        } else {
            std::os::windows::fs::symlink_file(source, target)?;
        }
        Ok(())
    }
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

fn shell_escape_double_quoted(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\"', "\\\"")
}

fn ensure_docker_cli_available(docker_bin: &Path) -> Result<(), AdapterError> {
    match std::process::Command::new(docker_bin)
        .arg("--version")
        .output()
    {
        Ok(output) if output.status.success() => Ok(()),
        Ok(output) => Err(AdapterError::Config {
            detail: format!(
                "Docker CLI not found. Install Docker or ensure `docker` is in your PATH. status={} stderr=`{}`",
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        }),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Err(AdapterError::Config {
            detail: "Docker CLI not found. Install Docker or ensure `docker` is in your PATH."
                .to_string(),
        }),
        Err(err) => Err(AdapterError::Config {
            detail: format!(
                "Docker CLI not found. Install Docker or ensure `docker` is in your PATH. error: {err}"
            ),
        }),
    }
}
