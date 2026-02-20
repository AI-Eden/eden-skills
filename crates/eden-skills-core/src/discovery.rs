use std::fs;
use std::path::{Path, PathBuf};

use crate::error::EdenError;
use serde::Deserialize;

const DISCOVERY_PARENT_DIRS: &[&str] = &[
    "skills",
    "packages",
    "skills/.curated",
    "skills/.experimental",
    "skills/.system",
    ".agents/skills",
    ".agent/skills",
    ".augment/skills",
    ".claude/skills",
    ".cline/skills",
    ".codebuddy/skills",
    ".commandcode/skills",
    ".continue/skills",
    ".cortex/skills",
    ".crush/skills",
    ".factory/skills",
    ".goose/skills",
    ".junie/skills",
    ".iflow/skills",
    ".kilocode/skills",
    ".kiro/skills",
    ".kode/skills",
    ".mcpjam/skills",
    ".vibe/skills",
    ".mux/skills",
    ".openhands/skills",
    ".pi/skills",
    ".qoder/skills",
    ".qwen/skills",
    ".roo/skills",
    ".trae/skills",
    ".windsurf/skills",
    ".zencoder/skills",
    ".neovate/skills",
    ".pochi/skills",
    ".adal/skills",
];
const MAX_RECURSIVE_DISCOVERY_DEPTH: usize = 6;
const MAX_RECURSIVE_DISCOVERY_RESULTS: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredSkill {
    pub name: String,
    pub description: String,
    pub subpath: String,
}

pub fn discover_skills(root: &Path) -> Result<Vec<DiscoveredSkill>, EdenError> {
    let mut discovered = Vec::new();

    let root_skill = root.join("SKILL.md");
    if root_skill.is_file() {
        discovered.push(parse_skill_markdown(&root_skill, ".", root)?);
    }

    for parent_dir in DISCOVERY_PARENT_DIRS {
        discovered.extend(discover_directory_children(root, parent_dir)?);
    }
    discovered.extend(discover_from_plugin_manifests(root)?);
    dedupe_by_subpath(&mut discovered);

    if discovered.is_empty() {
        discovered.extend(discover_recursive_fallback(root)?);
        dedupe_by_subpath(&mut discovered);
    }

    Ok(discovered)
}

fn discover_directory_children(
    root: &Path,
    directory_name: &str,
) -> Result<Vec<DiscoveredSkill>, EdenError> {
    let parent = root.join(directory_name);
    if !parent.is_dir() {
        return Ok(Vec::new());
    }

    let mut child_dirs = Vec::new();
    for entry in fs::read_dir(&parent)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            child_dirs.push(entry.path());
        }
    }
    child_dirs.sort();

    let mut discovered = Vec::new();
    for child in child_dirs {
        let skill_file = child.join("SKILL.md");
        if !skill_file.is_file() {
            continue;
        }
        let subpath = child
            .strip_prefix(root)
            .map_err(|err| EdenError::Runtime(format!("failed to compute relative path: {err}")))?
            .to_string_lossy()
            .replace('\\', "/");
        discovered.push(parse_skill_markdown(&skill_file, &subpath, &child)?);
    }
    Ok(discovered)
}

fn parse_skill_markdown(
    file_path: &Path,
    subpath: &str,
    fallback_dir: &Path,
) -> Result<DiscoveredSkill, EdenError> {
    let content = fs::read_to_string(file_path)?;
    let (name, description) = parse_frontmatter_name_description(&content);

    let fallback_name = fallback_dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("skill")
        .to_string();

    Ok(DiscoveredSkill {
        name: name.unwrap_or(fallback_name),
        description: description.unwrap_or_default(),
        subpath: subpath.to_string(),
    })
}

fn parse_frontmatter_name_description(content: &str) -> (Option<String>, Option<String>) {
    let mut lines = content.lines();
    if lines.next().map(str::trim) != Some("---") {
        return (None, None);
    }

    let mut name = None;
    let mut description = None;
    for line in lines {
        let trimmed = line.trim();
        if trimmed == "---" {
            break;
        }
        let Some((key, value)) = trimmed.split_once(':') else {
            continue;
        };
        let normalized = value
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .to_string();
        match key.trim() {
            "name" => {
                if !normalized.is_empty() {
                    name = Some(normalized);
                }
            }
            "description" => {
                if !normalized.is_empty() {
                    description = Some(normalized);
                }
            }
            _ => {}
        }
    }
    (name, description)
}

fn dedupe_by_subpath(discovered: &mut Vec<DiscoveredSkill>) {
    let mut seen_subpaths = std::collections::HashSet::new();
    discovered.retain(|skill| seen_subpaths.insert(skill.subpath.clone()));
}

#[derive(Debug, Deserialize, Default)]
struct ClaudePluginManifest {
    #[serde(default)]
    metadata: ClaudePluginMetadata,
    #[serde(default)]
    plugins: Vec<ClaudePluginEntry>,
    source: Option<String>,
    #[serde(default)]
    skills: Vec<String>,
}

#[derive(Debug, Deserialize, Default)]
struct ClaudePluginMetadata {
    #[serde(rename = "pluginRoot")]
    plugin_root: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct ClaudePluginEntry {
    source: Option<String>,
    #[serde(default)]
    skills: Vec<String>,
}

fn discover_from_plugin_manifests(root: &Path) -> Result<Vec<DiscoveredSkill>, EdenError> {
    let mut discovered = Vec::new();
    for manifest_name in ["marketplace.json", "plugin.json"] {
        let manifest_path = root.join(".claude-plugin").join(manifest_name);
        if !manifest_path.is_file() {
            continue;
        }
        discovered.extend(discover_from_plugin_manifest(root, &manifest_path)?);
    }
    Ok(discovered)
}

fn discover_from_plugin_manifest(
    root: &Path,
    manifest_path: &Path,
) -> Result<Vec<DiscoveredSkill>, EdenError> {
    let raw = fs::read_to_string(manifest_path)?;
    let parsed = match serde_json::from_str::<ClaudePluginManifest>(&raw) {
        Ok(manifest) => manifest,
        Err(_) => return Ok(Vec::new()),
    };

    let mut entries = parsed.plugins;
    if entries.is_empty() && !parsed.skills.is_empty() {
        entries.push(ClaudePluginEntry {
            source: parsed.source,
            skills: parsed.skills,
        });
    }

    let plugin_root = normalize_manifest_relative_path(parsed.metadata.plugin_root.as_deref());
    let mut discovered = Vec::new();
    for entry in entries {
        discovered.extend(discover_plugin_entry_skills(
            root,
            &plugin_root,
            entry.source.as_deref(),
            &entry.skills,
        )?);
    }
    Ok(discovered)
}

fn discover_plugin_entry_skills(
    root: &Path,
    plugin_root: &str,
    source: Option<&str>,
    skills: &[String],
) -> Result<Vec<DiscoveredSkill>, EdenError> {
    let mut discovered = Vec::new();
    for skill_path in skills {
        if skill_path.trim().is_empty() {
            continue;
        }

        let mut candidates = vec![root.join(normalize_manifest_relative_path(Some(skill_path)))];
        if let Some(source) = source.filter(|value| !value.trim().is_empty()) {
            candidates.push(
                root.join(plugin_root)
                    .join(normalize_manifest_relative_path(Some(source)))
                    .join(normalize_manifest_relative_path(Some(skill_path))),
            );
        }

        for candidate in candidates {
            let Some(skill_md) = resolve_skill_markdown_path(&candidate) else {
                continue;
            };
            let Some(skill_dir) = skill_md.parent() else {
                continue;
            };
            let Ok(relative_skill_dir) = skill_dir.strip_prefix(root) else {
                continue;
            };
            let subpath = relative_skill_dir.to_string_lossy().replace('\\', "/");
            discovered.push(parse_skill_markdown(&skill_md, &subpath, skill_dir)?);
            break;
        }
    }
    Ok(discovered)
}

fn normalize_manifest_relative_path(path: Option<&str>) -> String {
    path.unwrap_or(".")
        .trim()
        .trim_start_matches("./")
        .trim_start_matches('/')
        .replace('\\', "/")
}

fn resolve_skill_markdown_path(path: &Path) -> Option<PathBuf> {
    if path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.eq_ignore_ascii_case("SKILL.md"))
        && path.is_file()
    {
        return Some(path.to_path_buf());
    }
    let candidate = path.join("SKILL.md");
    if candidate.is_file() {
        return Some(candidate);
    }
    None
}

fn discover_recursive_fallback(root: &Path) -> Result<Vec<DiscoveredSkill>, EdenError> {
    let mut discovered = Vec::new();
    walk_recursive_for_skill_markdown(root, root, 0, &mut discovered)?;
    Ok(discovered)
}

fn walk_recursive_for_skill_markdown(
    root: &Path,
    dir: &Path,
    depth: usize,
    discovered: &mut Vec<DiscoveredSkill>,
) -> Result<(), EdenError> {
    if depth > MAX_RECURSIVE_DISCOVERY_DEPTH || discovered.len() >= MAX_RECURSIVE_DISCOVERY_RESULTS
    {
        return Ok(());
    }

    let mut entries = fs::read_dir(dir)?.collect::<Result<Vec<_>, std::io::Error>>()?;
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        if discovered.len() >= MAX_RECURSIVE_DISCOVERY_RESULTS {
            break;
        }
        let file_type = entry.file_type()?;
        if !file_type.is_dir() {
            continue;
        }

        let child = entry.path();
        if child
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name == ".git")
        {
            continue;
        }

        let skill_md = child.join("SKILL.md");
        if skill_md.is_file() {
            let subpath = child
                .strip_prefix(root)
                .map_err(|err| {
                    EdenError::Runtime(format!("failed to compute recursive relative path: {err}"))
                })?
                .to_string_lossy()
                .replace('\\', "/");
            discovered.push(parse_skill_markdown(&skill_md, &subpath, &child)?);
        }

        walk_recursive_for_skill_markdown(root, &child, depth + 1, discovered)?;
    }

    Ok(())
}
