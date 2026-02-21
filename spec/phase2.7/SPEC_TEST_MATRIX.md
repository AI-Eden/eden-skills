# SPEC_TEST_MATRIX.md

Minimum acceptance test matrix for Phase 2.7 features.

## 1. Environments

- Linux (latest stable)
- macOS (latest stable)
- Windows (latest stable)

Docker is NOT required for Phase 2.7 tests.

## 2. Lock File Scenarios

### TM-P27-001: Lock File Creation on First Apply

- On a system with no existing lock file, `apply` with a non-empty
  config creates `skills.lock` adjacent to `skills.toml`.
- Lock file contains entries matching all installed skills.
- Exit code `0`.

### TM-P27-002: Lock File Updated After Install

- `install` a skill → `skills.lock` is created or updated.
- Lock entry contains the installed skill's `id`, `source_repo`,
  `resolved_commit` (if available), `install_mode`, `installed_at`,
  and `targets`.

### TM-P27-003: Lock File Updated After Remove

- `remove` a skill → `skills.lock` is updated.
- The removed skill's entry is no longer present in the lock file.

### TM-P27-004: Orphan Removal via Apply

- Given: lock file records skills `A`, `B`, `C`; TOML contains only
  `A`, `B`.
- `apply` MUST generate a `Remove` action for skill `C`.
- After `apply`: skill `C`'s target symlinks/files, agent path entries,
  and storage directory are cleaned up.
- Lock file reflects only `A`, `B`.

### TM-P27-005: Plan Shows Remove Actions

- Same setup as TM-P27-004.
- `plan` output includes `remove` action lines for skill `C`.
- No filesystem changes occur (plan is read-only).

### TM-P27-006: Missing Lock File Fallback

- Delete the lock file manually.
- `apply` succeeds with full reconciliation.
- Lock file is regenerated from scratch.
- No error or warning is emitted about missing lock.

### TM-P27-007: Corrupted Lock File Recovery

- Write garbage content to `skills.lock`.
- `apply` emits warning about corrupted lock.
- `apply` performs full reconciliation and succeeds.
- Lock file is regenerated with valid content.

### TM-P27-008: Lock File Co-Location with Custom Config

- `--config /tmp/test-skills.toml` → lock file created at
  `/tmp/test-skills.lock`.
- Lock file content is valid and reflects installed state.

### TM-P27-009: Lock Entries Sorted Alphabetically

- Install skills with IDs `zebra`, `alpha`, `middle`.
- Lock file lists entries in order: `alpha`, `middle`, `zebra`.

### TM-P27-010: Lock Preserves Resolved Commit

- Install a skill from a Git repository.
- Lock entry `resolved_commit` contains a 40-character hex string.
- Re-running `apply` with no config changes does not alter the commit.

### TM-P27-011: Apply Noop Optimization

- Install skills, verify lock file exists.
- Re-run `apply` with identical config.
- Apply completes faster than first run (source sync may be skipped
  for unchanged skills).
- Exit code `0`, all actions are `noop`.

### TM-P27-012: Lock Init Creates Empty Lock

- `init` creates a config file.
- A lock file is also created with `version = 1` and no skill entries.

### TM-P27-013: Repair Updates Lock

- Given: a broken symlink for an installed skill.
- `repair` fixes the symlink and updates the lock file.
- Lock entry `installed_at` reflects the repair timestamp.

### TM-P27-014: Apply Remove With Docker Target in Lock

- Given: lock file contains a skill with a Docker target.
- Skill is removed from TOML.
- `apply` invokes Docker adapter uninstall for the removed target.
- Lock file is updated without the removed skill.

### TM-P27-015: Strict Mode Does Not Block Removals

- Given: lock has skill `old`; TOML does not.
- `apply --strict` removes skill `old` without error.
- Exit code `0`.

## 3. Help System Scenarios

### TM-P27-016: Version Flag

- `eden-skills --version` outputs `eden-skills <version>`.
- `eden-skills -V` produces the same output.
- Exit code `0`.

### TM-P27-017: Root Help Contains Version and Groups

- `eden-skills --help` output contains:
  - Version string in header.
  - `Deterministic skill installation` in about text.
  - Command group headings: `Install & Update`, `State Reconciliation`,
    `Configuration`.
  - After-help examples section.

### TM-P27-018: Subcommand Help Has Description

- For each subcommand (`plan`, `apply`, `doctor`, `repair`, `update`,
  `install`, `init`, `list`, `add`, `remove`, `set`, `config`):
  - `eden-skills <cmd> --help` contains a non-empty about description.

### TM-P27-019: Argument Help Has Description

- `eden-skills install --help` shows help text for `<SOURCE>`, `--id`,
  `--ref`, `--skill`, `--all`, `--list`, `--target`, `--dry-run`.
- No argument or option has a blank description.

### TM-P27-020: Short Flags Work

- `eden-skills install user/repo -s browser-tool -t cursor` succeeds
  (equivalent to `--skill browser-tool --target cursor`).
- `eden-skills -V` succeeds.

### TM-P27-021: Install Copy Flag

- `eden-skills install ./local-skill --copy` persists config entry
  with `install.mode = "copy"`.
- Verify checks default to `["path-exists", "content-present"]`.

## 4. Output Polish Scenarios

### TM-P27-022: No Hardcoded ANSI in Source

- A source code scan confirms no `\u{1b}[` or `\x1b[` string literals
  exist in `ui.rs` or `commands.rs` (outside of test assertions).
- All color output uses `owo-colors` trait methods.

### TM-P27-023: Console Crate Removed

- `crates/eden-skills-cli/Cargo.toml` does not list `console` as a
  direct dependency.
- `cargo build` succeeds without `console`.

### TM-P27-024: Color Flag Auto

- `eden-skills install ./skill --color auto` with TTY produces ANSI
  output.
- Same command piped to `cat` (non-TTY) produces no ANSI codes.

### TM-P27-025: Color Flag Never

- `eden-skills install ./skill --color never` produces output with no
  ANSI escape sequences, even on TTY.

### TM-P27-026: Color Flag Always

- `eden-skills install ./skill --color always | cat` produces ANSI
  escape sequences even though stdout is not a TTY.

### TM-P27-027: Error Format With Hint

- Trigger a "config not found" error (e.g., `eden-skills list` with
  nonexistent default config).
- Output contains `error:` prefix and `→` hint line.
- When colors are enabled, `error:` is red bold.

### TM-P27-028: Error Context for Missing Config

- `eden-skills list --config /nonexistent/path.toml` produces:
  - Message mentioning the specific path.
  - Hint suggesting `eden-skills init`.
- NOT a raw "io error: No such file or directory".

### TM-P27-029: Error Context for Unknown Skill

- `eden-skills remove nonexistent-skill` produces:
  - Message stating the skill is not found.
  - Hint listing available skill names.

### TM-P27-030: Windows ANSI Support

- On Windows, ANSI escape sequences render correctly when colors are
  enabled (via `enable-ansi-support`).

### TM-P27-031: JSON Mode Unaffected

- `eden-skills apply --json --color always` still produces clean JSON
  without ANSI codes.
- JSON structure matches Phase 1/2/2.5 contracts.

## 5. Remove Enhancement Scenarios

### TM-P27-032: Batch Remove Multiple Skills

- Given: config with skills `a`, `b`, `c`.
- `eden-skills remove a c` removes both skills.
- Config retains only skill `b`.
- Lock file reflects only skill `b`.

### TM-P27-033: Batch Remove Atomic Validation

- Given: config with skills `a`, `b`.
- `eden-skills remove a nonexistent` fails with error listing
  `nonexistent` as unknown.
- Skill `a` is NOT removed (atomic — no partial execution).

### TM-P27-034: Interactive Remove on TTY

- `eden-skills remove` (no arguments) on TTY displays skill list and
  prompts for selection.
- Selecting skill numbers removes only those skills.

### TM-P27-035: Non-TTY Remove Without Arguments Fails

- `echo "" | eden-skills remove` (non-TTY, no arguments) fails with
  error and exit code `2`.

### TM-P27-036: Remove Yes Flag Skips Prompt

- `eden-skills remove browser-tool -y` removes without confirmation
  prompt.

### TM-P27-037: Install Yes Flag Skips Prompt

- `eden-skills install user/repo -y` on a multi-skill repo installs
  all skills without confirmation prompt.

### TM-P27-038: Remove Empty Config

- `eden-skills remove` on an empty config displays "Nothing to remove"
  and exits with code `0`.

### TM-P27-039: Batch Remove JSON Output

- `eden-skills remove a b --json` produces JSON with `removed` array
  containing both skill IDs.

## 6. Regression Gate

### TM-P27-040: Full Regression

- All Phase 1, Phase 2, and Phase 2.5 tests MUST pass without
  modification (except test assertions that explicitly match hardcoded
  ANSI sequences — these MUST be updated for `owo-colors` output).

## 7. CI Gate (Phase 2.7)

A release candidate MUST pass:

- All Phase 1/2/2.5 regression tests on Linux, macOS, and Windows.
- All lock file tests (TM-P27-001 ~ TM-P27-015) on at least two platforms.
- All help system tests (TM-P27-016 ~ TM-P27-021) on at least one platform.
- All output polish tests (TM-P27-022 ~ TM-P27-031) on at least two platforms.
- All remove enhancement tests (TM-P27-032 ~ TM-P27-039) on at least one
  platform.
- Full regression test (TM-P27-040) on all three platforms.
