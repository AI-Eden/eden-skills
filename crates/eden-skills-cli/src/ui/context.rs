//! [`UiContext`] — the central entry point for color-aware output.
//!
//! Captures TTY state, color/symbol policy, and JSON mode at construction
//! time so that every rendering call produces output consistent with the
//! user's terminal capabilities and CLI flags.

use comfy_table::{presets, Cell, ContentArrangement, Table};
use owo_colors::OwoColorize;

use super::color::{configured_color_when, env_var_present, stdout_is_tty, ColorWhen};
use super::format::{abbreviate_home_path, create_spinner, UiSpinner};
use super::table::StatusSymbol;

/// Central context for all human-mode output decisions.
///
/// Captures TTY state, color/symbol policy, and JSON mode at construction
/// time so that every rendering call produces output consistent with the
/// user's terminal capabilities and CLI flags.
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
    /// Construct a context by snapshotting the current environment.
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

    /// Whether JSON output mode is active.
    pub fn json_mode(&self) -> bool {
        self.json_mode
    }

    /// Whether ANSI colors should be emitted in the current context.
    ///
    /// Precedence: `--json` → `--color` flag → `NO_COLOR` → `FORCE_COLOR` → `CI` → TTY.
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

    /// Whether Unicode status symbols (✓, ✗, etc.) should be emitted.
    pub fn symbols_enabled(&self) -> bool {
        let force_symbols = matches!(self.color_when, ColorWhen::Always) || self.force_color;
        !self.json_mode && (self.stdout_is_tty || force_symbols) && !self.ci
    }

    /// Whether a progress spinner should be displayed.
    pub fn spinner_enabled(&self) -> bool {
        !self.json_mode && self.stdout_is_tty && !self.ci
    }

    /// Whether interactive prompts (confirm, input) are allowed.
    pub fn interactive_enabled(&self) -> bool {
        !self.json_mode && self.stdout_is_tty && !self.ci
    }

    /// Render a colored status symbol string for the given semantic value.
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

    /// Render a right-padded, bold-cyan action label (e.g. `" Install"`).
    pub fn action_prefix(&self, action: &str) -> String {
        let padded = format!("{action:>8}");
        if self.colors_enabled() {
            padded.cyan().bold().to_string()
        } else {
            padded
        }
    }

    /// Abbreviate a path with `~` and colorize it for human output.
    ///
    /// Paths are rendered in cyan when colors are enabled. JSON mode and
    /// `--color never` keep the abbreviated path as plain text.
    pub fn styled_path(&self, path: &str) -> String {
        let abbreviated = abbreviate_home_path(path);
        if self.colors_enabled() {
            abbreviated.cyan().to_string()
        } else {
            abbreviated
        }
    }

    /// Style a skill identifier for table cells and other human-facing output.
    pub fn styled_skill_id(&self, skill_id: &str) -> String {
        if self.colors_enabled() {
            skill_id.bold().magenta().to_string()
        } else {
            skill_id.to_string()
        }
    }

    /// Style an agent name for table cells and other human-facing output.
    pub fn styled_agent_name(&self, agent_name: &str) -> String {
        if self.colors_enabled() {
            agent_name.magenta().to_string()
        } else {
            agent_name.to_string()
        }
    }

    /// Style a version string for table cells and other human-facing output.
    pub fn styled_version(&self, version: &str) -> String {
        if self.colors_enabled() {
            version.yellow().to_string()
        } else {
            version.to_string()
        }
    }

    /// Style a semantic status label for table output.
    pub fn styled_status(&self, status: &str) -> String {
        if !self.colors_enabled() {
            return status.to_string();
        }
        match status {
            "up-to-date" | "ok" | "noop" => status.green().to_string(),
            "failed" | "error" => status.red().to_string(),
            "warning" | "conflict" => status.yellow().to_string(),
            "skipped" | "missing" => status.dimmed().to_string(),
            "cloned" | "updated" | "new commit" => status.cyan().to_string(),
            _ => status.to_string(),
        }
    }

    /// Style secondary detail text such as modes or explanatory suffixes.
    pub fn styled_secondary(&self, text: &str) -> String {
        if self.colors_enabled() {
            text.dimmed().to_string()
        } else {
            text.to_string()
        }
    }

    /// Style generic cyan content such as source labels.
    pub fn styled_cyan(&self, text: &str) -> String {
        if self.colors_enabled() {
            text.cyan().to_string()
        } else {
            text.to_string()
        }
    }

    /// Style warning-emphasis text such as list truncation markers.
    pub fn styled_warning_text(&self, text: &str) -> String {
        if self.colors_enabled() {
            text.yellow().to_string()
        } else {
            text.to_string()
        }
    }

    /// Render the canonical hint prefix used across CLI guidance lines.
    pub fn hint_prefix(&self) -> String {
        if self.colors_enabled() {
            "~>".magenta().to_string()
        } else {
            "~>".to_string()
        }
    }

    /// Render a signal-driven cancellation line for interactive prompts.
    pub fn signal_cancelled_line(&self, action: &str) -> String {
        let content = format!("◆  {action} canceled");
        if self.colors_enabled() {
            content.red().to_string()
        } else {
            content
        }
    }

    /// Style a table header label.
    pub fn style_table_header(&self, header: &str) -> String {
        if self.colors_enabled() {
            header.bold().to_string()
        } else {
            header.to_string()
        }
    }

    /// Create a [`Table`] pre-configured for the current terminal context.
    ///
    /// TTY output uses content-driven column widths with bold headers when
    /// colors are enabled; non-TTY output keeps ASCII borders capped at
    /// 80 columns with dynamic wrapping.
    pub fn table(&self, headers: &[&str]) -> Table {
        let mut table = Table::new();
        let human_tty = self.stdout_is_tty && !self.ci;
        if human_tty {
            table.load_preset(presets::UTF8_FULL_CONDENSED);
            table.set_content_arrangement(ContentArrangement::Disabled);
        } else {
            table.load_preset(presets::ASCII_FULL_CONDENSED);
            table.set_width(80);
            table.set_content_arrangement(ContentArrangement::Dynamic);
        }

        let header_cells = headers
            .iter()
            .map(|header| Cell::new(self.style_table_header(header)))
            .collect::<Vec<_>>();
        table.set_header(header_cells);
        table
    }

    /// Start a terminal spinner with an action label and detail message.
    ///
    /// Returns a no-op spinner when the terminal does not support animation.
    pub fn spinner(&self, action: &str, detail: String) -> UiSpinner {
        create_spinner(action, detail, self)
    }
}
