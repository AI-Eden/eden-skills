use std::io::IsTerminal;
use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};

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
}

impl UiContext {
    pub fn from_env(json_mode: bool) -> Self {
        let stdout_is_tty = forced_tty_for_tests() || std::io::stdout().is_terminal();
        let no_color = env_var_present("NO_COLOR");
        let force_color = env_var_present("FORCE_COLOR");
        let ci = env_var_present("CI");
        let colors_enabled = if json_mode || no_color {
            false
        } else if force_color {
            true
        } else if ci {
            false
        } else {
            stdout_is_tty
        };
        console::set_colors_enabled(colors_enabled);
        console::set_colors_enabled_stderr(colors_enabled);
        Self {
            json_mode,
            stdout_is_tty,
            no_color,
            force_color,
            ci,
        }
    }

    pub fn json_mode(&self) -> bool {
        self.json_mode
    }

    pub fn colors_enabled(&self) -> bool {
        if self.json_mode || self.no_color {
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

    pub fn symbols_enabled(&self) -> bool {
        !self.json_mode && (self.stdout_is_tty || self.force_color) && !self.ci
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
            StatusSymbol::Success => format!("\u{1b}[32m{raw}\u{1b}[0m"),
            StatusSymbol::Failure => format!("\u{1b}[31m{raw}\u{1b}[0m"),
            StatusSymbol::Skipped => format!("\u{1b}[2m{raw}\u{1b}[0m"),
            StatusSymbol::Warning => format!("\u{1b}[33m{raw}\u{1b}[0m"),
        }
    }

    pub fn action_prefix(&self, action: &str) -> String {
        let padded = format!("{action:>8}");
        if self.colors_enabled() {
            format!("\u{1b}[1;36m{padded}\u{1b}[0m")
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

fn forced_tty_for_tests() -> bool {
    std::env::var("EDEN_SKILLS_FORCE_TTY")
        .ok()
        .is_some_and(|value| value == "1")
}
