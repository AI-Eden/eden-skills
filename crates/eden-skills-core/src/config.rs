use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::error::EdenError;
use crate::paths::resolve_path_string;

const DEFAULT_STORAGE_ROOT: &str = "~/.local/share/eden-skills/repos";

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
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

    let value: serde_yaml::Value = serde_yaml::from_str(&config_raw)
        .map_err(|err| EdenError::Validation(format!("root: invalid yaml: {err}")))?;
    let warnings = collect_top_level_unknown_key_warnings(&value)?;

    if options.strict && !warnings.is_empty() {
        return Err(EdenError::Validation(format!(
            "root: unknown top-level keys in strict mode: {}",
            warnings.join(", ")
        )));
    }

    let raw: RawConfig = serde_yaml::from_value(value)
        .map_err(|err| EdenError::Validation(format!("root: invalid config shape: {err}")))?;

    let config = raw.into_config(config_dir)?;
    Ok(LoadedConfig { config, warnings })
}

fn collect_top_level_unknown_key_warnings(
    value: &serde_yaml::Value,
) -> Result<Vec<String>, EdenError> {
    let Some(map) = value.as_mapping() else {
        return Err(EdenError::Validation(
            "root: expected top-level YAML mapping".to_string(),
        ));
    };

    let mut warnings = Vec::new();
    let allowed_keys = ["version", "storage", "skills"];
    for key in map.keys() {
        let Some(key_str) = key.as_str() else {
            warnings.push("unknown non-string top-level key".to_string());
            continue;
        };
        if !allowed_keys.contains(&key_str) {
            warnings.push(format!("unknown top-level key `{key_str}`"));
        }
    }
    Ok(warnings)
}

#[derive(Debug, Clone, Deserialize)]
struct RawConfig {
    version: Option<u32>,
    storage: Option<RawStorageConfig>,
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
            let skill = raw_skill.into_skill(config_dir, &skill_path)?;
            if !ids.insert(skill.id.clone()) {
                return Err(EdenError::Validation(format!(
                    "{skill_path}.id: duplicate id `{}`",
                    skill.id
                )));
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
struct RawSkillConfig {
    id: Option<String>,
    source: Option<RawSourceConfig>,
    install: Option<RawInstallConfig>,
    targets: Option<Vec<RawTargetConfig>>,
    verify: Option<RawVerifyConfig>,
    safety: Option<RawSafetyConfig>,
}

impl RawSkillConfig {
    fn into_skill(self, config_dir: &Path, field_path: &str) -> Result<SkillConfig, EdenError> {
        let id = required(self.id, &format!("{field_path}.id"))?;
        let source = required(self.source, &format!("{field_path}.source"))?
            .into_source_config(&format!("{field_path}.source"))?;
        validate_repo_url(&source.repo, &format!("{field_path}.source.repo"))?;

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
            .into_verify_config(install_mode, &format!("{field_path}.verify"))?;
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

        Ok(TargetConfig {
            agent,
            expected_path: self.expected_path,
            path: self.path,
        })
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
struct RawVerifyConfig {
    enabled: Option<bool>,
    checks: Option<Vec<String>>,
}

impl RawVerifyConfig {
    fn into_verify_config(
        self,
        install_mode: InstallMode,
        _field_path: &str,
    ) -> Result<VerifyConfig, EdenError> {
        let enabled = self.enabled.unwrap_or(true);
        let checks = self
            .checks
            .unwrap_or_else(|| default_verify_checks(install_mode));

        Ok(VerifyConfig { enabled, checks })
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

fn validate_repo_url(url: &str, field_path: &str) -> Result<(), EdenError> {
    let is_https = url.starts_with("https://");
    let is_ssh = url.starts_with("ssh://");
    let is_scp_like = url.starts_with("git@") && url.contains(':');
    if is_https || is_ssh || is_scp_like {
        return Ok(());
    }
    Err(EdenError::Validation(format!(
        "{field_path}: must be a valid git URL (https/ssh)"
    )))
}

pub fn config_dir_from_path(config_path: &Path) -> PathBuf {
    config_path.parent().unwrap_or(Path::new(".")).to_path_buf()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::{load_from_file, LoadOptions};

    #[test]
    fn load_valid_config_with_defaults() {
        let dir = tempdir().expect("tempdir");
        let config_path = dir.path().join("skills.yaml");
        fs::write(
            &config_path,
            r#"
version: 1
skills:
  - id: "x"
    source:
      repo: "https://github.com/vercel-labs/skills.git"
    targets:
      - agent: "claude-code"
"#,
        )
        .expect("write config");

        let loaded = load_from_file(&config_path, LoadOptions::default()).expect("load config");
        assert_eq!(
            loaded.config.storage_root,
            "~/.local/share/eden-skills/repos"
        );
        assert_eq!(loaded.config.skills.len(), 1);
        assert_eq!(loaded.config.skills[0].source.subpath, ".");
        assert_eq!(loaded.config.skills[0].source.r#ref, "main");
        assert_eq!(
            loaded.config.skills[0].verify.checks,
            vec![
                "path-exists".to_string(),
                "target-resolves".to_string(),
                "is-symlink".to_string()
            ]
        );
    }

    #[test]
    fn reject_custom_target_without_path() {
        let dir = tempdir().expect("tempdir");
        let config_path = dir.path().join("skills.yaml");
        fs::write(
            &config_path,
            r#"
version: 1
skills:
  - id: "x"
    source:
      repo: "https://github.com/vercel-labs/skills.git"
    targets:
      - agent: "custom"
"#,
        )
        .expect("write config");

        let err = load_from_file(&config_path, LoadOptions::default()).expect_err("expected error");
        let message = err.to_string();
        assert!(message.contains("skills[0].targets[0].path"));
    }
}
