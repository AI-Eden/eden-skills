# SPEC_TEST_MATRIX.md

Minimum acceptance test matrix for Phase 2.8 features.

## 1. Environments

- Linux (latest stable)
- macOS (latest stable)
- Windows (latest stable)

Docker is NOT required for Phase 2.8 tests.

## 2. Code Structure Scenarios

### TM-P28-001: Commands Module Split Regression

- After the `commands.rs` → `commands/` decomposition, the full
  workspace test suite MUST pass without any test file modifications.
- `cargo fmt --all -- --check` MUST pass.
- `cargo clippy --workspace -- -D warnings` MUST pass.
- `cargo test --workspace` MUST pass.

### TM-P28-002: Public API Unchanged

- Test files that reference `eden_skills_cli::commands::install_async`,
  `eden_skills_cli::commands::CommandOptions`, and
  `eden_skills_cli::run_with_args` MUST compile without changes.

### TM-P28-003: Module Doc Comments Present

- Every file under `commands/` begins with `//!` documentation.
- `ui.rs` begins with `//!` documentation.
- Core crate `lib.rs` begins with `//!` documentation.

## 3. Table Rendering Scenarios

### TM-P28-004: comfy-table Dependency Present

- `crates/eden-skills-cli/Cargo.toml` lists `comfy-table` as a
  dependency.
- `cargo build` succeeds.

### TM-P28-005: List Renders as Table

- Given a config with 3+ skills.
- `eden-skills list` output contains column-aligned text with `Skill`,
  `Mode`, `Source`, `Agents` headers (or aligned column structure).
- Output does NOT contain `skill id=` key-value format.

### TM-P28-006: List Table Non-TTY Degradation

- `eden-skills list | cat` produces ASCII-bordered table output.
- No ANSI escape sequences in output.

### TM-P28-007: List Table JSON Unchanged

- `eden-skills list --json` output matches the existing Phase 1 JSON
  schema (count + skills array).
- No table formatting appears in JSON output.

### TM-P28-008: Install Dry-Run Renders Targets Table

- Given a config with a registry skill targeting 2+ agents.
- `eden-skills install skill-name --dry-run` output contains a metadata
  header (Skill, Version, Source) and a targets table (Agent, Path, Mode).
- Output does NOT contain `target agent=` key-value format.

### TM-P28-009: Install List Renders Numbered Table

- `eden-skills install owner/repo --list` output contains numbered
  entries in a tabular format.
- Each entry shows a number, skill name, and description.

### TM-P28-010: Plan Table Threshold

- Given a config with 6 skills targeting 1 agent each (6 plan items).
- `eden-skills plan` output renders a table with Action, Skill, Target,
  Mode columns.
- Given a config with 3 skills (3 plan items), output uses text format
  with action labels, not a table.

### TM-P28-011: Update Renders Registry Table

- Given a config with 2 registries.
- `eden-skills update` output renders a table with Registry, Status,
  Detail columns.
- Output does NOT contain `registry sync:` key-value format.

## 4. Output Upgrade Scenarios — Category A

### TM-P28-012: Apply Source Sync Styled

- `eden-skills apply` output includes `Syncing` as a styled action
  prefix followed by human-readable counts (e.g., `1 cloned, 0 updated`).
- Output does NOT contain `source sync: cloned=` format.

### TM-P28-013: Apply Safety Summary Styled

- `eden-skills apply` output includes `Safety` as a styled action
  prefix.
- Output does NOT contain `safety summary: permissive=` format.

### TM-P28-014: Apply Per-Skill Install Lines

- Given a config with 2 skills each targeting 2 agents.
- `eden-skills apply` output includes `Install` action prefix and
  individual `✓ skill → path (mode)` lines for each target.
- Symbols are present when TTY is forced.

### TM-P28-015: Apply Summary Styled

- `eden-skills apply` output includes a summary line with `✓` and
  counts: `N created, M updated, K noop, L conflicts`.
- Output does NOT contain `apply summary: create=` format.

### TM-P28-016: Apply Verification Styled

- `eden-skills apply` output includes `✓ Verification passed`.
- Output does NOT contain `apply verification: ok`.

### TM-P28-017: Repair Output Matches Apply Format

- `eden-skills repair` output follows the same styled format as
  `apply` (source sync, safety, per-skill lines, summary, verification).

### TM-P28-018: Doctor Header Styled

- `eden-skills doctor` output shows `Doctor` as a styled action prefix
  followed by issue count.
- When no issues: `Doctor   ✓ no issues detected`.
- Output does NOT contain `doctor: detected` format.

### TM-P28-019: Doctor Findings Cards

- Given a config with a broken symlink (triggering a finding).
- `eden-skills doctor` output shows the finding with:
  - Severity symbol (`✗` or `!`).
  - `[CODE]` and `skill_id` on the same line.
  - Indented message on the next line.
  - `→` remediation on the following line.
- Output does NOT contain `code=X severity=Y` format.

### TM-P28-020: Doctor Summary Table Conditional

- Given a config triggering 4+ doctor findings.
- Output includes a summary table before the detail cards.
- Given a config triggering 2 findings, no summary table appears.

### TM-P28-021: Plan Header and Colored Actions

- `eden-skills plan` output starts with `Plan   N actions`.
- Action labels are right-aligned and colored: `create` in green,
  `conflict` in yellow, `noop` in dim, `remove` in red.
- `→` replaces `->` in target paths.

### TM-P28-022: Plan Empty State

- Given a config with all skills in sync (no pending actions).
- `eden-skills plan` output shows `Plan   ✓ 0 actions (up to date)`.

### TM-P28-023: Init Next Steps

- `eden-skills init` output includes `✓ Created config at` with `~`
  abbreviation.
- Output includes a `Next steps:` block with 3 command examples.
- Output does NOT contain `init: wrote` format.

### TM-P28-024: Install Per-Skill Results

- Install 2 skills from a multi-skill repo.
- Output includes `Install  ✓ skill → path (mode)` for each target.
- Final summary includes `N skills installed to M agents, K conflicts`.
- Output does NOT contain `install: N skill(s) status=installed`.

### TM-P28-025: Install Discovery Numbered

- `eden-skills install owner/repo` on a repo with 3+ skills shows:
  - `Found` action prefix.
  - Numbered list: `1. name — description`.

## 5. Output Upgrade Scenarios — Error Format

### TM-P28-026: Error Hint Uses Arrow

- Trigger a config-not-found error.
- Error output contains `→` (not `hint:`) as the hint prefix.
- When colors are enabled, `→` is dimmed.

### TM-P28-027: Error Path Abbreviated

- Trigger a config-not-found error for the default path.
- Error message contains `~/.eden-skills/skills.toml` (not the
  absolute path).

### TM-P28-028: Warning Format Styled

- Trigger a warning (e.g., `update` with no registries configured).
- Warning output starts with `  warning:` (2-space indent). <!-- markdownlint-disable-line MD038 -->
- When colors are enabled, `warning:` is yellow bold.

## 6. Non-TTY and Color Interaction

### TM-P28-029: Non-TTY Tables Use ASCII Borders

- Pipe any table-producing command through `cat`.
- Table borders use ASCII characters (`+`, `-`, `|`), not Unicode
  box-drawing characters.

### TM-P28-030: Color Never Disables Table Styling

- `eden-skills list --color never` output contains no ANSI codes.
- Table structure is preserved but header is plain text.

### TM-P28-031: JSON Mode Never Renders Tables

- `eden-skills list --json` produces only JSON.
- No table fragments appear in the output.
- `eden-skills plan --json` produces only JSON.

## 7. Path Abbreviation

### TM-P28-032: Home Path Abbreviated

- `abbreviate_home_path("/home/user/.claude/skills/x")` returns
  `~/.claude/skills/x` (when `$HOME` is `/home/user`).
- A path not under `$HOME` is returned unchanged.

### TM-P28-033: Repo URL Abbreviated

- `abbreviate_repo_url("https://github.com/owner/repo.git")` returns
  `owner/repo`.
- `abbreviate_repo_url("https://github.com/owner/repo")` returns
  `owner/repo`.
- A non-GitHub URL is returned unchanged.

## 8. Doc Comment Coverage

### TM-P28-034: CLI Module Docs

- All `commands/*.rs` files have `//!` module-level documentation.
- All public command functions (`install_async`, `apply_async`,
  `repair_async`, `doctor`, `plan`, `init`, `list`, `add`, `set`,
  `remove_many_async`, `update_async`) have `///` doc comments.

### TM-P28-035: Core Module Docs

- `reactor.rs`, `lock.rs`, `adapter.rs`, `source_format.rs`,
  `discovery.rs`, `config.rs`, `plan.rs`, `error.rs` in core crate
  have `//!` module-level documentation.
- Core `lib.rs` has `//!` crate-level documentation.

### TM-P28-036: UiContext Documented

- `ui.rs` has `//!` module-level documentation.
- `UiContext`, `UiSpinner`, `ColorWhen`, `StatusSymbol`,
  `configure_color_output`, `color_output_enabled` have `///` doc
  comments.
- The new `table()` method has a `///` doc comment.

## 9. Regression Gate

### TM-P28-037: Full Regression

- All Phase 1, Phase 2, Phase 2.5, and Phase 2.7 tests MUST pass.
- Test assertions that match specific output strings (e.g.,
  `source sync: cloned=`) MUST be updated to match the new format.
- These updates are expected and legitimate — the old format is being
  replaced by design.

### TM-P28-038: JSON Regression

- All existing `--json` output tests MUST pass without modification.
- JSON output schemas are unchanged.

### TM-P28-039: Exit Code Regression

- All exit code tests MUST pass without modification.
- Exit codes 0/1/2/3 semantics are unchanged.

### TM-P28-040: No Hardcoded ANSI Regression

- The TM-P27-022 check (no `\u{1b}[` literals in source) MUST
  continue to pass after Phase 2.8 changes.

## 10. CI Gate (Phase 2.8)

A release candidate MUST pass:

- All Phase 1/2/2.5/2.7 regression tests on Linux, macOS, Windows.
- Code structure tests (TM-P28-001 ~ TM-P28-003) on all platforms.
- Table rendering tests (TM-P28-004 ~ TM-P28-011) on at least two
  platforms.
- Output upgrade tests (TM-P28-012 ~ TM-P28-028) on at least two
  platforms.
- Non-TTY/color interaction tests (TM-P28-029 ~ TM-P28-031) on at
  least one platform.
- Path abbreviation tests (TM-P28-032 ~ TM-P28-033) on all platforms.
- Doc comment coverage tests (TM-P28-034 ~ TM-P28-036) on at least
  one platform.
- Full regression (TM-P28-037 ~ TM-P28-040) on all platforms.
