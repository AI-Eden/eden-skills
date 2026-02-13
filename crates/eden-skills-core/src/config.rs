#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub version: u32,
    pub storage_root: Option<String>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallMode {
    Symlink,
    Copy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstallConfig {
    pub mode: InstallMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
