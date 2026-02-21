use std::path::Path;

use crate::error::EdenError;
use crate::paths::resolve_path_string;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UrlSourceType {
    LocalPath,
    GitHubTree,
    FullUrl,
    SshUrl,
    GitHubShorthand,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UrlInstallSource {
    pub source_type: UrlSourceType,
    pub repo: String,
    pub reference: Option<String>,
    pub subpath: Option<String>,
    pub is_local: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetectedInstallSource {
    Url(UrlInstallSource),
    RegistryName(String),
}

pub fn detect_install_source(input: &str, cwd: &Path) -> Result<DetectedInstallSource, EdenError> {
    if looks_like_local_path(input) {
        let resolved = resolve_path_string(input, cwd)?;
        return Ok(DetectedInstallSource::Url(UrlInstallSource {
            source_type: UrlSourceType::LocalPath,
            repo: resolved.display().to_string(),
            reference: None,
            subpath: None,
            is_local: true,
        }));
    }

    if let Some(tree_source) = parse_github_tree_url(input) {
        return Ok(DetectedInstallSource::Url(tree_source));
    }

    if input.contains("://") {
        return Ok(DetectedInstallSource::Url(UrlInstallSource {
            source_type: UrlSourceType::FullUrl,
            repo: normalize_full_url(input),
            reference: None,
            subpath: None,
            is_local: false,
        }));
    }

    if is_ssh_url(input) {
        return Ok(DetectedInstallSource::Url(UrlInstallSource {
            source_type: UrlSourceType::SshUrl,
            repo: input.to_string(),
            reference: None,
            subpath: None,
            is_local: false,
        }));
    }

    if let Some((owner, repo)) = parse_github_shorthand(input) {
        return Ok(DetectedInstallSource::Url(UrlInstallSource {
            source_type: UrlSourceType::GitHubShorthand,
            repo: format!("https://github.com/{owner}/{repo}.git"),
            reference: None,
            subpath: None,
            is_local: false,
        }));
    }

    Ok(DetectedInstallSource::RegistryName(input.to_string()))
}

pub fn derive_skill_id_from_source_repo(source_repo: &str) -> Result<String, EdenError> {
    if Path::new(source_repo).is_absolute() {
        let path = Path::new(source_repo);
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            return Err(EdenError::InvalidArguments(format!(
                "failed to derive skill id from local path `{source_repo}`"
            )));
        };
        if name.trim().is_empty() {
            return Err(EdenError::InvalidArguments(format!(
                "failed to derive skill id from local path `{source_repo}`"
            )));
        }
        return Ok(name.to_string());
    }

    let tail = if let Some(rest) = source_repo.strip_prefix("git@") {
        rest.rsplit(':').next().unwrap_or(rest)
    } else {
        source_repo.rsplit('/').next().unwrap_or(source_repo)
    };
    let candidate = tail.trim_end_matches(".git").trim_end_matches('/');
    if candidate.trim().is_empty() {
        return Err(EdenError::InvalidArguments(format!(
            "failed to derive skill id from source `{source_repo}`"
        )));
    }
    Ok(candidate.to_string())
}

fn looks_like_local_path(input: &str) -> bool {
    input.starts_with("./")
        || input.starts_with("../")
        || input.starts_with('/')
        || input.starts_with('~')
        || std::path::Path::new(input).is_absolute()
}

fn parse_github_tree_url(input: &str) -> Option<UrlInstallSource> {
    const PREFIX: &str = "https://github.com/";
    if !input.starts_with(PREFIX) {
        return None;
    }
    let path = &input[PREFIX.len()..];
    let segments = path.split('/').collect::<Vec<_>>();
    if segments.len() < 5 {
        return None;
    }
    if segments[2] != "tree" {
        return None;
    }

    let owner = segments[0];
    let repo = segments[1];
    let reference = segments[3];
    let subpath = segments[4..].join("/");
    if owner.trim().is_empty()
        || repo.trim().is_empty()
        || reference.trim().is_empty()
        || subpath.trim().is_empty()
    {
        return None;
    }

    Some(UrlInstallSource {
        source_type: UrlSourceType::GitHubTree,
        repo: format!("https://github.com/{owner}/{repo}.git"),
        reference: Some(reference.to_string()),
        subpath: Some(subpath),
        is_local: false,
    })
}

fn is_ssh_url(input: &str) -> bool {
    input.starts_with("git@")
        && input.contains(':')
        && input.ends_with(".git")
        && !input.contains(char::is_whitespace)
}

fn parse_github_shorthand(input: &str) -> Option<(&str, &str)> {
    if input.contains("://")
        || input.starts_with("git@")
        || input.contains(char::is_whitespace)
        || input.starts_with("./")
        || input.starts_with("../")
        || input.starts_with('/')
        || input.starts_with('~')
    {
        return None;
    }

    let mut parts = input.split('/');
    let owner = parts.next()?;
    let repo = parts.next()?;
    if parts.next().is_some() || owner.is_empty() || repo.is_empty() {
        return None;
    }
    Some((owner, repo))
}

fn normalize_full_url(input: &str) -> String {
    let trimmed = input.trim_end_matches('/');
    let is_github_or_gitlab =
        trimmed.starts_with("https://github.com/") || trimmed.starts_with("https://gitlab.com/");
    if is_github_or_gitlab && !trimmed.ends_with(".git") {
        return format!("{trimmed}.git");
    }
    trimmed.to_string()
}
