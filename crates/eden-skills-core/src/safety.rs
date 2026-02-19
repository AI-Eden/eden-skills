use std::collections::BTreeSet;
use std::fs;
use std::io::Read;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::Config;
use crate::error::EdenError;
use crate::paths::{normalize_lexical, resolve_path_string};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LicenseStatus {
    Permissive,
    NonPermissive,
    Unknown,
}

impl LicenseStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Permissive => "permissive",
            Self::NonPermissive => "non-permissive",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillSafetyReport {
    pub skill_id: String,
    pub repo_path: PathBuf,
    pub source_path: PathBuf,
    pub metadata_path: PathBuf,
    pub license_status: LicenseStatus,
    pub license_hint: Option<String>,
    pub risk_labels: Vec<String>,
    pub no_exec_metadata_only: bool,
    pub commit_sha: Option<String>,
    pub retrieved_at_unix: u64,
}

pub fn analyze_skills(
    config: &Config,
    config_dir: &Path,
) -> Result<Vec<SkillSafetyReport>, EdenError> {
    let storage_root = resolve_path_string(&config.storage_root, config_dir)?;
    let mut reports = Vec::with_capacity(config.skills.len());
    let retrieved_at_unix = unix_now()?;

    for skill in &config.skills {
        let repo_path = normalize_lexical(&storage_root.join(&skill.id));
        let source_path = normalize_lexical(&repo_path.join(&skill.source.subpath));
        let metadata_path = repo_path.join(".eden-safety.toml");

        let (license_status, license_hint) = detect_license_status(&repo_path);
        let risk_labels = detect_risk_labels(&source_path).unwrap_or_default();
        let commit_sha = read_commit_sha(&repo_path);

        reports.push(SkillSafetyReport {
            skill_id: skill.id.clone(),
            repo_path,
            source_path,
            metadata_path,
            license_status,
            license_hint,
            risk_labels,
            no_exec_metadata_only: skill.safety.no_exec_metadata_only,
            commit_sha,
            retrieved_at_unix,
        });
    }

    Ok(reports)
}

pub fn persist_reports(reports: &[SkillSafetyReport]) -> Result<(), EdenError> {
    for report in reports {
        if let Some(parent) = report.metadata_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&report.metadata_path, render_metadata_toml(report))?;
    }
    Ok(())
}

fn detect_license_status(repo_path: &Path) -> (LicenseStatus, Option<String>) {
    let Some(text) = read_license_text(repo_path) else {
        return (LicenseStatus::Unknown, None);
    };
    let normalized = text.to_lowercase();

    if normalized.contains("mit license") {
        return (LicenseStatus::Permissive, Some("MIT".to_string()));
    }
    if normalized.contains("apache license") && normalized.contains("version 2.0") {
        return (LicenseStatus::Permissive, Some("Apache-2.0".to_string()));
    }
    if normalized.contains("bsd license")
        || normalized.contains("redistribution and use in source and binary forms")
    {
        return (LicenseStatus::Permissive, Some("BSD".to_string()));
    }
    if normalized.contains("isc license") {
        return (LicenseStatus::Permissive, Some("ISC".to_string()));
    }

    (LicenseStatus::NonPermissive, None)
}

fn read_license_text(repo_path: &Path) -> Option<String> {
    const CANDIDATES: &[&str] = &[
        "LICENSE",
        "LICENSE.md",
        "LICENSE.txt",
        "COPYING",
        "COPYING.md",
        "COPYRIGHT",
    ];

    for name in CANDIDATES {
        let path = repo_path.join(name);
        if !path.exists() {
            continue;
        }
        if let Ok(content) = fs::read_to_string(path) {
            return Some(content);
        }
    }

    None
}

fn detect_risk_labels(source_path: &Path) -> Result<Vec<String>, EdenError> {
    if !source_path.exists() {
        return Ok(Vec::new());
    }

    let mut labels = BTreeSet::new();
    scan_path_for_risk(source_path, &mut labels)?;
    Ok(labels.into_iter().collect())
}

fn scan_path_for_risk(path: &Path, labels: &mut BTreeSet<String>) -> Result<(), EdenError> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Ok(());
    }

    if metadata.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            scan_path_for_risk(&entry.path(), labels)?;
        }
        return Ok(());
    }

    if metadata.is_file() {
        detect_file_risk(path, &metadata, labels)?;
    }
    Ok(())
}

fn detect_file_risk(
    path: &Path,
    _metadata: &fs::Metadata,
    labels: &mut BTreeSet<String>,
) -> Result<(), EdenError> {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_ascii_lowercase());

    match ext.as_deref() {
        Some("sh") | Some("bash") | Some("zsh") => {
            labels.insert("contains-shell-script".to_string());
        }
        Some("py") | Some("pyw") => {
            labels.insert("contains-python-script".to_string());
        }
        Some("ps1") | Some("bat") | Some("cmd") => {
            labels.insert("contains-platform-script".to_string());
        }
        _ => {}
    }

    #[cfg(unix)]
    {
        if _metadata.permissions().mode() & 0o111 != 0 {
            labels.insert("contains-executable-permissions".to_string());
        }
    }

    let mut file = fs::File::open(path)?;
    let mut head = [0u8; 4];
    let n = file.read(&mut head)?;
    let is_elf = n >= 4 && head == [0x7F, b'E', b'L', b'F'];
    let is_pe = n >= 2 && head[0] == b'M' && head[1] == b'Z';
    if is_elf || is_pe {
        labels.insert("contains-binary-artifact".to_string());
    }

    Ok(())
}

fn read_commit_sha(repo_path: &Path) -> Option<String> {
    if !repo_path.join(".git").exists() {
        return None;
    }

    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let sha = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if sha.is_empty() {
        None
    } else {
        Some(sha)
    }
}

fn unix_now() -> Result<u64, EdenError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| EdenError::Runtime(format!("system clock before unix epoch: {err}")))?;
    Ok(now.as_secs())
}

fn render_metadata_toml(report: &SkillSafetyReport) -> String {
    let mut out = String::new();
    out.push_str("version = 1\n");
    out.push_str(&format!(
        "skill_id = \"{}\"\n",
        toml_escape_str(&report.skill_id)
    ));
    out.push_str(&format!(
        "repo_path = \"{}\"\n",
        toml_escape_str(&report.repo_path.display().to_string())
    ));
    out.push_str(&format!(
        "source_path = \"{}\"\n",
        toml_escape_str(&report.source_path.display().to_string())
    ));
    out.push_str(&format!(
        "retrieved_at_unix = {}\n",
        report.retrieved_at_unix
    ));
    out.push_str(&format!(
        "license_status = \"{}\"\n",
        report.license_status.as_str()
    ));
    if let Some(hint) = &report.license_hint {
        out.push_str(&format!("license_hint = \"{}\"\n", toml_escape_str(hint)));
    }
    if let Some(commit_sha) = &report.commit_sha {
        out.push_str(&format!(
            "commit_sha = \"{}\"\n",
            toml_escape_str(commit_sha)
        ));
    }
    out.push_str(&format!(
        "no_exec_metadata_only = {}\n",
        report.no_exec_metadata_only
    ));
    out.push_str("risk_labels = [");
    out.push_str(
        &report
            .risk_labels
            .iter()
            .map(|label| format!("\"{}\"", toml_escape_str(label)))
            .collect::<Vec<_>>()
            .join(", "),
    );
    out.push_str("]\n");
    out
}

fn toml_escape_str(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
