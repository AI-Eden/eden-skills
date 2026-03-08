//! Color configuration and global color output state.
//!
//! Manages the `--color` flag, `NO_COLOR`/`FORCE_COLOR` env vars, and
//! JSON mode to determine whether ANSI escape sequences should be emitted.

use std::io::IsTerminal;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use clap::ValueEnum;
use dialoguer::console::{set_colors_enabled, set_colors_enabled_stderr};

/// When to emit ANSI color sequences.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ColorWhen {
    Auto,
    Always,
    Never,
}

impl ColorWhen {
    pub(crate) const fn as_u8(self) -> u8 {
        match self {
            Self::Auto => 0,
            Self::Always => 1,
            Self::Never => 2,
        }
    }

    pub(crate) const fn from_u8(value: u8) -> Self {
        match value {
            1 => Self::Always,
            2 => Self::Never,
            _ => Self::Auto,
        }
    }
}

static COLOR_WHEN_OVERRIDE: AtomicU8 = AtomicU8::new(ColorWhen::Auto.as_u8());
static COLOR_ENABLED_OVERRIDE: AtomicBool = AtomicBool::new(true);

/// Initialize the global color override from the `--color` flag and JSON mode.
///
/// Must be called once during CLI startup before any output is produced.
pub fn configure_color_output(color_when: ColorWhen, json_mode: bool) {
    #[cfg(windows)]
    {
        enable_ansi_support::enable_ansi_support().ok();
    }

    COLOR_WHEN_OVERRIDE.store(color_when.as_u8(), Ordering::Relaxed);
    let enabled = resolve_colors_enabled(color_when, json_mode, stdout_is_tty());
    COLOR_ENABLED_OVERRIDE.store(enabled, Ordering::Relaxed);
    set_colors_enabled(enabled);
    set_colors_enabled_stderr(enabled);
    owo_colors::set_override(enabled);
}

/// Query whether color output is globally enabled.
pub fn color_output_enabled() -> bool {
    COLOR_ENABLED_OVERRIDE.load(Ordering::Relaxed)
}

pub(crate) fn configured_color_when() -> ColorWhen {
    ColorWhen::from_u8(COLOR_WHEN_OVERRIDE.load(Ordering::Relaxed))
}

pub(crate) fn env_var_present(name: &str) -> bool {
    std::env::var(name)
        .ok()
        .is_some_and(|value| !value.is_empty())
}

pub(crate) fn stdout_is_tty() -> bool {
    forced_tty_for_tests() || std::io::stdout().is_terminal()
}

fn forced_tty_for_tests() -> bool {
    std::env::var("EDEN_SKILLS_FORCE_TTY")
        .ok()
        .is_some_and(|value| value == "1")
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
