use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};

use crate::error::EdenError;
use crate::paths::resolve_path_string;

const DEFAULT_STORAGE_ROOT: &str = "~/.local/share/eden-skills/repos";
const REGISTRY_MODE_REPO_PREFIX: &str = "registry://";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub version: u32,
    pub storage_root: String,
    pub skills: Vec<SkillConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillConfig {
    pub id: String,
    pub source: SourceConfig,
    pub install: InstallConfig,
    pub targets: Vec<TargetConfig>,
    pub verify: VerifyConfig,
    pub safety: SafetyConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceConfig {
    pub repo: String,
    pub subpath: String,
    pub r#ref: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum InstallMode {
    Symlink,
    Copy,
}

impl InstallMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Symlink => "symlink",
            Self::Copy => "copy",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstallConfig {
    pub mode: InstallMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentKind {
    ClaudeCode,
    Cursor,
    Custom,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetConfig {
    pub agent: AgentKind,
    pub expected_path: Option<String>,
    pub path: Option<String>,
    pub environment: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifyConfig {
    pub enabled: bool,
    pub checks: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SafetyConfig {
    pub no_exec_metadata_only: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LoadOptions {
    pub strict: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedConfig {
    pub config: Config,
    pub warnings: Vec<String>,
}

pub fn load_from_file(config_path: &Path, options: LoadOptions) -> Result<LoadedConfig, EdenError> {
    let config_raw = fs::read_to_string(config_path)?;
    let config_dir = config_path.parent().unwrap_or(Path::new("."));

    let value: toml::Value = toml::from_str(&config_raw)
        .map_err(|err| EdenError::Validation(format!("root: invalid toml: {err}")))?;
    let warnings = collect_top_level_unknown_key_warnings(&value)?;

    if options.strict && !warnings.is_empty() {
        return Err(EdenError::Validation(format!(
            "root: unknown top-level keys in strict mode: {}",
            warnings.join(", ")
        )));
    }

    let raw: RawConfig = toml::from_str(&config_raw)
        .map_err(|err| EdenError::Validation(format!("root: invalid config shape: {err}")))?;

    let config = raw.into_config(config_dir)?;
    Ok(LoadedConfig { config, warnings })
}

fn collect_top_level_unknown_key_warnings(value: &toml::Value) -> Result<Vec<String>, EdenError> {
    let Some(map) = value.as_table() else {
        return Err(EdenError::Validation(
            "root: expected top-level TOML table".to_string(),
        ));
    };

    let mut warnings = Vec::new();
    let allowed_keys = ["version", "storage", "registries", "skills"];
    for key in map.keys() {
        if !allowed_keys.contains(&key.as_str()) {
            warnings.push(format!("unknown top-level key `{key}`"));
        }
    }
    Ok(warnings)
}

#[derive(Debug, Clone, Deserialize)]
struct RawConfig {
    version: Option<u32>,
    storage: Option<RawStorageConfig>,
    registries: Option<BTreeMap<String, RawRegistryConfig>>,
    skills: Option<Vec<RawSkillConfig>>,
}

impl RawConfig {
    fn into_config(self, config_dir: &Path) -> Result<Config, EdenError> {
        let version = required(self.version, "version")?;
        if version != 1 {
            return Err(EdenError::Validation(format!(
                "version: expected 1, got {version}"
            )));
        }

        let storage_root = self
            .storage
            .and_then(|storage| storage.root)
            .unwrap_or_else(|| DEFAULT_STORAGE_ROOT.to_string());
        resolve_path_string(&storage_root, config_dir)?;

        let raw_registries = self.registries.unwrap_or_default();
        let mut registry_names = HashSet::new();
        for (registry_name, raw_registry) in raw_registries {
            if registry_name.trim().is_empty() {
                return Err(EdenError::Validation(
                    "registries: registry name must not be empty".to_string(),
                ));
            }
            validate_repo_url(
                &raw_registry.url,
                &format!("registries.{registry_name}.url"),
            )?;
            if let Some(priority) = raw_registry.priority {
                if priority < 0 {
                    return Err(EdenError::Validation(format!(
                        "registries.{registry_name}.priority: must be non-negative"
                    )));
                }
            }
            let _auto_update = raw_registry.auto_update.unwrap_or(false);
            registry_names.insert(registry_name);
        }
        let has_registries = !registry_names.is_empty();

        let raw_skills = required(self.skills, "skills")?;
        if raw_skills.is_empty() {
            return Err(EdenError::Validation(
                "skills: must contain at least one skill".to_string(),
            ));
        }

        let mut ids = HashSet::new();
        let mut skills = Vec::with_capacity(raw_skills.len());
        for (idx, raw_skill) in raw_skills.into_iter().enumerate() {
            let skill_path = format!("skills[{idx}]");
            let skill =
                raw_skill.into_skill(config_dir, &skill_path, has_registries, &registry_names)?;
            if !ids.insert(skill.id.clone()) {
                return Err(phase2_validation_error(
                    "DUPLICATE_SKILL_ID",
                    &format!("{skill_path}.id"),
                    &format!("duplicate id `{}`", skill.id),
                ));
            }
            skills.push(skill);
        }

        Ok(Config {
            version,
            storage_root,
            skills,
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
struct RawStorageConfig {
    root: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct RawRegistryConfig {
    url: String,
    priority: Option<i64>,
    auto_update: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
struct RawSkillConfig {
    id: Option<String>,
    name: Option<String>,
    version: Option<String>,
    registry: Option<String>,
    source: Option<RawSourceConfig>,
    install: Option<RawInstallConfig>,
    targets: Option<Vec<RawTargetConfig>>,
    verify: Option<RawVerifyConfig>,
    safety: Option<RawSafetyConfig>,
}

impl RawSkillConfig {
    fn into_skill(
        self,
        config_dir: &Path,
        field_path: &str,
        has_registries: bool,
        registry_names: &HashSet<String>,
    ) -> Result<SkillConfig, EdenError> {
        let mode_a_present = self.id.is_some() || self.source.is_some();
        let mode_b_present =
            self.name.is_some() || self.version.is_some() || self.registry.is_some();

        if self.name.is_some() && mode_a_present {
            return Err(phase2_validation_error(
                "INVALID_SKILL_MODE",
                field_path,
                "Mode B (`name`) cannot be mixed with Mode A (`id` + `source`)",
            ));
        }
        if self.name.is_none() && mode_b_present {
            return Err(phase2_validation_error(
                "INVALID_SKILL_MODE",
                field_path,
                "Mode B fields require `name`",
            ));
        }

        let (id, source) = if let Some(name) = self.name {
            if !has_registries {
                return Err(phase2_validation_error(
                    "MISSING_REGISTRIES",
                    field_path,
                    "Mode B skill requires [registries] section",
                ));
            }

            let version_constraint = self.version.unwrap_or_else(|| "*".to_string());
            validate_semver_constraint(&version_constraint, &format!("{field_path}.version"))?;

            let registry_name = self.registry;
            if let Some(registry_name) = registry_name.as_ref() {
                if !registry_names.contains(registry_name) {
                    return Err(phase2_validation_error(
                        "UNKNOWN_REGISTRY",
                        &format!("{field_path}.registry"),
                        &format!("unknown registry `{registry_name}`"),
                    ));
                }
            }

            (
                name,
                SourceConfig {
                    repo: encode_registry_mode_repo(registry_name.as_deref()),
                    subpath: ".".to_string(),
                    r#ref: version_constraint,
                },
            )
        } else {
            let id = required(self.id, &format!("{field_path}.id"))?;
            let source = required(self.source, &format!("{field_path}.source"))?
                .into_source_config(&format!("{field_path}.source"))?;
            validate_repo_url(&source.repo, &format!("{field_path}.source.repo"))?;
            (id, source)
        };

        let install_mode = self
            .install
            .and_then(|install| install.mode)
            .unwrap_or(InstallMode::Symlink);
        let install = InstallConfig { mode: install_mode };

        let raw_targets = required(self.targets, &format!("{field_path}.targets"))?;
        if raw_targets.is_empty() {
            return Err(EdenError::Validation(format!(
                "{field_path}.targets: must contain at least one target"
            )));
        }
        let mut targets = Vec::with_capacity(raw_targets.len());
        for (target_idx, raw_target) in raw_targets.into_iter().enumerate() {
            targets.push(
                raw_target.into_target_config(
                    config_dir,
                    &format!("{field_path}.targets[{target_idx}]"),
                )?,
            );
        }

        let verify = self
            .verify
            .unwrap_or_default()
            .into_verify_config(install_mode);
        if verify.enabled && verify.checks.is_empty() {
            return Err(EdenError::Validation(format!(
                "{field_path}.verify.checks: must not be empty when verify.enabled=true"
            )));
        }

        let safety = self.safety.unwrap_or_default().into_safety_config();

        Ok(SkillConfig {
            id,
            source,
            install,
            targets,
            verify,
            safety,
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
struct RawSourceConfig {
    repo: Option<String>,
    subpath: Option<String>,
    r#ref: Option<String>,
}

impl RawSourceConfig {
    fn into_source_config(self, field_path: &str) -> Result<SourceConfig, EdenError> {
        let repo = required(self.repo, &format!("{field_path}.repo"))?;
        let subpath = self.subpath.unwrap_or_else(|| ".".to_string());
        let r#ref = self.r#ref.unwrap_or_else(|| "main".to_string());

        Ok(SourceConfig {
            repo,
            subpath,
            r#ref,
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
struct RawInstallConfig {
    mode: Option<InstallMode>,
}

#[derive(Debug, Clone, Deserialize)]
struct RawTargetConfig {
    agent: Option<AgentKind>,
    expected_path: Option<String>,
    path: Option<String>,
    environment: Option<String>,
}

impl RawTargetConfig {
    fn into_target_config(
        self,
        config_dir: &Path,
        field_path: &str,
    ) -> Result<TargetConfig, EdenError> {
        let agent = required(self.agent, &format!("{field_path}.agent"))?;
        if matches!(agent, AgentKind::Custom) && self.path.is_none() {
            return Err(EdenError::Validation(format!(
                "{field_path}.path: required when agent=custom"
            )));
        }

        if let Some(path) = &self.path {
            resolve_path_string(path, config_dir)?;
        }
        if let Some(expected_path) = &self.expected_path {
            resolve_path_string(expected_path, config_dir)?;
        }
        let environment = self.environment.unwrap_or_else(|| "local".to_string());
        validate_environment(&environment, &format!("{field_path}.environment"))?;

        Ok(TargetConfig {
            agent,
            expected_path: self.expected_path,
            path: self.path,
            environment,
        })
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
struct RawVerifyConfig {
    enabled: Option<bool>,
    checks: Option<Vec<String>>,
}

impl RawVerifyConfig {
    fn into_verify_config(self, install_mode: InstallMode) -> VerifyConfig {
        let enabled = self.enabled.unwrap_or(true);
        let checks = self
            .checks
            .unwrap_or_else(|| default_verify_checks(install_mode));

        VerifyConfig { enabled, checks }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
struct RawSafetyConfig {
    no_exec_metadata_only: Option<bool>,
}

impl RawSafetyConfig {
    fn into_safety_config(self) -> SafetyConfig {
        SafetyConfig {
            no_exec_metadata_only: self.no_exec_metadata_only.unwrap_or(false),
        }
    }
}

fn required<T>(value: Option<T>, field_path: &str) -> Result<T, EdenError> {
    value.ok_or_else(|| EdenError::Validation(format!("{field_path}: missing required field")))
}

fn phase2_validation_error(code: &str, field_path: &str, detail: &str) -> EdenError {
    EdenError::Validation(format!("{code}: {field_path}: {detail}"))
}

pub fn encode_registry_mode_repo(registry_name: Option<&str>) -> String {
    match registry_name {
        Some(name) if !name.trim().is_empty() => format!("{REGISTRY_MODE_REPO_PREFIX}{name}"),
        _ => REGISTRY_MODE_REPO_PREFIX.to_string(),
    }
}

pub fn decode_registry_mode_repo(repo: &str) -> Option<Option<String>> {
    let rest = repo.strip_prefix(REGISTRY_MODE_REPO_PREFIX)?;
    if rest.is_empty() {
        Some(None)
    } else {
        Some(Some(rest.to_string()))
    }
}

pub fn is_registry_mode_repo(repo: &str) -> bool {
    repo.starts_with(REGISTRY_MODE_REPO_PREFIX)
}

fn validate_semver_constraint(value: &str, field_path: &str) -> Result<(), EdenError> {
    let constraint = value.trim();
    if constraint.is_empty() {
        return Err(phase2_validation_error(
            "INVALID_SEMVER",
            field_path,
            "version constraint must not be empty",
        ));
    }
    if constraint == "*" {
        return Ok(());
    }
    if Version::parse(constraint).is_ok() {
        return Ok(());
    }

    VersionReq::parse(constraint).map_err(|err| {
        phase2_validation_error(
            "INVALID_SEMVER",
            field_path,
            &format!("invalid semver constraint `{constraint}`: {err}"),
        )
    })?;
    Ok(())
}

fn validate_environment(environment: &str, field_path: &str) -> Result<(), EdenError> {
    if environment == "local" {
        return Ok(());
    }
    if let Some(container_name) = environment.strip_prefix("docker:") {
        if !container_name.trim().is_empty() {
            return Ok(());
        }
    }

    Err(phase2_validation_error(
        "INVALID_ENVIRONMENT",
        field_path,
        "expected `local` or `docker:<container>`",
    ))
}

fn default_verify_checks(install_mode: InstallMode) -> Vec<String> {
    match install_mode {
        InstallMode::Symlink => vec![
            "path-exists".to_string(),
            "target-resolves".to_string(),
            "is-symlink".to_string(),
        ],
        InstallMode::Copy => vec!["path-exists".to_string(), "content-present".to_string()],
    }
}

pub fn default_verify_checks_for_mode(install_mode: InstallMode) -> Vec<String> {
    default_verify_checks(install_mode)
}

pub fn validate_config(config: &Config, config_dir: &Path) -> Result<(), EdenError> {
    if config.version != 1 {
        return Err(EdenError::Validation(format!(
            "version: expected 1, got {}",
            config.version
        )));
    }

    resolve_path_string(&config.storage_root, config_dir)
        .map_err(|err| EdenError::Validation(format!("storage.root: invalid path: {err}")))?;

    if config.skills.is_empty() {
        return Err(EdenError::Validation(
            "skills: must contain at least one skill".to_string(),
        ));
    }

    let mut ids = HashSet::new();
    for (idx, skill) in config.skills.iter().enumerate() {
        let skill_path = format!("skills[{idx}]");
        if !ids.insert(skill.id.clone()) {
            return Err(phase2_validation_error(
                "DUPLICATE_SKILL_ID",
                &format!("{skill_path}.id"),
                &format!("duplicate id `{}`", skill.id),
            ));
        }

        if is_registry_mode_repo(&skill.source.repo) {
            validate_semver_constraint(&skill.source.r#ref, &format!("{skill_path}.version"))?;
        } else {
            validate_repo_url(&skill.source.repo, &format!("{skill_path}.source.repo"))?;
        }

        if skill.targets.is_empty() {
            return Err(EdenError::Validation(format!(
                "{skill_path}.targets: must contain at least one target"
            )));
        }
        for (target_idx, target) in skill.targets.iter().enumerate() {
            let target_path = format!("{skill_path}.targets[{target_idx}]");
            if matches!(target.agent, AgentKind::Custom) && target.path.is_none() {
                return Err(EdenError::Validation(format!(
                    "{target_path}.path: required when agent=custom"
                )));
            }
            if let Some(path) = &target.path {
                resolve_path_string(path, config_dir).map_err(|err| {
                    EdenError::Validation(format!("{target_path}.path: invalid path: {}", err))
                })?;
            }
            if let Some(expected) = &target.expected_path {
                resolve_path_string(expected, config_dir).map_err(|err| {
                    EdenError::Validation(format!(
                        "{target_path}.expected_path: invalid path: {}",
                        err
                    ))
                })?;
            }
            validate_environment(&target.environment, &format!("{target_path}.environment"))?;
        }

        if skill.verify.enabled && skill.verify.checks.is_empty() {
            return Err(EdenError::Validation(format!(
                "{skill_path}.verify.checks: must not be empty when verify.enabled=true"
            )));
        }
    }

    Ok(())
}

fn validate_repo_url(url: &str, field_path: &str) -> Result<(), EdenError> {
    let is_https = url.starts_with("https://");
    let is_ssh = url.starts_with("ssh://");
    let is_scp_like = url.starts_with("git@") && url.contains(':');
    let is_file = url.starts_with("file://");
    if is_https || is_ssh || is_scp_like || is_file {
        return Ok(());
    }
    Err(EdenError::Validation(format!(
        "{field_path}: must be a valid git URL (https/ssh/file)"
    )))
}

pub fn config_dir_from_path(config_path: &Path) -> PathBuf {
    config_path.parent().unwrap_or(Path::new(".")).to_path_buf()
}
