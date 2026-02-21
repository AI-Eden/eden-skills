# SPEC_OUTPUT_POLISH.md

Output beautification, color library migration, and error message
refinement for `eden-skills`.

**Related contracts:**

- `spec/phase2.5/SPEC_CLI_UX.md` (Phase 2.5 visual design language)
- `spec/phase1/SPEC_COMMANDS.md` (error semantics and exit codes)

**Amendment:** This spec amends `SPEC_CLI_UX.md` Section 2 (Technology
Stack) by replacing `console` with `owo-colors` as the primary color
library. All other Phase 2.5 UX contracts remain in effect.

## 1. Purpose

Phase 2.5 introduced the visual design language (symbols, action prefixes,
spinners) but the implementation uses hardcoded ANSI escape sequences
(e.g., `\u{1b}[32m`). This spec mandates:

1. Migration to a proper color library (`owo-colors`).
2. A global `--color` option following Rust CLI best practices.
3. Systematic error message refinement with actionable context.
4. Consistent formatting across all commands.

## 2. Technology Stack Amendment

### 2.1 Dependency Changes

| Action | Crate | Version | Purpose |
| :--- | :--- | :--- | :--- |
| **Add** | `owo-colors` | `4` (with `supports-colors` feature) | Zero-allocation ANSI colors with global override |
| **Add** | `enable-ansi-support` | latest stable | Windows 10+ ANSI sequence activation |
| **Keep** | `indicatif` | `0.18+` | Progress spinners (unchanged) |
| **Keep** | `dialoguer` | `0.12+` | Interactive prompts (unchanged) |
| **Remove** | `console` | `0.15` | No longer needed; replaced by `owo-colors` |

### 2.2 Rationale

Per [Rain's Rust CLI recommendations](https://rust-cli-recommendations.sunshowers.io/managing-colors-in-rust.html):

- `owo-colors` is the recommended library for terminal colors in Rust.
- It provides zero-allocation formatting via the `OwoColorize` trait.
- Global color override via `owo_colors::set_override()` eliminates the
  need for per-call color checks.
- `supports-color` feature auto-detects terminal capabilities and respects
  `NO_COLOR` / `FORCE_COLOR`.

### 2.3 Color Palette Constraint

Default output MUST restrict colors to the 12 standard ANSI colors
(red, green, yellow, blue, magenta, cyan, and their bright variants).
Truecolor (24-bit) and Xterm 256-color MUST NOT be used by default, as
they render inconsistently across terminal themes.

## 3. Global `--color` Option

### 3.1 Flag Definition

The root CLI MUST accept a `--color <WHEN>` option:

```text
--color <WHEN>    Control color output [possible values: auto, always, never] [default: auto]
```

### 3.2 Semantics

| Value | Behavior |
| :--- | :--- |
| `auto` | Enable colors when stdout is a TTY, `NO_COLOR` is not set, and `CI` is not set. |
| `always` | Force colors regardless of TTY status (equivalent to `FORCE_COLOR=1`). |
| `never` | Disable all colors (equivalent to `NO_COLOR=1`). |

### 3.3 Precedence

Resolution order (highest to lowest precedence):

1. `--color never` or `--color always` (explicit CLI flag)
2. `NO_COLOR` environment variable (disables)
3. `FORCE_COLOR` environment variable (enables)
4. `CI` environment variable (disables)
5. TTY detection (auto)

When `--json` is set, colors are always disabled regardless of
`--color` value.

### 3.4 Implementation Pattern

At CLI initialization, resolve the color mode and call:

```rust
owo_colors::set_override(resolved_color_enabled);
```

This sets a global atomic flag. All subsequent `.red()`, `.green()`,
`.bold()` calls via `OwoColorize` automatically respect the override
without per-call checks.

On Windows, before setting the override, call:

```rust
enable_ansi_support::enable_ansi_support().ok();
```

to activate the Windows Console ANSI processor.

## 4. UI Module Refactoring

### 4.1 Replace Hardcoded ANSI

All instances of hardcoded escape sequences (`\u{1b}[...`) in `ui.rs`
and `commands.rs` MUST be replaced with `owo-colors` trait methods:

| Before | After |
| :--- | :--- |
| `format!("\u{1b}[32m{raw}\u{1b}[0m")` | `raw.green().to_string()` |
| `format!("\u{1b}[31m{raw}\u{1b}[0m")` | `raw.red().to_string()` |
| `format!("\u{1b}[33m{raw}\u{1b}[0m")` | `raw.yellow().to_string()` |
| `format!("\u{1b}[2m{raw}\u{1b}[0m")` | `raw.dimmed().to_string()` |
| `format!("\u{1b}[1;36m{padded}\u{1b}[0m")` | `padded.cyan().bold().to_string()` |

### 4.2 Stylesheet Pattern

The `UiContext` SHOULD define a stylesheet struct for consistent styling:

```rust
use owo_colors::OwoColorize;

impl UiContext {
    pub fn status_symbol(&self, symbol: StatusSymbol) -> String {
        let raw = match symbol {
            StatusSymbol::Success => "✓",
            StatusSymbol::Failure => "✗",
            StatusSymbol::Skipped => "·",
            StatusSymbol::Warning => "!",
        };
        match symbol {
            StatusSymbol::Success => raw.green().to_string(),
            StatusSymbol::Failure => raw.red().to_string(),
            StatusSymbol::Skipped => raw.dimmed().to_string(),
            StatusSymbol::Warning => raw.yellow().to_string(),
        }
    }
}
```

Since `owo_colors::set_override()` controls the global state, individual
methods no longer need to check `self.colors_enabled()` before applying
styles. The `OwoColorize` trait automatically respects the override.

### 4.3 `console` Crate Removal

After migration, the `console` crate MUST be removed from
`Cargo.toml`. The `console::set_colors_enabled()` calls in
`UiContext::from_env()` MUST be replaced with
`owo_colors::set_override()`.

`indicatif` and `dialoguer` do not depend on `console` for their core
functionality (they use their own TTY detection). If `indicatif` or
`dialoguer` pull `console` transitively, this is acceptable but the
direct dependency MUST be removed.

## 5. Error Message Refinement

### 5.1 Error Display Format

All errors displayed to the user MUST follow this format:

```text
error: <concise description>
  → <actionable hint or remediation>
```

In human mode, `error:` MUST be styled in red bold. The hint line MUST
be indented with two spaces and prefixed with `→`.

Warnings MUST follow the same format with `warning:` in yellow bold.

### 5.2 Contextual Error Wrapping

The following error categories MUST include additional context instead
of exposing raw OS errors:

| Raw Error | Refined Message |
| :--- | :--- |
| `io error: No such file or directory` (config) | `error: config file not found: ~/.eden-skills/skills.toml` + `→ Run 'eden-skills init' to create a new config.` |
| `io error: No such file or directory` (storage) | `error: storage directory not found: ~/.eden-skills/skills/<id>` + `→ Run 'eden-skills apply' to sync sources.` |
| `io error: Permission denied` (target) | `error: permission denied writing to <path>` + `→ Check file permissions or run with appropriate privileges.` |
| `io error: No such file or directory` (git) | `error: git executable not found` + `→ Install Git: https://git-scm.com/downloads` |
| `validation error: ...` | `error: invalid config: <detail>` + `→ Check skills.toml syntax at <path>.` |
| `unknown skill id: \`foo\`` | `error: skill 'foo' not found in config` + `→ Available skills: bar, baz, qux` |

### 5.3 Error Context Implementation

The `EdenError` type in `error.rs` SHOULD be extended with structured
context fields rather than relying on string formatting:

```rust
#[derive(Debug, Error)]
pub enum EdenError {
    #[error("config file not found: {path}")]
    ConfigNotFound { path: String, hint: String },

    #[error("permission denied: {path}")]
    PermissionDenied { path: String },

    // ... existing variants remain for backward compatibility
}
```

Alternatively, the CLI layer MAY wrap core errors with context at the
call site using helper functions. The implementation strategy is at
Builder discretion as long as the user-facing output matches Section 5.2.

### 5.4 Early Failure Detection

The CLI SHOULD perform pre-flight checks for common failure modes and
emit specific errors before attempting the operation:

| Check | When | Error |
| :--- | :--- | :--- |
| Git not on `$PATH` | Before any git clone/fetch | `error: git executable not found` |
| Docker not running | Before Docker adapter operations | `error: Docker daemon is not running` |
| Config parent dir missing (non-default) | Before config write | `error: directory does not exist: /path/to` |
| Storage root not writable | Before `apply`/`install` | `error: storage directory not writable: <path>` |

### 5.5 Main Entry Point Formatting

The `main.rs` error handler MUST format errors consistently:

```rust
Err(err) => {
    // Use owo-colors for error prefix
    eprintln!("{}: {}", "error".red().bold(), err);
    if let Some(hint) = err.hint() {
        eprintln!("  {} {}", "→".dimmed(), hint);
    }
    ExitCode::from(exit_code_for_error(&err))
}
```

## 6. Output Consistency Rules

### 6.1 Action Prefix Width

All action prefixes MUST be right-aligned to a consistent width of
10 characters (increased from 8 to accommodate `"  Removing"`):

```text
  Cloning   vercel-labs/agent-skills
  Installing browser-tool → ~/.claude/skills/ (symlink)
  Removing   old-skill → ~/.claude/skills/
  Syncing   3 cloned, 2 up-to-date
```

Builder MAY keep the existing 8-character width if all action labels fit.
The key constraint is that ALL action labels MUST use the SAME width
within a single command invocation.

### 6.2 Multi-Line Error Indentation

When an error or warning has multiple lines (e.g., listing available
skills), continuation lines MUST be indented to align with the first
line's content:

```text
error: skill 'nonexistent' not found in config
  → Available skills:
      browser-tool
      code-review
      filesystem-tool
```

### 6.3 Remove Action Output

The new `Remove` action (from `SPEC_LOCK.md`) MUST follow the same
visual pattern as other actions:

```text
  Remove   ✓ old-skill → ~/.claude/skills/old-skill
           ✓ old-skill → ~/.cursor/skills/old-skill
```

## 7. Normative Requirements

| ID | Owner | Priority | Statement | Verification |
| :--- | :--- | :--- | :--- | :--- |
| **OUT-001** | Builder | **P0** | All hardcoded ANSI escape sequences MUST be replaced with `owo-colors` trait methods. | No `\u{1b}[` literals remain in source code outside of test assertions. |
| **OUT-002** | Builder | **P0** | `console` crate MUST be removed as a direct dependency. | `Cargo.toml` does not list `console`. |
| **OUT-003** | Builder | **P0** | Root CLI MUST accept `--color auto\|always\|never` with documented precedence. | `--color never` disables all ANSI output; `--color always` enables even on non-TTY. |
| **OUT-004** | Builder | **P0** | Error output MUST use `error:` prefix in red bold with hint line. | `eprintln` output for config-not-found includes red `error:` and `→` hint. |
| **OUT-005** | Builder | **P0** | IO errors for config/storage/git MUST include contextual path and remediation hint (Section 5.2). | Missing config file produces "config file not found: <path>" not "io error: No such file or directory". |
| **OUT-006** | Builder | **P1** | Windows MUST call `enable_ansi_support` before color initialization. | ANSI colors render correctly on Windows Terminal and cmd.exe. |
| **OUT-007** | Builder | **P1** | Color palette MUST be limited to 12 standard ANSI colors. | No truecolor or 256-color codes in output. |
| **OUT-008** | Builder | **P1** | Pre-flight checks SHOULD detect missing git/docker before operations (Section 5.4). | `install` without git on PATH produces specific error, not generic IO error. |

## 8. Migration Strategy

1. Add `owo-colors` and `enable-ansi-support` to `Cargo.toml`.
2. Refactor `UiContext::from_env()` to use `owo_colors::set_override()`.
3. Replace all hardcoded ANSI sequences in `ui.rs`.
4. Replace all hardcoded ANSI sequences in `commands.rs`.
5. Add `--color` flag to root CLI parser.
6. Refactor `main.rs` error display to use formatted error output.
7. Add contextual error wrapping in CLI command functions.
8. Remove `console` from `Cargo.toml`.
9. Update all test assertions that match ANSI escape sequences.

Each step MUST preserve existing `--json` output contracts and pass the
full Phase 1/2/2.5 regression suite.

## 9. Backward Compatibility

| Existing Feature | Phase 2.7 Behavior |
| :--- | :--- |
| `NO_COLOR` / `FORCE_COLOR` / `CI` env vars | Unchanged. `--color` flag adds explicit control but env vars still work. |
| `--json` output | Unchanged. Colors are never applied in JSON mode. |
| Error exit codes | Unchanged (1/2/3 semantics preserved). |
| `UiContext` public API | Method signatures preserved; internal implementation changes only. |
