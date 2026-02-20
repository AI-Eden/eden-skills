# SPEC_TEST_MATRIX.md

Minimum acceptance test matrix for Phase 2.5 features.

## 1. Environments

- Linux (latest stable)
- macOS (latest stable)
- Windows (latest stable)

Docker is NOT required for Phase 2.5 tests (adapter tests remain in Phase 2).

## 2. Schema Amendment Scenarios

### TM-P25-001: Empty Skills Array Validation

- Config with `version = 1`, `[storage]`, and no `[[skills]]` entries MUST
  load without validation errors.
- Exit code `0`.

### TM-P25-002: Empty Config Plan

- `plan` with empty skills config produces no plan items.
- Output: `0 actions` (human) or empty array (JSON).
- Exit code `0`.

### TM-P25-003: Empty Config Apply

- `apply` with empty skills config performs no mutations.
- Source sync, safety, and install phases all produce zero counts.
- Exit code `0`.

### TM-P25-004: Init Template Minimal

- `eden-skills init` creates a config file containing `version = 1` and
  `[storage]` section, with no `[[skills]]` entries.
- The generated file passes validation.

### TM-P25-005: Backward Compatibility

- A legacy non-empty Phase 1/2 `skills.toml` fixture (for example, 5 skills)
  loads and validates unchanged under the amended schema.
- All Phase 1 and Phase 2 tests pass without modification.

## 3. Source Format Scenarios

### TM-P25-006: GitHub Shorthand

- `eden-skills install vercel-labs/agent-skills` expands to
  `https://github.com/vercel-labs/agent-skills.git` and clones successfully.

### TM-P25-007: Full GitHub URL

- `eden-skills install https://github.com/user/repo` clones the repository.
- `.git` suffix is appended automatically if missing.

### TM-P25-008: GitHub Tree URL

- `eden-skills install https://github.com/owner/repo/tree/main/skills/browser`
  extracts repo URL, ref=`main`, subpath=`skills/browser`.
- Only the specified subpath is installed.

### TM-P25-009: Git SSH URL

- `eden-skills install git@github.com:user/repo.git` is accepted and
  triggers clone via SSH.

### TM-P25-010: Local Path

- `eden-skills install ./test-skills` treats the argument as a local
  directory source.
- No clone operation is performed.
- Persisted config entry uses the resolved absolute path.

### TM-P25-011: Source Format Precedence

- A source matching multiple patterns (e.g., `./owner/repo` which looks
  like both a local path and a shorthand) is classified as local path
  (local path check comes first in precedence).

### TM-P25-012: Registry Fallback

- `eden-skills install browser-tool` (no URL pattern match) falls through
  to Mode B registry resolution (existing Phase 2 behavior).

## 4. Skill ID Derivation Scenarios

### TM-P25-013: Auto-Derived ID

- `eden-skills install https://github.com/user/my-skill.git` derives
  ID = `my-skill`.
- Config entry is persisted with `id = "my-skill"`.

### TM-P25-014: ID Override

- `eden-skills install user/repo --id custom-name` uses `custom-name`
  as the skill ID instead of `repo`.

### TM-P25-015: ID Upsert

- When `skills.toml` already contains a skill with the derived ID,
  the existing entry is updated (not duplicated).

## 5. Skill Discovery Scenarios

### TM-P25-016: Single SKILL.md at Root

- Repository with `SKILL.md` at root is detected as a single-skill repo.
- Install proceeds without confirmation prompt.

### TM-P25-017: Multiple Skills in `skills/` Directory

- Repository with `skills/a/SKILL.md` and `skills/b/SKILL.md` discovers
  2 skills.
- In non-interactive mode (`--all`), both are installed.

### TM-P25-018: Multiple Skills in `packages/` Directory

- Repository with `packages/x/SKILL.md` and `packages/y/SKILL.md`
  discovers 2 skills.

### TM-P25-019: No SKILL.md Found

- Repository with no `SKILL.md` files installs the root directory as
  a single unnamed skill.
- A warning is emitted about missing `SKILL.md`.

### TM-P25-020: List Flag

- `eden-skills install user/repo --list` outputs discovered skill names
  and descriptions.
- No filesystem changes occur.
- No config changes occur.

## 6. Multi-Skill Resolution Scenarios

### TM-P25-021: Install All Flag

- `eden-skills install user/repo --all` installs all discovered skills
  without any prompt.

### TM-P25-022: Install Specific Skills

- `eden-skills install user/repo --skill browser-tool --skill search-tool`
  installs only the two named skills.
- Other discovered skills are not installed.

### TM-P25-023: Unknown Skill Name

- `eden-skills install user/repo --skill nonexistent` fails with an error
  listing available skill names.
- If discovery returns no skills and `--skill` is provided, install MUST fail
  (no root-directory fallback install).

### TM-P25-024: Interactive Confirmation (TTY)

- On TTY, multi-skill repo without `--all`/`--skill` displays skill list
  and prompts for confirmation.
- Responding `y` installs all.
- Responding `n` prompts for specific skill names.

### TM-P25-025: Non-TTY Default

- When stdout is not a TTY, multi-skill repo without `--all`/`--skill`
  defaults to installing all skills (no prompt).

## 7. Agent Detection Scenarios

### TM-P25-026: Multi-Agent Detection

- On a system with both `~/.claude/` and `~/.cursor/` directories,
  `install` without `--target` installs to both agent skill directories.

### TM-P25-027: No Agent Fallback

- On a system with no known agent directories, `install` defaults to
  `claude-code` target and emits a warning.

### TM-P25-028: Target Override

- `eden-skills install user/repo --target cursor` installs only to
  Cursor, even if Claude Code directory exists.

## 8. Config Auto-Creation Scenarios

### TM-P25-029: Fresh System Install

- On a system with no existing config, `eden-skills install user/repo`
  auto-creates the config file.
- The install completes successfully.
- The config contains the installed skill entry.

### TM-P25-030: Missing Parent Directory

- When `--config /nonexistent/path/skills.toml` is used and the parent
  directory does not exist, the CLI fails with an IO error (does not
  silently create arbitrary directory trees).

### TM-P25-041: Default Config Parent Auto-Creation

- When config path is default (`~/.eden-skills/skills.toml`) and
  `~/.eden-skills/` does not exist, `install` auto-creates the parent
  directory and then auto-creates config.
- Install continues successfully after config creation.

## 9. CLI UX Scenarios

### TM-P25-031: TTY Color Output

- When stdout is a TTY, CLI output contains ANSI color codes.
- Status symbols (`✓`, `✗`, `·`) are present in output.

### TM-P25-032: NO_COLOR Compliance

- `NO_COLOR=1 eden-skills install ...` produces output without ANSI
  escape codes.
- Functional output (skill names, status) remains intact.

### TM-P25-033: JSON Mode Unchanged

- `eden-skills install ... --json` produces identical JSON structure
  as Phase 2 install.
- No ANSI codes or visual elements in JSON output.

### TM-P25-034: Spinner During Clone

- During `install` Git clone, a spinner is visible on TTY.
- Spinner is replaced with completion status when done.

### TM-P25-042: Windows Hardcopy Fallback Warning

- When Windows symlink permission is unavailable during `install`, CLI falls
  back to hardcopy mode for persisted skill install config.
- CLI emits a warning that hardcopy fallback may slow down installs.

## 10. Distribution Scenarios

### TM-P25-035: Cargo Install

- `cargo install eden-skills` from a clean environment produces a
  working `eden-skills` binary.
- `eden-skills --help` displays usage information.

### TM-P25-036: Release Binary

- Release binary for the current platform executes without runtime
  dependencies (except Git and optionally Docker).
- `eden-skills init && eden-skills install vercel-labs/agent-skills --all`
  completes successfully.

## 11. CI Gate (Phase 2.5)

A release candidate MUST pass:

- All Phase 1 and Phase 2 regression tests on Linux, macOS, and Windows.
- All Phase 2.5 schema amendment tests (TM-P25-001 ~ TM-P25-005).
- All Phase 2.5 source format tests (TM-P25-006 ~ TM-P25-012).
- All Phase 2.5 skill discovery tests (TM-P25-016 ~ TM-P25-020).
- All Phase 2.5 multi-skill resolution tests (TM-P25-021 ~ TM-P25-025).
- At least one agent detection test (TM-P25-026 ~ TM-P25-028) per platform.
- CLI UX tests (TM-P25-031 ~ TM-P25-034) on at least one platform.
- `cargo install` smoke test (TM-P25-035) on at least one platform.
- Discovery compatibility tests (TM-P25-037 ~ TM-P25-040) on at least one platform.

## 12. Discovery Compatibility Scenarios

### TM-P25-037: Agent-Convention Directory Discovery

- Repository with skills under agent convention roots (e.g.
  `.claude/skills/pdf/SKILL.md`, `.agents/skills/review/SKILL.md`) is
  discovered without requiring recursive fallback.

### TM-P25-038: Marketplace Manifest Discovery

- Repository with `.claude-plugin/marketplace.json` that declares plugin
  skill paths discovers those skills when standard roots do not contain them.

### TM-P25-039: Plugin Manifest Discovery

- Repository with `.claude-plugin/plugin.json` that declares `source` +
  `skills` discovers those skills when standard roots do not contain them.

### TM-P25-040: Recursive Fallback Discovery

- If standard-root scan and plugin-manifest scan return zero, bounded
  recursive discovery finds nested `SKILL.md` directories (for example
  `vendor/tools/pdf/SKILL.md`).

## 13. Cross-Platform Path Assertion Rule

For tests that validate persisted local absolute paths:

- Tests MUST compare canonicalized filesystem locations instead of raw path strings.
- On macOS, `/var/...` and `/private/var/...` SHOULD be treated as equivalent
  when they resolve to the same canonical location.
- CLI integration tests SHOULD use shared test helpers for this assertion style
  (for example, `assert_paths_resolve_to_same_location` in
  `crates/eden-skills-cli/tests/common/mod.rs`) to avoid repeated ad-hoc
  path-comparison logic.
