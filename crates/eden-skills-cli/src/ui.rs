use std::io::IsTerminal;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::time::Duration;

use clap::ValueEnum;
use comfy_table::{presets, Cell, ContentArrangement, Table};
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ColorWhen {
    Auto,
    Always,
    Never,
}

impl ColorWhen {
    const fn as_u8(self) -> u8 {
        match self {
            Self::Auto => 0,
            Self::Always => 1,
            Self::Never => 2,
        }
    }

    const fn from_u8(value: u8) -> Self {
        match value {
            1 => Self::Always,
            2 => Self::Never,
            _ => Self::Auto,
        }
    }
}

static COLOR_WHEN_OVERRIDE: AtomicU8 = AtomicU8::new(ColorWhen::Auto.as_u8());
static COLOR_ENABLED_OVERRIDE: AtomicBool = AtomicBool::new(true);

pub fn configure_color_output(color_when: ColorWhen, json_mode: bool) {
    #[cfg(windows)]
    {
        enable_ansi_support::enable_ansi_support().ok();
    }

    COLOR_WHEN_OVERRIDE.store(color_when.as_u8(), Ordering::Relaxed);
    let enabled = resolve_colors_enabled(color_when, json_mode, stdout_is_tty());
    COLOR_ENABLED_OVERRIDE.store(enabled, Ordering::Relaxed);
    owo_colors::set_override(enabled);
}

pub fn color_output_enabled() -> bool {
    COLOR_ENABLED_OVERRIDE.load(Ordering::Relaxed)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusSymbol {
    Success,
    Failure,
    Skipped,
    Warning,
}

#[derive(Debug, Clone)]
pub struct UiContext {
    json_mode: bool,
    stdout_is_tty: bool,
    no_color: bool,
    force_color: bool,
    ci: bool,
    color_when: ColorWhen,
}

impl UiContext {
    pub fn from_env(json_mode: bool) -> Self {
        let stdout_is_tty = stdout_is_tty();
        Self {
            json_mode,
            stdout_is_tty,
            no_color: env_var_present("NO_COLOR"),
            force_color: env_var_present("FORCE_COLOR"),
            ci: env_var_present("CI"),
            color_when: configured_color_when(),
        }
    }

    pub fn json_mode(&self) -> bool {
        self.json_mode
    }

    pub fn colors_enabled(&self) -> bool {
        if self.json_mode {
            return false;
        }
        match self.color_when {
            ColorWhen::Never => false,
            ColorWhen::Always => true,
            ColorWhen::Auto => {
                if self.no_color {
                    return false;
                }
                if self.force_color {
                    return true;
                }
                if self.ci {
                    return false;
                }
                self.stdout_is_tty
            }
        }
    }

    pub fn symbols_enabled(&self) -> bool {
        let force_symbols = matches!(self.color_when, ColorWhen::Always) || self.force_color;
        !self.json_mode && (self.stdout_is_tty || force_symbols) && !self.ci
    }

    pub fn spinner_enabled(&self) -> bool {
        !self.json_mode && self.stdout_is_tty && !self.ci
    }

    pub fn interactive_enabled(&self) -> bool {
        !self.json_mode && self.stdout_is_tty && !self.ci
    }

    pub fn status_symbol(&self, symbol: StatusSymbol) -> String {
        let raw = match symbol {
            StatusSymbol::Success => "✓",
            StatusSymbol::Failure => "✗",
            StatusSymbol::Skipped => "·",
            StatusSymbol::Warning => "!",
        };
        if !self.colors_enabled() {
            return raw.to_string();
        }
        match symbol {
            StatusSymbol::Success => raw.green().to_string(),
            StatusSymbol::Failure => raw.red().to_string(),
            StatusSymbol::Skipped => raw.dimmed().to_string(),
            StatusSymbol::Warning => raw.yellow().to_string(),
        }
    }

    pub fn action_prefix(&self, action: &str) -> String {
        let padded = format!("{action:>8}");
        if self.colors_enabled() {
            padded.cyan().bold().to_string()
        } else {
            padded
        }
    }

    pub fn table(&self, headers: &[&str]) -> Table {
        let mut table = Table::new();
        let human_tty = self.stdout_is_tty && !self.ci;
        if human_tty {
            table.load_preset(presets::UTF8_FULL_CONDENSED);
        } else {
            table.load_preset(presets::ASCII_FULL_CONDENSED);
            table.set_width(80);
        }
        table.set_content_arrangement(ContentArrangement::Dynamic);

        let style_headers = human_tty && self.colors_enabled();
        let header_cells = headers
            .iter()
            .map(|header| {
                let content = if style_headers {
                    (*header).bold().to_string()
                } else {
                    (*header).to_string()
                };
                Cell::new(content)
            })
            .collect::<Vec<_>>();
        table.set_header(header_cells);
        table
    }

    pub fn spinner(&self, action: &str, detail: String) -> UiSpinner {
        if !self.spinner_enabled() {
            return UiSpinner {
                action: action.to_string(),
                detail,
                progress: None,
            };
        }

        let progress = ProgressBar::new_spinner();
        let style = ProgressStyle::with_template("{prefix}  {msg} {spinner}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner())
            .tick_strings(&["-", "\\", "|", "/"]);
        progress.set_style(style);
        progress.set_prefix(self.action_prefix(action));
        progress.set_message(detail.clone());
        progress.enable_steady_tick(Duration::from_millis(100));

        UiSpinner {
            action: action.to_string(),
            detail,
            progress: Some(progress),
        }
    }
}

#[derive(Debug)]
pub struct UiSpinner {
    action: String,
    detail: String,
    progress: Option<ProgressBar>,
}

impl UiSpinner {
    pub fn finish_success(self, ui: &UiContext) {
        if let Some(progress) = self.progress {
            progress.finish_and_clear();
            println!(
                "{}  {} {} done",
                ui.action_prefix(&self.action),
                self.detail,
                ui.status_symbol(StatusSymbol::Success)
            );
        }
    }

    pub fn finish_failure(self, ui: &UiContext, summary: &str) {
        if let Some(progress) = self.progress {
            progress.finish_and_clear();
            println!(
                "{}  {} {} {}",
                ui.action_prefix(&self.action),
                self.detail,
                ui.status_symbol(StatusSymbol::Failure),
                summary
            );
        }
    }
}

pub fn abbreviate_home_path(path: &str) -> String {
    let Some(home_dir) = resolve_home_dir() else {
        return path.to_string();
    };
    let home_dir = home_dir.trim_end_matches(['/', '\\']);
    if home_dir.is_empty() {
        return path.to_string();
    }

    if path == home_dir {
        return "~".to_string();
    }
    if let Some(remainder) = path.strip_prefix(home_dir) {
        if remainder.starts_with('/') || remainder.starts_with('\\') {
            return format!("~{remainder}");
        }
    }

    let normalized_home = home_dir.replace('\\', "/");
    if normalized_home != home_dir {
        let normalized_path = path.replace('\\', "/");
        if normalized_path == normalized_home {
            return "~".to_string();
        }
        if let Some(remainder) = normalized_path.strip_prefix(&normalized_home) {
            if remainder.starts_with('/') {
                return format!("~{remainder}");
            }
        }
    }

    path.to_string()
}

pub fn abbreviate_repo_url(url: &str) -> String {
    let remainder = if let Some(rest) = url.strip_prefix("https://github.com/") {
        rest
    } else if let Some(rest) = url.strip_prefix("http://github.com/") {
        rest
    } else if let Some(rest) = url.strip_prefix("git@github.com:") {
        rest
    } else {
        return url.to_string();
    };

    let path = remainder
        .split(['?', '#'])
        .next()
        .unwrap_or(remainder)
        .trim_end_matches('/');
    let mut parts = path.split('/');
    let Some(owner) = parts.next() else {
        return url.to_string();
    };
    let Some(repo_raw) = parts.next() else {
        return url.to_string();
    };
    if owner.is_empty() || repo_raw.is_empty() || parts.next().is_some() {
        return url.to_string();
    }

    let repo = repo_raw.strip_suffix(".git").unwrap_or(repo_raw);
    if repo.is_empty() {
        return url.to_string();
    }
    format!("{owner}/{repo}")
}

fn env_var_present(name: &str) -> bool {
    std::env::var(name)
        .ok()
        .is_some_and(|value| !value.is_empty())
}

fn configured_color_when() -> ColorWhen {
    ColorWhen::from_u8(COLOR_WHEN_OVERRIDE.load(Ordering::Relaxed))
}

fn resolve_home_dir() -> Option<String> {
    std::env::var("HOME")
        .ok()
        .filter(|value| !value.is_empty())
        .or_else(|| {
            std::env::var("USERPROFILE")
                .ok()
                .filter(|value| !value.is_empty())
        })
}

fn resolve_colors_enabled(color_when: ColorWhen, json_mode: bool, stdout_is_tty: bool) -> bool {
    if json_mode {
        return false;
    }
    match color_when {
        ColorWhen::Never => false,
        ColorWhen::Always => true,
        ColorWhen::Auto => {
            let no_color = env_var_present("NO_COLOR");
            if no_color {
                return false;
            }
            let force_color = env_var_present("FORCE_COLOR");
            if force_color {
                return true;
            }
            let ci = env_var_present("CI");
            if ci {
                return false;
            }
            stdout_is_tty
        }
    }
}

fn stdout_is_tty() -> bool {
    forced_tty_for_tests() || std::io::stdout().is_terminal()
}

fn forced_tty_for_tests() -> bool {
    std::env::var("EDEN_SKILLS_FORCE_TTY")
        .ok()
        .is_some_and(|value| value == "1")
}
