//! Terminal UI primitives for the eden-skills CLI.
//!
//! Provides [`UiContext`] — the central entry point for color-aware output,
//! status symbols, action prefixes, spinners, and table construction.
//! All human-mode rendering flows through this module so that JSON mode,
//! non-TTY pipes, `NO_COLOR`/`FORCE_COLOR`, and `--color` flags are
//! handled consistently in one place.

mod color;
mod context;
mod format;
mod prompt;
mod table;

pub use color::{color_output_enabled, configure_color_output, ColorWhen};
pub use context::UiContext;
pub use format::{abbreviate_home_path, abbreviate_repo_url, UiSpinner};
pub use prompt::{
    prompt_skill_multi_select, wake_interactive_prompt_input, SkillSelectItem, SkillSelectOutcome,
    SkillSelectTheme,
};
pub use table::StatusSymbol;
