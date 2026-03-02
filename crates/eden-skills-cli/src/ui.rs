use std::io::IsTerminal;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::time::Duration;

use clap::ValueEnum;
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
static COLOR_ENABLED_OVERRIDE: AtomicBool = AtomicBool::new(false);

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

fn env_var_present(name: &str) -> bool {
    std::env::var(name)
        .ok()
        .is_some_and(|value| !value.is_empty())
}

fn configured_color_when() -> ColorWhen {
    ColorWhen::from_u8(COLOR_WHEN_OVERRIDE.load(Ordering::Relaxed))
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
