//! Target adapter abstraction for local and Docker environments.
//!
//! [`TargetAdapter`] defines the install/uninstall/health-check interface.
//! [`LocalAdapter`] operates on the host filesystem (symlink or copy).
//! [`DockerAdapter`] proxies operations into a running Docker container
//! via `docker exec` / `docker cp`. Both are `Send + Sync` to enable
//! concurrent spawning from the reactor.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use serde::Deserialize;
use tokio::process::Command;

use crate::config::{AgentKind, InstallMode, TargetConfig};
pub use crate::error::AdapterError;
use crate::paths::{default_agent_path, normalize_lexical};

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

/// Adapter that proxies skill operations into a Docker container via `docker cp` / `docker exec`.
#[derive(Debug, Clone)]
pub struct DockerAdapter {
    container_name: String,
    docker_bin: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
struct DockerMount {
    #[serde(rename = "Type")]
    mount_type: String,
    #[serde(rename = "Source")]
    source: PathBuf,
    #[serde(rename = "Destination")]
    destination: PathBuf,
    #[serde(rename = "RW", default)]
    writable: bool,
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

    pub fn container_name(&self) -> &str {
        &self.container_name
    }

    pub async fn container_home(&self) -> Result<PathBuf, AdapterError> {
        let output = self
            .run_docker(
                vec![
                    OsString::from("exec"),
                    OsString::from(self.container_name.clone()),
                    OsString::from("sh"),
                    OsString::from("-c"),
                    OsString::from("printf '%s' \"$HOME\""),
                ],
                "resolve container HOME",
            )
            .await?;
        if !output.status.success() {
            return Err(AdapterError::Runtime {
                detail: format!(
                    "docker exec failed while resolving HOME in container `{}`: status={} stderr=`{}` stdout=`{}`",
                    self.container_name,
                    output.status,
                    String::from_utf8_lossy(&output.stderr).trim(),
                    String::from_utf8_lossy(&output.stdout).trim()
                ),
            });
        }

        let home = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if home.is_empty() {
            return Err(AdapterError::Runtime {
                detail: format!(
                    "docker exec returned empty HOME for container `{}`",
                    self.container_name
                ),
            });
        }
        Ok(PathBuf::from(home))
    }

    pub async fn default_target_root_for_agent(
        &self,
        agent: &AgentKind,
    ) -> Result<PathBuf, AdapterError> {
        let relative = agent_relative_path(agent).ok_or_else(|| AdapterError::Config {
            detail: format!(
                "agent `{}` does not have a default target path for docker installs",
                agent.as_str()
            ),
        })?;
        let home = self.container_home().await?;
        Ok(normalize_lexical(&home.join(relative)))
    }

    async fn inspect_mounts(&self) -> Result<Vec<DockerMount>, AdapterError> {
        let output = self
            .run_docker(
                vec![
                    OsString::from("inspect"),
                    OsString::from("--format"),
                    OsString::from("{{json .Mounts}}"),
                    OsString::from(self.container_name.clone()),
                ],
                "inspect container mounts",
            )
            .await?;
        if !output.status.success() {
            return Err(AdapterError::Runtime {
                detail: format!(
                    "docker inspect mounts failed for container `{}`: status={} stderr=`{}` stdout=`{}`",
                    self.container_name,
                    output.status,
                    String::from_utf8_lossy(&output.stderr).trim(),
                    String::from_utf8_lossy(&output.stdout).trim()
                ),
            });
        }

        let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if matches!(raw.as_str(), "true" | "false" | "null") {
            return Ok(Vec::new());
        }
        serde_json::from_str::<Vec<DockerMount>>(&raw).map_err(|err| AdapterError::Runtime {
            detail: format!(
                "failed to parse docker mounts for container `{}`: {err}",
                self.container_name
            ),
        })
    }

    async fn mounted_host_path_for_path_with_writable_policy(
        &self,
        target: &Path,
        require_writable: bool,
    ) -> Result<Option<PathBuf>, AdapterError> {
        let target = normalize_lexical(target);
        for mount in self.inspect_mounts().await? {
            if mount.mount_type != "bind" {
                continue;
            }
            if require_writable && !mount.writable {
                continue;
            }
            let destination = normalize_lexical(&mount.destination);
            if !(target == destination || target.starts_with(&destination)) {
                continue;
            }

            let suffix = target.strip_prefix(&destination).unwrap_or(Path::new(""));
            let host_path = normalize_lexical(&mount.source.join(suffix));
            return Ok(Some(host_path));
        }
        Ok(None)
    }

    pub async fn mounted_host_path_for_path(
        &self,
        target: &Path,
    ) -> Result<Option<PathBuf>, AdapterError> {
        self.mounted_host_path_for_path_with_writable_policy(target, false)
            .await
    }

    pub async fn bind_mount_for_path(
        &self,
        target: &Path,
    ) -> Result<Option<PathBuf>, AdapterError> {
        self.mounted_host_path_for_path_with_writable_policy(target, true)
            .await
    }

    pub async fn detect_agents(&self) -> Result<Vec<TargetConfig>, AdapterError> {
        let detection_rules = AgentKind::all_non_custom()
            .iter()
            .filter(|agent| agent.is_auto_detect_eligible())
            .filter_map(|agent| {
                container_detection_root(agent).map(|subpath| (agent.clone(), subpath))
            })
            .collect::<Vec<_>>();
        let mut script = String::from("for d in");
        for (_, subpath) in &detection_rules {
            script.push_str(" \"");
            script.push_str(&shell_escape_double_quoted(subpath));
            script.push('"');
        }
        script.push_str("; do test -d \"$HOME/$d\" && echo \"$d\"; done");

        let output = self
            .run_docker(
                vec![
                    OsString::from("exec"),
                    OsString::from(self.container_name.clone()),
                    OsString::from("sh"),
                    OsString::from("-c"),
                    OsString::from(script),
                ],
                "detect installed agents in container",
            )
            .await?;
        if !output.status.success() {
            return Err(AdapterError::Runtime {
                detail: format!(
                    "docker exec agent detection failed in container `{}`: status={} stderr=`{}` stdout=`{}`",
                    self.container_name,
                    output.status,
                    String::from_utf8_lossy(&output.stderr).trim(),
                    String::from_utf8_lossy(&output.stdout).trim()
                ),
            });
        }

        let detected_stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let detected_roots = detected_stdout
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>();
        let mut targets = Vec::new();
        for (agent, subpath) in detection_rules {
            if detected_roots.iter().any(|detected| detected == &subpath) {
                targets.push(TargetConfig {
                    agent,
                    expected_path: None,
                    path: None,
                    environment: format!("docker:{}", self.container_name),
                });
            }
        }
        Ok(targets)
    }

    pub fn resolve_install_mode(
        mode: InstallMode,
        container_name: &str,
    ) -> (InstallMode, Option<String>) {
        if matches!(mode, InstallMode::Symlink) {
            return (
                InstallMode::Copy,
                Some(format!(
                    "docker target `{container_name}` does not support symlink mode; falling back to copy"
                )),
            );
        }
        (InstallMode::Copy, None)
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
        mode: InstallMode,
    ) -> Result<(), AdapterError> {
        self.health_check().await?;
        if let Some(host_target) = self.bind_mount_for_path(target).await? {
            return LocalAdapter::new()
                .install(source, &host_target, mode)
                .await;
        }

        let source_metadata = tokio::fs::symlink_metadata(source).await?;
        let (_effective_mode, warning) = Self::resolve_install_mode(mode, &self.container_name);
        if let Some(warning) = warning {
            eprintln!("warning: {warning}");
        }
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
                    "docker cp failed for container `{}` at target `{}`: status={} stderr=`{}`",
                    self.container_name,
                    target.display(),
                    output.status,
                    String::from_utf8_lossy(&output.stderr).trim()
                ),
            });
        }

        Ok(())
    }

    async fn uninstall(&self, target: &Path) -> Result<(), AdapterError> {
        self.health_check().await?;
        if let Some(host_target) = self.bind_mount_for_path(target).await? {
            return LocalAdapter::new().uninstall(&host_target).await;
        }
        let command = format!(
            "rm -rf \"{}\"",
            shell_escape_double_quoted(&target.display().to_string())
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
                "remove installed target from container",
            )
            .await?;
        if output.status.success() {
            return Ok(());
        }

        Err(AdapterError::Runtime {
            detail: format!(
                "docker uninstall failed in container `{}` for target `{}`: status={} stderr=`{}`",
                self.container_name,
                target.display(),
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        })
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

fn shell_escape_double_quoted(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\"', "\\\"")
}

fn agent_relative_path(agent: &AgentKind) -> Option<&'static str> {
    default_agent_path(agent)?.strip_prefix("~/")
}

fn container_detection_root(agent: &AgentKind) -> Option<&'static str> {
    agent_relative_path(agent)?.strip_suffix("/skills")
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
