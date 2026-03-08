//! Install plan execution and adapter dispatch.

use std::fs;
use std::path::{Path, PathBuf};

use eden_skills_core::adapter::{
    read_managed_manifest, write_managed_manifest, DockerAdapter, LocalAdapter, TargetAdapter,
};
use eden_skills_core::config::{
    default_verify_checks_for_mode, encode_registry_mode_repo, AgentKind, Config, InstallMode,
    SkillConfig, SourceConfig, TargetConfig,
};
use eden_skills_core::error::EdenError;
use eden_skills_core::managed::{external_install_origin, local_install_origin, ManagedSource};
use eden_skills_core::paths::{normalize_lexical, resolve_path_string};
use eden_skills_core::plan::{build_plan, Action};
use eden_skills_core::source::resolve_skill_source_path;

use crate::ui::UiContext;

use super::platform::default_install_mode;
use crate::commands::common::{
    apply_plan_item, copy_recursively, ensure_parent_dir, path_is_symlink_or_junction,
    print_warning, remove_path,
};

#[derive(Debug, Default)]
pub(super) struct InstallExecutionSummary {
    pub(super) installed_targets: Vec<InstallTargetLine>,
    pub(super) conflicts: usize,
    pub(super) skipped_skills: usize,
    pub(super) docker_cp_hint_containers: Vec<String>,
}

#[derive(Debug)]
pub(super) struct InstallTargetLine {
    pub(super) skill_id: String,
    pub(super) target_path: String,
    pub(super) mode: String,
}

impl InstallExecutionSummary {
    pub(super) fn merge(&mut self, mut other: InstallExecutionSummary) {
        self.conflicts += other.conflicts;
        self.skipped_skills += other.skipped_skills;
        self.installed_targets.append(&mut other.installed_targets);
        for container in other.docker_cp_hint_containers.drain(..) {
            self.record_docker_cp_hint(container);
        }
    }

    pub(super) fn installed_skill_count(&self) -> usize {
        let mut seen = std::collections::HashSet::new();
        for target in &self.installed_targets {
            seen.insert(&target.skill_id);
        }
        seen.len()
    }

    pub(super) fn record_docker_cp_hint(&mut self, container_name: impl Into<String>) {
        let container_name = container_name.into();
        if !self
            .docker_cp_hint_containers
            .iter()
            .any(|existing| existing == &container_name)
        {
            self.docker_cp_hint_containers.push(container_name);
        }
    }
}

pub(super) async fn execute_install_plan_async(
    single_skill_config: &Config,
    config_dir: &Path,
    strict: bool,
    force: bool,
    ui: &UiContext,
) -> Result<InstallExecutionSummary, EdenError> {
    if single_skill_config.skills.iter().all(|skill| {
        skill
            .targets
            .iter()
            .all(|target| target.environment == "local")
    }) {
        return execute_install_plan(single_skill_config, config_dir, strict, force, ui).await;
    }

    let skill = single_skill_config
        .skills
        .first()
        .ok_or_else(|| EdenError::Runtime("install skill is missing".to_string()))?;
    let storage_root = resolve_path_string(&single_skill_config.storage_root, config_dir)?;
    let source_path = resolve_skill_source_path(&storage_root, skill);
    execute_single_skill_targets_async(skill, &source_path, config_dir, strict, force, ui).await
}

async fn execute_install_plan(
    single_skill_config: &Config,
    config_dir: &Path,
    strict: bool,
    _force: bool,
    ui: &UiContext,
) -> Result<InstallExecutionSummary, EdenError> {
    let plan = build_plan(single_skill_config, config_dir)?;
    let mut summary = InstallExecutionSummary::default();
    for item in &plan {
        match item.action {
            Action::Create | Action::Update => {
                apply_plan_item(item)?;
                update_managed_manifest_after_install(
                    &item.skill_id,
                    "local",
                    Path::new(&item.target_path).parent().ok_or_else(|| {
                        EdenError::Runtime(format!(
                            "installed target path missing parent directory: {}",
                            item.target_path
                        ))
                    })?,
                    ui,
                )
                .await?;
                summary.installed_targets.push(InstallTargetLine {
                    skill_id: item.skill_id.clone(),
                    target_path: item.target_path.clone(),
                    mode: item.install_mode.as_str().to_string(),
                });
            }
            Action::Conflict => summary.conflicts += 1,
            Action::Noop | Action::Remove => {}
        }
    }
    if strict && summary.conflicts > 0 {
        return Err(EdenError::Conflict(format!(
            "strict mode blocked install: {} conflict entries",
            summary.conflicts
        )));
    }
    Ok(summary)
}

pub(super) async fn install_local_source_skill_async(
    single_skill_config: &Config,
    config_dir: &Path,
    strict: bool,
    force: bool,
    ui: &UiContext,
) -> Result<InstallExecutionSummary, EdenError> {
    let skill = single_skill_config
        .skills
        .first()
        .ok_or_else(|| EdenError::Runtime("local install skill is missing".to_string()))?;
    let source_repo_root = resolve_path_string(&skill.source.repo, config_dir)?;
    let storage_root = resolve_path_string(&single_skill_config.storage_root, config_dir)?;
    let staged_repo_root = normalize_lexical(&storage_root.join(&skill.id));

    stage_local_source_into_storage(&source_repo_root, &staged_repo_root)?;
    let source_path = normalize_lexical(&staged_repo_root.join(&skill.source.subpath));
    if !source_path.exists() {
        return Err(EdenError::Runtime(format!(
            "source path missing for skill `{}`: {}",
            skill.id,
            source_path.display()
        )));
    }

    execute_single_skill_targets_async(skill, &source_path, config_dir, strict, force, ui).await
}

async fn execute_single_skill_targets_async(
    skill: &SkillConfig,
    source_path: &Path,
    config_dir: &Path,
    strict: bool,
    force: bool,
    ui: &UiContext,
) -> Result<InstallExecutionSummary, EdenError> {
    let mut summary = InstallExecutionSummary::default();
    for target in &skill.targets {
        if let Some(container_name) = target.environment.strip_prefix("docker:") {
            let docker = DockerAdapter::new(container_name).map_err(EdenError::from)?;
            let target_root = resolve_docker_target_root(target, &docker).await?;
            let target_path = normalize_lexical(&target_root.join(&skill.id));
            let uses_bind_mount = docker.bind_mount_for_path(&target_path).await?.is_some();
            docker
                .install(source_path, &target_path, skill.install.mode)
                .await
                .map_err(EdenError::from)?;
            update_managed_manifest_after_install(&skill.id, &target.environment, &target_root, ui)
                .await?;
            summary.installed_targets.push(InstallTargetLine {
                skill_id: skill.id.clone(),
                target_path: target_path.display().to_string(),
                mode: skill.install.mode.as_str().to_string(),
            });
            if !uses_bind_mount {
                summary.record_docker_cp_hint(container_name.to_string());
            }
            continue;
        }

        let target_root = eden_skills_core::paths::resolve_target_path(target, config_dir)?;
        let target_path = normalize_lexical(&target_root.join(&skill.id));
        if maybe_adopt_external_local_target(
            skill,
            &target.environment,
            &target_root,
            &target_path,
            force,
            ui,
        )
        .await?
        {
            summary.installed_targets.push(InstallTargetLine {
                skill_id: skill.id.clone(),
                target_path: target_path.display().to_string(),
                mode: skill.install.mode.as_str().to_string(),
            });
            continue;
        }
        match fs::symlink_metadata(&target_path) {
            Ok(metadata)
                if matches!(skill.install.mode, InstallMode::Symlink)
                    && !path_is_symlink_or_junction(&target_path, &metadata) =>
            {
                summary.conflicts += 1;
                continue;
            }
            Ok(metadata)
                if matches!(skill.install.mode, InstallMode::Copy)
                    && path_is_symlink_or_junction(&target_path, &metadata) =>
            {
                summary.conflicts += 1;
                continue;
            }
            Ok(_) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => return Err(EdenError::Io(err)),
        }

        LocalAdapter::new()
            .install(source_path, &target_path, skill.install.mode)
            .await
            .map_err(EdenError::from)?;
        update_managed_manifest_after_install(&skill.id, &target.environment, &target_root, ui)
            .await?;
        summary.installed_targets.push(InstallTargetLine {
            skill_id: skill.id.clone(),
            target_path: target_path.display().to_string(),
            mode: skill.install.mode.as_str().to_string(),
        });
    }

    if strict && summary.conflicts > 0 {
        return Err(EdenError::Conflict(format!(
            "strict mode blocked install: {} conflict entries",
            summary.conflicts
        )));
    }

    Ok(summary)
}

pub(super) async fn update_managed_manifest_after_install(
    skill_id: &str,
    environment: &str,
    agent_dir: &Path,
    ui: &UiContext,
) -> Result<(), EdenError> {
    let read_result = read_managed_manifest(environment, agent_dir)
        .await
        .map_err(EdenError::from)?;
    if let Some(warning) = read_result.warning {
        print_warning(ui, &warning);
    }

    let mut manifest = read_result.manifest;
    if environment.starts_with("docker:") {
        manifest.record_install(skill_id, ManagedSource::External, external_install_origin());
    } else {
        manifest.record_install(
            skill_id,
            ManagedSource::Local,
            local_install_origin(environment),
        );
    }

    write_managed_manifest(environment, agent_dir, &manifest)
        .await
        .map_err(EdenError::from)
}

async fn maybe_adopt_external_local_target(
    skill: &SkillConfig,
    environment: &str,
    agent_dir: &Path,
    target_path: &Path,
    force: bool,
    ui: &UiContext,
) -> Result<bool, EdenError> {
    if force || environment != "local" {
        return Ok(false);
    }

    let read_result = read_managed_manifest(environment, agent_dir)
        .await
        .map_err(EdenError::from)?;
    if let Some(warning) = read_result.warning {
        print_warning(ui, &warning);
    }
    let Some(record) = read_result.manifest.skill(&skill.id) else {
        return Ok(false);
    };
    let record_source = record.source.clone();
    let record_origin = record.origin.clone();
    if record_source != ManagedSource::External || fs::symlink_metadata(target_path).is_err() {
        return Ok(false);
    }

    let mut manifest = read_result.manifest;
    print_warning(
        ui,
        &format!(
            "Skill '{}' already exists, managed by external host ({}). Adopting into local config and keeping existing files. Use --force to overwrite files and take over management.",
            skill.id,
            describe_external_origin(&record_origin)
        ),
    );
    manifest.record_install(
        &skill.id,
        ManagedSource::Local,
        local_install_origin(environment),
    );
    write_managed_manifest(environment, agent_dir, &manifest)
        .await
        .map_err(EdenError::from)?;
    Ok(true)
}

pub(super) fn describe_external_origin(origin: &str) -> &str {
    origin.strip_prefix("host:").unwrap_or(origin)
}

pub(super) async fn resolve_docker_target_root(
    target: &TargetConfig,
    docker: &DockerAdapter,
) -> Result<PathBuf, EdenError> {
    if let Some(path) = &target.path {
        return Ok(PathBuf::from(path));
    }
    if let Some(expected_path) = &target.expected_path {
        return Ok(PathBuf::from(expected_path));
    }
    docker
        .default_target_root_for_agent(&target.agent)
        .await
        .map_err(EdenError::from)
}

fn stage_local_source_into_storage(
    source_repo_root: &Path,
    staged_repo_root: &Path,
) -> Result<(), EdenError> {
    if !source_repo_root.exists() {
        return Err(EdenError::Runtime(format!(
            "local source path does not exist: {}",
            source_repo_root.display()
        )));
    }

    ensure_parent_dir(staged_repo_root)?;
    if fs::symlink_metadata(staged_repo_root).is_ok() {
        remove_path(staged_repo_root)?;
    }
    copy_recursively(source_repo_root, staged_repo_root)?;
    Ok(())
}

pub(super) fn print_install_success_json(
    skill_id: &str,
    version_or_ref: &str,
) -> Result<(), EdenError> {
    let payload = serde_json::json!({
        "skill": skill_id,
        "version": version_or_ref,
        "status": "installed",
    });
    let encoded = serde_json::to_string_pretty(&payload)
        .map_err(|err| EdenError::Runtime(format!("failed to encode install json: {err}")))?;
    println!("{encoded}");
    Ok(())
}

pub(super) fn upsert_mode_b_skill(
    config: &mut Config,
    skill_name: &str,
    version_constraint: &str,
    registry: Option<&str>,
    target_override: Option<Vec<TargetConfig>>,
    install_mode_override: Option<InstallMode>,
) -> Result<(), EdenError> {
    let install_mode = install_mode_override.unwrap_or_else(default_install_mode);
    if let Some(skill) = config
        .skills
        .iter_mut()
        .find(|skill| skill.id == skill_name)
    {
        skill.source = SourceConfig {
            repo: encode_registry_mode_repo(registry),
            subpath: ".".to_string(),
            r#ref: version_constraint.to_string(),
        };
        skill.install.mode = install_mode;
        skill.verify.enabled = true;
        skill.verify.checks = default_verify_checks_for_mode(install_mode);
        if let Some(targets) = target_override {
            skill.targets = targets;
        }
        return Ok(());
    }

    let targets = target_override.unwrap_or_else(|| vec![default_install_target()]);

    config.skills.push(SkillConfig {
        id: skill_name.to_string(),
        source: SourceConfig {
            repo: encode_registry_mode_repo(registry),
            subpath: ".".to_string(),
            r#ref: version_constraint.to_string(),
        },
        install: eden_skills_core::config::InstallConfig { mode: install_mode },
        targets,
        verify: eden_skills_core::config::VerifyConfig {
            enabled: true,
            checks: default_verify_checks_for_mode(install_mode),
        },
        safety: eden_skills_core::config::SafetyConfig {
            no_exec_metadata_only: false,
        },
    });
    Ok(())
}

pub(super) fn upsert_mode_a_skill(
    config: &mut Config,
    skill_id: &str,
    repo: &str,
    subpath: &str,
    reference: &str,
    target_override: Option<Vec<TargetConfig>>,
    install_mode_override: Option<InstallMode>,
) -> Result<(), EdenError> {
    let install_mode = install_mode_override.unwrap_or_else(default_install_mode);
    if let Some(skill) = config.skills.iter_mut().find(|skill| skill.id == skill_id) {
        skill.source = SourceConfig {
            repo: repo.to_string(),
            subpath: subpath.to_string(),
            r#ref: reference.to_string(),
        };
        skill.install.mode = install_mode;
        skill.verify.enabled = true;
        skill.verify.checks = default_verify_checks_for_mode(install_mode);
        if let Some(targets) = target_override {
            skill.targets = targets;
        }
        return Ok(());
    }

    let effective_targets = target_override.unwrap_or_else(|| vec![default_install_target()]);

    config.skills.push(SkillConfig {
        id: skill_id.to_string(),
        source: SourceConfig {
            repo: repo.to_string(),
            subpath: subpath.to_string(),
            r#ref: reference.to_string(),
        },
        install: eden_skills_core::config::InstallConfig { mode: install_mode },
        targets: effective_targets,
        verify: eden_skills_core::config::VerifyConfig {
            enabled: true,
            checks: default_verify_checks_for_mode(install_mode),
        },
        safety: eden_skills_core::config::SafetyConfig {
            no_exec_metadata_only: false,
        },
    });
    Ok(())
}

pub(super) fn default_install_target() -> TargetConfig {
    TargetConfig {
        agent: AgentKind::ClaudeCode,
        expected_path: None,
        path: None,
        environment: "local".to_string(),
    }
}

pub(super) fn default_docker_install_target(container_name: &str) -> TargetConfig {
    TargetConfig {
        agent: AgentKind::ClaudeCode,
        expected_path: None,
        path: None,
        environment: format!("docker:{container_name}"),
    }
}

pub(super) fn should_preserve_existing_targets(skill: &SkillConfig) -> bool {
    skill.targets.iter().any(|target| {
        target.environment != "local" || target.path.is_some() || target.expected_path.is_some()
    })
}
