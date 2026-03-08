//! `docker mount-hint` subcommand.
//!
//! Inspects the current `skills.toml` and a running Docker container to
//! produce a list of recommended `-v` bind-mount flags.  The output can
//! be copy-pasted into `docker run` or `docker-compose.yml`.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use eden_skills_core::adapter::DockerAdapter;
use eden_skills_core::config::{config_dir_from_path, AgentKind, Config, TargetConfig};
use eden_skills_core::error::EdenError;
use eden_skills_core::paths::{default_agent_path, normalize_lexical, resolve_path_string};

use super::common::{load_config_with_context, resolve_config_path};

#[derive(Debug, Clone)]
struct MountRecommendation {
    host_source: PathBuf,
    container_dest: PathBuf,
    read_only: bool,
}

/// Print recommended `docker run -v` flags for the given container,
/// based on the current `skills.toml` configuration.
pub async fn docker_mount_hint_async(
    container_name: &str,
    config_path: &str,
) -> Result<(), EdenError> {
    let config_path_buf = resolve_config_path(config_path)?;
    let config_path = config_path_buf.as_path();
    let loaded = load_config_with_context(config_path, false)?;
    let config_dir = config_dir_from_path(config_path);
    let adapter = DockerAdapter::new(container_name).map_err(EdenError::from)?;
    let recommendations =
        build_mount_recommendations(&loaded.config, &config_dir, &adapter).await?;

    let mut all_covered = true;
    for recommendation in &recommendations {
        if !recommendation_is_covered(&adapter, recommendation).await? {
            all_covered = false;
            break;
        }
    }

    if all_covered {
        println!(
            "  ✓ Container '{}' already has all recommended bind mounts.",
            container_name
        );
        return Ok(());
    }

    println!("  Docker mount-hint for container '{}':", container_name);
    println!();
    println!("  Recommended bind mounts (add to your docker run / docker-compose):");
    println!();
    for recommendation in &recommendations {
        let suffix = if recommendation.read_only { ":ro" } else { "" };
        println!(
            "    -v {}:{}{}",
            recommendation.host_source.display(),
            recommendation.container_dest.display(),
            suffix
        );
    }
    println!();
    println!("  After adding these mounts, restart the container and run:");
    println!("    eden-skills apply --config {}", config_path.display());
    Ok(())
}

async fn build_mount_recommendations(
    config: &Config,
    config_dir: &Path,
    adapter: &DockerAdapter,
) -> Result<Vec<MountRecommendation>, EdenError> {
    let storage_root = resolve_path_string(&config.storage_root, config_dir)?;
    let container_home = adapter.container_home().await.map_err(EdenError::from)?;
    let mut recommendations = Vec::new();
    let mut seen = HashSet::new();

    push_mount_recommendation(
        &mut recommendations,
        &mut seen,
        MountRecommendation {
            host_source: normalize_lexical(&storage_root),
            container_dest: normalize_lexical(&container_home.join(".eden-skills/skills")),
            read_only: true,
        },
    );

    for skill in &config.skills {
        for target in &skill.targets {
            if target.environment != format!("docker:{}", adapter.container_name()) {
                continue;
            }
            let Some(host_source) = resolve_host_agent_source(target, config_dir)? else {
                continue;
            };
            let container_dest = resolve_container_target_dest(target, adapter)
                .await
                .map_err(EdenError::from)?;
            push_mount_recommendation(
                &mut recommendations,
                &mut seen,
                MountRecommendation {
                    host_source: normalize_lexical(&host_source),
                    container_dest: normalize_lexical(&container_dest),
                    read_only: false,
                },
            );
        }
    }

    Ok(recommendations)
}

fn push_mount_recommendation(
    recommendations: &mut Vec<MountRecommendation>,
    seen: &mut HashSet<String>,
    recommendation: MountRecommendation,
) {
    let key = format!(
        "{}|{}|{}",
        recommendation.host_source.display(),
        recommendation.container_dest.display(),
        recommendation.read_only
    );
    if seen.insert(key) {
        recommendations.push(recommendation);
    }
}

fn resolve_host_agent_source(
    target: &TargetConfig,
    config_dir: &Path,
) -> Result<Option<PathBuf>, EdenError> {
    if let Some(default_path) = default_agent_path(&target.agent) {
        return resolve_path_string(default_path, config_dir).map(Some);
    }
    if matches!(target.agent, AgentKind::Custom) {
        return Ok(target.path.as_deref().map(PathBuf::from));
    }
    Ok(None)
}

async fn resolve_container_target_dest(
    target: &TargetConfig,
    adapter: &DockerAdapter,
) -> Result<PathBuf, eden_skills_core::error::AdapterError> {
    if let Some(path) = &target.path {
        return Ok(PathBuf::from(path));
    }
    if let Some(expected_path) = &target.expected_path {
        return Ok(PathBuf::from(expected_path));
    }
    adapter.default_target_root_for_agent(&target.agent).await
}

async fn recommendation_is_covered(
    adapter: &DockerAdapter,
    recommendation: &MountRecommendation,
) -> Result<bool, EdenError> {
    let actual_host_path = if recommendation.read_only {
        adapter
            .mounted_host_path_for_path(&recommendation.container_dest)
            .await
            .map_err(EdenError::from)?
    } else {
        adapter
            .bind_mount_for_path(&recommendation.container_dest)
            .await
            .map_err(EdenError::from)?
    };
    Ok(actual_host_path.is_some_and(|actual| {
        normalize_lexical(&actual) == normalize_lexical(&recommendation.host_source)
    }))
}
