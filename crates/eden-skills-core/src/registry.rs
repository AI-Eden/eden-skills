use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use semver::{Version, VersionReq};
use serde::Deserialize;

pub use crate::error::RegistryError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrySpec {
    pub name: String,
    pub url: String,
    pub priority: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrySource {
    pub name: String,
    pub priority: u32,
    pub root: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedSkill {
    pub registry_name: String,
    pub registry_priority: u32,
    pub repo: String,
    pub subpath: String,
    pub version: String,
    pub git_ref: String,
    pub commit: String,
}

pub fn parse_registry_specs_from_toml(
    config_toml: &str,
) -> Result<Vec<RegistrySpec>, RegistryError> {
    let raw: RawRegistryConfigFile =
        toml::from_str(config_toml).map_err(|err| RegistryError::Config {
            detail: format!("invalid config TOML for registries: {err}"),
        })?;

    let mut specs = Vec::new();
    for (name, entry) in raw.registries {
        if name.trim().is_empty() {
            return Err(RegistryError::Config {
                detail: "registry name must not be empty".to_string(),
            });
        }
        validate_git_url(&entry.url, &name)?;
        specs.push(RegistrySpec {
            name,
            url: entry.url,
            priority: entry.priority.unwrap_or(0),
        });
    }
    Ok(specs)
}

pub fn sort_registry_specs_by_priority(registries: &[RegistrySpec]) -> Vec<RegistrySpec> {
    let mut sorted = registries.to_vec();
    sorted.sort_by(|a, b| {
        b.priority
            .cmp(&a.priority)
            .then_with(|| a.name.cmp(&b.name))
    });
    sorted
}

pub fn resolve_skill_from_registry_sources(
    sources: &[RegistrySource],
    skill_name: &str,
    version_constraint: Option<&str>,
) -> Result<ResolvedSkill, RegistryError> {
    if skill_name.trim().is_empty() {
        return Err(RegistryError::Config {
            detail: "skill name must not be empty".to_string(),
        });
    }

    let mut ordered_sources = sources.to_vec();
    ordered_sources.sort_by(|a, b| {
        b.priority
            .cmp(&a.priority)
            .then_with(|| a.name.cmp(&b.name))
    });

    let mut searched = Vec::new();
    for source in &ordered_sources {
        searched.push(format!("{}({})", source.name, source.priority));
        let Some(entry) = load_skill_index_entry(source, skill_name)? else {
            continue;
        };

        let selected = select_version(&entry.versions, version_constraint)?;
        return Ok(ResolvedSkill {
            registry_name: source.name.clone(),
            registry_priority: source.priority,
            repo: entry.skill.repo,
            subpath: entry.skill.subpath.unwrap_or_else(|| ".".to_string()),
            version: selected.version.to_string(),
            git_ref: selected.git_ref.clone(),
            commit: selected.commit.clone(),
        });
    }

    Err(RegistryError::Resolution {
        detail: format!(
            "skill `{skill_name}` not found in configured registries: {}",
            searched.join(", ")
        ),
    })
}

#[derive(Debug, Deserialize)]
struct RawRegistryConfigFile {
    #[serde(default)]
    registries: BTreeMap<String, RawRegistrySpec>,
}

#[derive(Debug, Deserialize)]
struct RawRegistrySpec {
    url: String,
    priority: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct RawSkillIndexEntry {
    skill: RawIndexedSkill,
    #[serde(default)]
    versions: Vec<RawIndexedVersion>,
}

#[derive(Debug, Deserialize)]
struct RawIndexedSkill {
    name: String,
    repo: String,
    subpath: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawIndexedVersion {
    version: String,
    #[serde(rename = "ref")]
    git_ref: String,
    commit: String,
    yanked: Option<bool>,
}

#[derive(Debug)]
struct SkillIndexEntry {
    skill: RawIndexedSkill,
    versions: Vec<IndexedVersion>,
}

#[derive(Debug)]
struct IndexedVersion {
    version: Version,
    git_ref: String,
    commit: String,
    yanked: bool,
}

fn load_skill_index_entry(
    source: &RegistrySource,
    skill_name: &str,
) -> Result<Option<SkillIndexEntry>, RegistryError> {
    let index_path = skill_index_path(&source.root, skill_name)?;
    if !index_path.exists() {
        return Ok(None);
    }

    let entry_raw = fs::read_to_string(&index_path)?;
    let entry: RawSkillIndexEntry =
        toml::from_str(&entry_raw).map_err(|err| RegistryError::Resolution {
            detail: format!(
                "failed to parse registry index entry `{}`: {err}",
                index_path.display()
            ),
        })?;

    if entry.skill.name != skill_name {
        return Err(RegistryError::Resolution {
            detail: format!(
                "registry entry `{}` declares skill `{}` but expected `{}`",
                index_path.display(),
                entry.skill.name,
                skill_name
            ),
        });
    }

    let mut versions = Vec::with_capacity(entry.versions.len());
    for item in entry.versions {
        let parsed = Version::parse(&item.version).map_err(|err| RegistryError::Resolution {
            detail: format!(
                "registry entry `{}` contains invalid version `{}`: {err}",
                index_path.display(),
                item.version
            ),
        })?;
        versions.push(IndexedVersion {
            version: parsed,
            git_ref: item.git_ref,
            commit: item.commit,
            yanked: item.yanked.unwrap_or(false),
        });
    }

    Ok(Some(SkillIndexEntry {
        skill: entry.skill,
        versions,
    }))
}

fn skill_index_path(registry_root: &Path, skill_name: &str) -> Result<PathBuf, RegistryError> {
    let mut chars = skill_name.chars();
    let first = chars.next().ok_or_else(|| RegistryError::Config {
        detail: "skill name must not be empty".to_string(),
    })?;

    Ok(registry_root
        .join("index")
        .join(first.to_ascii_lowercase().to_string())
        .join(format!("{skill_name}.toml")))
}

fn select_version<'a>(
    versions: &'a [IndexedVersion],
    version_constraint: Option<&str>,
) -> Result<&'a IndexedVersion, RegistryError> {
    let candidates = versions
        .iter()
        .filter(|candidate| !candidate.yanked)
        .collect::<Vec<_>>();

    if candidates.is_empty() {
        return Err(RegistryError::Resolution {
            detail: "no non-yanked versions are available".to_string(),
        });
    }

    if let Some(raw_constraint) = version_constraint {
        let constraint = raw_constraint.trim();
        if constraint.is_empty() {
            return Err(RegistryError::Config {
                detail: "version constraint must not be empty".to_string(),
            });
        }

        if let Ok(exact) = Version::parse(constraint) {
            let matched = candidates
                .into_iter()
                .filter(|candidate| candidate.version == exact)
                .max_by(|left, right| left.version.cmp(&right.version));
            return matched.ok_or_else(|| RegistryError::Resolution {
                detail: format!(
                    "no version matched exact constraint `{constraint}`; available versions: {}",
                    available_versions(versions)
                ),
            });
        }

        let requirement = VersionReq::parse(constraint).map_err(|err| RegistryError::Config {
            detail: format!("invalid version constraint `{constraint}`: {err}"),
        })?;
        let matched = candidates
            .into_iter()
            .filter(|candidate| requirement.matches(&candidate.version))
            .max_by(|left, right| left.version.cmp(&right.version));
        return matched.ok_or_else(|| RegistryError::Resolution {
            detail: format!(
                "no version matched constraint `{constraint}`; available versions: {}",
                available_versions(versions)
            ),
        });
    }

    candidates
        .into_iter()
        .max_by(|left, right| left.version.cmp(&right.version))
        .ok_or_else(|| RegistryError::Resolution {
            detail: "no version candidates available".to_string(),
        })
}

fn available_versions(versions: &[IndexedVersion]) -> String {
    let mut available = versions
        .iter()
        .filter(|candidate| !candidate.yanked)
        .map(|candidate| candidate.version.clone())
        .collect::<Vec<_>>();
    available.sort_unstable();
    available.reverse();
    available
        .into_iter()
        .map(|version| version.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

fn validate_git_url(url: &str, registry_name: &str) -> Result<(), RegistryError> {
    let is_https = url.starts_with("https://");
    let is_ssh = url.starts_with("ssh://");
    let is_scp_like = url.starts_with("git@") && url.contains(':');
    let is_file = url.starts_with("file://");
    if is_https || is_ssh || is_scp_like || is_file {
        return Ok(());
    }

    Err(RegistryError::Config {
        detail: format!(
            "registry `{registry_name}` has invalid url `{url}` (expected https/ssh/file git URL)"
        ),
    })
}
