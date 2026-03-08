//! Table rendering helpers and semantic status symbols.
//!
//! [`StatusSymbol`] provides the canonical set of outcome glyphs
//! used across all human-mode CLI output.

/// Semantic symbols rendered in human-mode output (e.g. `✓`, `✗`, `!`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusSymbol {
    Success,
    Failure,
    Skipped,
    Warning,
}
