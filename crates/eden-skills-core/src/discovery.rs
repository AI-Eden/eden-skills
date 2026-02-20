use std::fs;
use std::path::Path;

use crate::error::EdenError;

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

    discovered.extend(discover_directory_children(root, "skills")?);
    discovered.extend(discover_directory_children(root, "packages")?);
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
