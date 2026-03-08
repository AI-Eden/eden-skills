//! Remote and local skill discovery via temporary git checkouts.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use eden_skills_core::discovery::{discover_skills, DiscoveredSkill};
use eden_skills_core::error::EdenError;
use eden_skills_core::paths::normalize_lexical;
use eden_skills_core::source::resolve_repo_cache_root;
use owo_colors::OwoColorize;

use crate::ui::{
    prompt_skill_multi_select, SkillSelectItem, SkillSelectOutcome, StatusSymbol, UiContext,
};

use crate::commands::clean::DISCOVERY_TEMP_DIR_PREFIX;
use crate::commands::common::run_git_command;

#[derive(Debug)]
pub(super) struct RemoteDiscoveryResult {
    pub(super) discovered: Vec<DiscoveredSkill>,
    pub(super) temp_checkout: Option<TempDiscoveryCheckout>,
}

#[derive(Debug)]
pub(super) struct TempDiscoveryCheckout {
    path: Option<PathBuf>,
}

impl TempDiscoveryCheckout {
    pub(super) fn path(&self) -> &Path {
        self.path
            .as_deref()
            .expect("temp discovery checkout path should be present")
    }

    pub(super) fn disarm(&mut self) {
        self.path = None;
    }
}

impl Drop for TempDiscoveryCheckout {
    fn drop(&mut self) {
        if let Some(path) = &self.path {
            let _ = fs::remove_dir_all(path);
        }
    }
}

pub(super) async fn discover_remote_skills_via_temp_clone(
    repo_url: &str,
    reference: &str,
    scoped_subpath: &str,
) -> Result<RemoteDiscoveryResult, EdenError> {
    let repo_url = repo_url.to_string();
    let reference = reference.to_string();
    let scoped_subpath = scoped_subpath.to_string();
    tokio::task::spawn_blocking(move || {
        discover_remote_skills_via_temp_clone_blocking(&repo_url, &reference, &scoped_subpath)
    })
    .await
    .map_err(|err| EdenError::Runtime(format!("remote discovery worker failed: {err}")))?
}

fn discover_remote_skills_via_temp_clone_blocking(
    repo_url: &str,
    reference: &str,
    scoped_subpath: &str,
) -> Result<RemoteDiscoveryResult, EdenError> {
    let temp_checkout = create_discovery_temp_checkout()?;
    clone_repo_for_discovery(repo_url, reference, temp_checkout.path())?;
    let discovery_root = normalize_lexical(&temp_checkout.path().join(scoped_subpath));
    if !discovery_root.exists() {
        return Err(EdenError::Runtime(format!(
            "discovery path does not exist: {}",
            discovery_root.display()
        )));
    }
    Ok(RemoteDiscoveryResult {
        discovered: discover_skills(&discovery_root)?,
        temp_checkout: Some(temp_checkout),
    })
}

fn create_discovery_temp_checkout() -> Result<TempDiscoveryCheckout, EdenError> {
    for attempt in 0..10u32 {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| EdenError::Runtime(format!("system clock before unix epoch: {err}")))?
            .as_nanos();
        let candidate = std::env::temp_dir().join(format!(
            "{DISCOVERY_TEMP_DIR_PREFIX}{}-{unique}-{attempt}",
            std::process::id()
        ));
        match fs::create_dir(&candidate) {
            Ok(()) => {
                return Ok(TempDiscoveryCheckout {
                    path: Some(candidate),
                })
            }
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(err) => return Err(EdenError::Io(err)),
        }
    }
    Err(EdenError::Runtime(
        "failed to create temporary directory for remote discovery".to_string(),
    ))
}

pub(super) fn seed_repo_cache_from_discovery_checkout(
    temp_checkout: Option<TempDiscoveryCheckout>,
    storage_root: &Path,
    repo_url: &str,
    reference: &str,
) -> Result<(), EdenError> {
    let Some(mut temp_checkout) = temp_checkout else {
        return Ok(());
    };

    let cache_root = resolve_repo_cache_root(storage_root, repo_url, reference);
    if cache_root.exists() {
        return Ok(());
    }
    if let Some(parent) = cache_root.parent() {
        fs::create_dir_all(parent)?;
    }

    if let Ok(()) = rename_discovery_checkout_for_cache(temp_checkout.path(), &cache_root) {
        temp_checkout.disarm()
    }
    Ok(())
}

fn rename_discovery_checkout_for_cache(from: &Path, to: &Path) -> std::io::Result<()> {
    if std::env::var("EDEN_SKILLS_TEST_FORCE_DISCOVERY_RENAME_FAIL")
        .ok()
        .as_deref()
        == Some("1")
    {
        return Err(std::io::Error::other(
            "forced discovery checkout rename failure",
        ));
    }
    fs::rename(from, to)
}

fn clone_repo_for_discovery(
    repo_url: &str,
    reference: &str,
    repo_dir: &Path,
) -> Result<(), EdenError> {
    if let Some(parent) = repo_dir.parent() {
        fs::create_dir_all(parent)?;
    }

    record_test_git_clone_if_configured();
    let branch_clone_result = run_git_command(
        Command::new("git")
            .arg("clone")
            .arg("--depth")
            .arg("1")
            .arg("--branch")
            .arg(reference)
            .arg(repo_url)
            .arg(repo_dir),
        &format!(
            "clone `{repo_url}` into `{}` with ref `{reference}`",
            repo_dir.display()
        ),
    );

    if let Err(branch_error) = branch_clone_result {
        record_test_git_clone_if_configured();
        let fallback_clone = run_git_command(
            Command::new("git").arg("clone").arg(repo_url).arg(repo_dir),
            &format!(
                "clone `{repo_url}` into `{}` without branch hint",
                repo_dir.display()
            ),
        );
        if let Err(fallback_error) = fallback_clone {
            return Err(EdenError::Runtime(format!(
                "branch clone attempt failed: {branch_error}; fallback clone attempt failed: {fallback_error}"
            )));
        }
        run_git_command(
            Command::new("git")
                .arg("-C")
                .arg(repo_dir)
                .arg("checkout")
                .arg(reference),
            &format!(
                "checkout ref `{reference}` in temporary discovery repo `{}`",
                repo_dir.display()
            ),
        )
        .map_err(EdenError::Runtime)?;
    }

    Ok(())
}

fn record_test_git_clone_if_configured() {
    let Some(log_path) = std::env::var_os("EDEN_SKILLS_TEST_GIT_CLONE_LOG") else {
        return;
    };
    let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
    else {
        return;
    };
    let _ = std::io::Write::write_all(&mut file, b"clone\n");
}

pub(super) fn resolve_local_install_selection(
    discovered: &[DiscoveredSkill],
    all: bool,
    named: &[String],
    yes: bool,
    ui: &UiContext,
) -> Result<Vec<DiscoveredSkill>, EdenError> {
    if all || yes {
        return Ok(discovered.to_vec());
    }

    if !named.is_empty() {
        return select_named_skills(discovered, named);
    }

    if discovered.len() == 1 {
        return Ok(vec![discovered[0].clone()]);
    }

    if !ui.interactive_enabled() {
        return Ok(discovered.to_vec());
    }

    let items = discovered
        .iter()
        .map(|skill| SkillSelectItem {
            name: skill.name.as_str(),
            description: skill.description.as_str(),
        })
        .collect::<Vec<_>>();

    println!();

    let indices = match prompt_skill_multi_select(
        ui,
        "Select skills to install",
        &items,
        "EDEN_SKILLS_TEST_SKILL_INPUT",
        Some(format!(
            "Found {} skill{}",
            discovered.len().to_string().magenta(),
            if discovered.len() == 1 { "" } else { "s" }
        )),
    )? {
        SkillSelectOutcome::Cancelled => {
            print_install_cancelled(ui);
            return Ok(Vec::new());
        }
        SkillSelectOutcome::Interrupted => {
            print_install_interrupted(ui);
            return Ok(Vec::new());
        }
        SkillSelectOutcome::Selected(indices) => indices,
    };
    if indices.is_empty() {
        return Err(EdenError::InvalidArguments(
            "no skills selected".to_string(),
        ));
    }
    Ok(indices
        .into_iter()
        .map(|index| discovered[index].clone())
        .collect())
}

fn select_named_skills(
    discovered: &[DiscoveredSkill],
    names: &[String],
) -> Result<Vec<DiscoveredSkill>, EdenError> {
    let mut selected = Vec::new();
    let mut unknown = Vec::new();
    for name in names {
        if let Some(skill) = discovered.iter().find(|skill| skill.name == *name) {
            if !selected
                .iter()
                .any(|existing: &DiscoveredSkill| existing.name == skill.name)
            {
                selected.push(skill.clone());
            }
        } else {
            unknown.push(name.clone());
        }
    }

    if !unknown.is_empty() {
        let available = discovered
            .iter()
            .map(|skill| skill.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        return Err(EdenError::InvalidArguments(format!(
            "unknown skill name(s): {}; available: {}",
            unknown.join(", "),
            available
        )));
    }

    Ok(selected)
}

pub(super) fn print_install_cancelled(ui: &UiContext) {
    if !ui.json_mode() {
        println!(
            "  {} Install cancelled",
            ui.status_symbol(StatusSymbol::Skipped)
        );
    }
}

pub(super) fn print_install_interrupted(ui: &UiContext) {
    if !ui.json_mode() {
        println!();
        println!("{}", ui.signal_cancelled_line("Install"));
    }
}

pub(super) fn print_discovery_preview(
    ui: &UiContext,
    skills: &[DiscoveredSkill],
    show_all: bool,
) {
    const MAX_DISPLAY: usize = 8;
    println!(
        "{}  {} skills in repository:",
        ui.action_prefix("Found"),
        skills.len()
    );
    if skills.is_empty() {
        println!();
        println!("  (no SKILL.md discovered)");
        return;
    }
    println!();

    let display_len = if show_all {
        skills.len()
    } else {
        skills.len().min(MAX_DISPLAY)
    };
    let display_skills = &skills[..display_len];
    let number_width = skills.len().to_string().len().max(1);
    let leading_spaces = 5usize.saturating_sub(number_width);
    let description_indent = " ".repeat(leading_spaces + number_width + 2);
    let description_wrap_width = discovery_description_wrap_width(description_indent.len());

    for (index, skill) in display_skills.iter().enumerate() {
        let number_prefix = format!(
            "{}{:>width$}. ",
            " ".repeat(leading_spaces),
            index + 1,
            width = number_width
        );
        println!(
            "{number_prefix}{}",
            super::output::style_skill_id(ui, &skill.name)
        );

        if !skill.description.trim().is_empty() {
            for line in wrap_discovery_description(&skill.description, description_wrap_width) {
                let rendered = if ui.colors_enabled() {
                    line.dimmed().to_string()
                } else {
                    line
                };
                println!("{description_indent}{rendered}");
            }
        }
    }

    if !show_all && skills.len() > MAX_DISPLAY {
        println!();
        let footer = format!(
            "  ... and {} more (use --list to see all)",
            skills.len() - MAX_DISPLAY
        );
        if ui.colors_enabled() {
            println!("{}", footer.dimmed());
        } else {
            println!("{footer}");
        }
    }

    println!();
}

pub(super) fn print_discovery_json(skills: &[DiscoveredSkill]) -> Result<(), EdenError> {
    let payload = skills
        .iter()
        .map(|skill| {
            serde_json::json!({
                "name": &skill.name,
                "description": &skill.description,
                "subpath": &skill.subpath,
            })
        })
        .collect::<Vec<_>>();
    let encoded = serde_json::to_string_pretty(&payload)
        .map_err(|err| EdenError::Runtime(format!("failed to encode install list json: {err}")))?;
    println!("{encoded}");
    Ok(())
}

fn discovery_description_wrap_width(indent_len: usize) -> usize {
    let terminal_width = std::env::var("COLUMNS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|width| *width > indent_len)
        .unwrap_or(80);
    terminal_width.saturating_sub(indent_len).max(1)
}

fn wrap_discovery_description(description: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for paragraph in description.lines() {
        let trimmed = paragraph.trim();
        if trimmed.is_empty() {
            continue;
        }
        let mut current = String::new();
        for word in trimmed.split_whitespace() {
            if current.is_empty() {
                current.push_str(word);
                continue;
            }
            let proposed_len = current.chars().count() + 1 + word.chars().count();
            if proposed_len <= width {
                current.push(' ');
                current.push_str(word);
            } else {
                lines.push(current);
                current = word.to_string();
            }
        }
        if !current.is_empty() {
            lines.push(current);
        }
    }
    if lines.is_empty() && !description.trim().is_empty() {
        lines.push(description.trim().to_string());
    }
    lines
}

pub(super) fn join_scoped_subpath(scope_subpath: &str, discovered_subpath: &str) -> String {
    if scope_subpath == "." {
        return discovered_subpath.to_string();
    }
    if discovered_subpath == "." {
        return scope_subpath.to_string();
    }
    format!("{scope_subpath}/{discovered_subpath}")
}
