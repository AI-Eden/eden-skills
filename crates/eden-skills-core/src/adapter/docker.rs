//! Docker container adapter for remote skill installation.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use serde::Deserialize;
use tokio::process::Command;

use crate::config::{AgentKind, InstallMode, TargetConfig};
use crate::error::AdapterError;
use crate::paths::{default_agent_path, normalize_lexical};

use super::local::LocalAdapter;
use super::manifest::{
    read_local_managed_manifest, write_local_managed_manifest, ManagedManifestReadResult,
};
use super::{shell_escape_double_quoted, TargetAdapter};

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

    pub(super) async fn read_managed_manifest(
        &self,
        agent_dir: &Path,
    ) -> Result<ManagedManifestReadResult, AdapterError> {
        self.health_check().await?;
        if let Some(host_agent_dir) = self.bind_mount_for_path(agent_dir).await? {
            return read_local_managed_manifest(&host_agent_dir).await;
        }

        let manifest_path = super::manifest::managed_manifest_path(agent_dir);
        let command = format!(
            "cat \"{}\"",
            shell_escape_double_quoted(&manifest_path.display().to_string())
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
                "read .eden-managed manifest in container",
            )
            .await?;
        if output.status.success() {
            return super::manifest::parse_managed_manifest(
                &manifest_path,
                &String::from_utf8_lossy(&output.stdout),
                "docker target",
            );
        }
        if output.status.code() == Some(1) {
            return Ok(ManagedManifestReadResult {
                manifest: crate::managed::ManagedManifest::default(),
                warning: None,
            });
        }

        Err(AdapterError::Runtime {
            detail: format!(
                "docker exec failed while reading manifest `{}` in container `{}`: status={} stderr=`{}`",
                manifest_path.display(),
                self.container_name,
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        })
    }

    pub(super) async fn write_managed_manifest(
        &self,
        agent_dir: &Path,
        manifest: &crate::managed::ManagedManifest,
    ) -> Result<(), AdapterError> {
        self.health_check().await?;
        if let Some(host_agent_dir) = self.bind_mount_for_path(agent_dir).await? {
            return write_local_managed_manifest(&host_agent_dir, manifest).await;
        }

        let manifest_path = super::manifest::managed_manifest_path(agent_dir);
        let temp_path = super::manifest::managed_manifest_temp_path();
        let encoded = manifest
            .to_pretty_json()
            .map_err(|err| AdapterError::Runtime {
                detail: format!("failed to serialize managed manifest: {err}"),
            })?;
        tokio::fs::write(&temp_path, format!("{encoded}\n")).await?;

        let mkdir_output = self
            .run_docker(
                vec![
                    OsString::from("exec"),
                    OsString::from(self.container_name.clone()),
                    OsString::from("sh"),
                    OsString::from("-c"),
                    OsString::from(format!(
                        "mkdir -p \"{}\"",
                        shell_escape_double_quoted(&agent_dir.display().to_string())
                    )),
                ],
                "ensure agent directory exists before writing manifest",
            )
            .await?;
        if !mkdir_output.status.success() {
            let _ = tokio::fs::remove_file(&temp_path).await;
            return Err(AdapterError::Runtime {
                detail: format!(
                    "docker exec failed while preparing manifest directory `{}` in container `{}`: status={} stderr=`{}`",
                    agent_dir.display(),
                    self.container_name,
                    mkdir_output.status,
                    String::from_utf8_lossy(&mkdir_output.stderr).trim()
                ),
            });
        }

        let copy_output = self
            .run_docker(
                vec![
                    OsString::from("cp"),
                    temp_path.as_os_str().to_os_string(),
                    OsString::from(format!(
                        "{}:{}",
                        self.container_name,
                        manifest_path.display()
                    )),
                ],
                "write .eden-managed manifest into container",
            )
            .await?;
        let _ = tokio::fs::remove_file(&temp_path).await;
        if copy_output.status.success() {
            return Ok(());
        }

        Err(AdapterError::Runtime {
            detail: format!(
                "docker cp failed while writing manifest `{}` in container `{}`: status={} stderr=`{}`",
                manifest_path.display(),
                self.container_name,
                copy_output.status,
                String::from_utf8_lossy(&copy_output.stderr).trim()
            ),
        })
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
