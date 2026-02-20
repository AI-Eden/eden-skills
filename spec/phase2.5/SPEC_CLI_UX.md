# SPEC_CLI_UX.md

CLI output beautification guidelines for `eden-skills`.

## 1. Purpose

Upgrade the CLI from plain key-value text output to a polished, modern
terminal experience with colors, spinners, status symbols, and structured
formatting — while preserving full backward compatibility for `--json`
machine-readable output and non-TTY environments.

## 2. Technology Stack

### 2.1 Required Crates

| Crate | Purpose | Minimum Version |
| :--- | :--- | :--- |
| `console` | Terminal styling, color support, emoji/symbols, text utilities | latest stable |
| `indicatif` | Progress spinners and progress bars for long-running operations | latest stable (0.18+) |
| `dialoguer` | Interactive confirmation prompts and text input | latest stable (0.12+) |

These three crates belong to the [console-rs](https://github.com/console-rs)
ecosystem and are designed to work together.

### 2.2 Optional Crate

| Crate | Purpose | When to Add |
| :--- | :--- | :--- |
| `owo-colors` | Zero-allocation ANSI colors (lighter alternative to `console::style`) | If `console` color API proves too heavy for simple formatting |

## 3. Output Modes

### 3.1 Human Mode (Default)

When `--json` is NOT set and stdout IS a TTY:

- Colors, spinners, and symbols MUST be enabled.
- Interactive prompts (Section 5) MUST be available.

### 3.2 Plain Mode (Non-TTY)

When stdout is NOT a TTY (piped, redirected, CI):

- Colors and spinners MUST be disabled.
- Interactive prompts MUST be skipped (see `SPEC_INSTALL_URL.md` 5.5).
- Plain text output without ANSI escape codes MUST be used.
- The `CI` environment variable, if set, SHOULD also trigger plain mode.

### 3.3 JSON Mode (`--json`)

When `--json` is set:

- Output MUST be valid JSON, identical to existing Phase 1/2 contracts.
- No colors, spinners, or symbols MUST appear in JSON output.
- JSON output format is a normative contract and MUST NOT change.

### 3.4 Environment Variable Support

| Variable | Effect |
| :--- | :--- |
| `NO_COLOR` | When set (any value), MUST disable all colors. Spinners and symbols remain. |
| `FORCE_COLOR` | When set (any value), MUST force colors even on non-TTY. |
| `CI` | When set, SHOULD disable colors and interactive prompts. |

`NO_COLOR` takes precedence over `FORCE_COLOR` when both are set.
Reference: [no-color.org](https://no-color.org/).

## 4. Visual Design Language

### 4.1 Status Symbols

| Symbol | Meaning | Usage |
| :--- | :--- | :--- |
| `✓` (green) | Success | Completed install, verification passed |
| `✗` (red) | Failure | Failed operation, verification failed |
| `·` (dim) | Skipped | Skipped items (no-exec, noop) |
| `!` (yellow) | Warning | Non-critical issues, deprecation notices |

### 4.2 Action Prefixes

Long-running or phased operations MUST display a right-aligned, styled
action prefix followed by the detail:

```text
  Cloning  vercel-labs/agent-skills ━━━━━━━━ done
  Syncing  1 cloned, 4 up-to-date
  Install  ✓ browser-tool → ~/.claude/skills/browser-tool (symlink)
```

Action prefixes SHOULD be styled in bold or a distinct color for scanability.

### 4.3 Error and Warning Formatting

- Errors MUST be prefixed with `error:` in red bold.
- Warnings MUST be prefixed with `warning:` in yellow bold.
- Remediation hints SHOULD be indented below the error/warning line.

Example:

```text
error: source path missing for skill `browser-tool`
  → Run `eden-skills apply` to sync sources.
```

## 5. Command-Specific UX Contracts

### 5.1 `install` (URL Mode)

```text
  Cloning  vercel-labs/agent-skills ━━━━━━━━ done       ← spinner during clone
  Found    3 skills in repository:                       ← discovery summary

    1. browser-tool        — Browser automation
    2. filesystem-tool     — File system operations
    3. github-tool         — GitHub API integration

  Install all 3 skills? [Y/n] y                         ← interactive prompt

  Install  ✓ browser-tool → ~/.claude/skills/ (symlink)
           ✓ browser-tool → ~/.cursor/skills/ (symlink)
           ✓ filesystem-tool → ~/.claude/skills/ (symlink)
           ✓ filesystem-tool → ~/.cursor/skills/ (symlink)
           ✓ github-tool → ~/.claude/skills/ (symlink)
           ✓ github-tool → ~/.cursor/skills/ (symlink)

  ✓ 3 skills installed to 2 agents, 0 conflicts
```

### 5.2 `apply` / `repair`

```text
  Syncing  3 cloned, 2 up-to-date, 0 failed
  Safety   5 permissive, 0 risk flags

  Install  ✓ browser-tool → ~/.claude/skills/ (symlink)
           ✓ browser-tool → ~/.cursor/skills/ (symlink)
           · github-tool (skipped: metadata-only)

  ✓ 2 created, 0 updated, 3 noop, 0 conflicts
  ✓ Verification passed
```

### 5.3 `doctor`

Findings MUST be grouped by severity and color-coded:

```text
  Doctor   2 issues detected

  ✗ [SOURCE_MISSING] browser-tool
    Source path does not exist
    → Run `eden-skills apply` to sync sources.

  ! [REGISTRY_STALE] registry:official
    Registry cache last synced 14 day(s) ago
    → Run `eden-skills update` to refresh.
```

### 5.4 `plan`

```text
  Plan     4 actions

  create   browser-tool → ~/.claude/skills/browser-tool (symlink)
  create   browser-tool → ~/.cursor/skills/browser-tool (symlink)
  noop     filesystem-tool → ~/.cursor/skills/filesystem-tool
  conflict github-tool → ~/.claude/skills/github-tool
           reason: target exists but is not a symlink
```

### 5.5 `list`

```text
  Skills   5 configured

  browser-tool       symlink  claude-code, cursor
  filesystem-tool    symlink  claude-code, cursor
  github-tool        copy     claude-code (metadata-only)
  search-tool        symlink  cursor
  custom-devops-tool symlink  custom:/opt/my-agent/tools
```

### 5.6 `init`

```text
  ✓ Created config at ~/.config/eden-skills/skills.toml

  Next steps:
    eden-skills install <owner/repo>     Install a skill from GitHub
    eden-skills list                     List configured skills
    eden-skills doctor                   Check installation health
```

## 6. Spinner Behavior

### 6.1 When to Use

Spinners MUST be displayed during operations that involve network I/O or
may take more than 1 second:

- Git clone/fetch during source sync.
- Git clone during registry update.
- Repository cloning during URL-mode install.

### 6.2 Completion States

When a spinner operation completes:

- Success: spinner replaced with `✓` (green) + elapsed time if > 2s.
- Failure: spinner replaced with `✗` (red) + error summary.

### 6.3 Multi-Item Progress

When syncing multiple sources concurrently, a single progress line
SHOULD summarize progress:

```text
  Syncing  [3/5] sources ━━━━━━━━━━━━░░░░░░ 60%
```

## 7. Normative Requirements

| ID | Owner | Priority | Statement | Verification |
| :--- | :--- | :--- | :--- | :--- |
| **UX-001** | Builder | **P0** | CLI MUST use colored output with status symbols in human-readable mode. | TTY output contains ANSI color codes and `✓`/`✗` symbols. |
| **UX-002** | Builder | **P0** | CLI MUST use spinner for Git clone and network operations. | Spinner is visible during `install` clone phase on TTY. |
| **UX-003** | Builder | **P0** | CLI MUST use `✓`/`✗`/`·`/`!` symbols for action results. | Each install action shows appropriate symbol. |
| **UX-004** | Builder | **P0** | CLI MUST respect `NO_COLOR`, `FORCE_COLOR`, and `CI` environment variables. | `NO_COLOR=1 eden-skills install ...` produces output without ANSI codes. |
| **UX-005** | Builder | **P0** | `--json` output MUST remain identical to Phase 1/2 contracts. | JSON output schema unchanged; no visual elements in JSON. |
| **UX-006** | Builder | **P1** | Non-TTY output MUST disable colors, spinners, and prompts. | Piped output (`| cat`) contains no ANSI escape codes. |
| **UX-007** | Builder | **P0** | Interactive prompts MUST use `dialoguer` for confirmation and text input. | Install multi-skill prompt uses `dialoguer::Confirm` and `dialoguer::Input`. |

## 8. Migration Strategy

The beautification SHOULD be applied incrementally:

1. Add `console`, `indicatif`, `dialoguer` dependencies.
2. Introduce a shared `ui` module for consistent formatting helpers.
3. Update `install` command output first (highest user visibility).
4. Update `apply`/`repair` output.
5. Update `doctor`, `plan`, `list`, `init` output.
6. Ensure all `--json` code paths are unaffected.

Each step MUST preserve existing test contracts. New UX tests SHOULD
verify TTY vs non-TTY behavior.
