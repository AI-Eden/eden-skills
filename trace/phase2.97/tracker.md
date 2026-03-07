# Phase 2.97 Execution Tracker

Phase: Reliability, Interactive UX & Docker Safety
Status: In Progress — Batches 1-3 Completed
Started: 2026-03-07
Completed: —

## Batch Plan

| Batch | Name | WP | Requirements | Status |
| --- | --- | --- | --- | --- |
| 1 | Update Concurrency Fix | WP-1 | UFX-001~003 | completed |
| 2 | Table Content Styling + Help Colorization + List Table + Hint Sync | WP-2 + WP-6 | TST-001~008, HSY-001~002 | completed |
| 3 | Interactive UX (Remove + Install) | WP-3 | IUX-001~010 | completed |
| 4 | Cache Clean | WP-4 | CCL-001~007 | pending |
| 5 | Docker Management Domain | WP-5 | DMG-001~008 | pending |
| 6 | Documentation + Regression + Closeout | WP-7 | DOC-001~002, TM regression | pending |

## Dependency Constraints

- Batches 1, 2, 3, 4, 5 are independent of each other.
- Batch 6 (documentation + regression) depends on all preceding batches.

## Completion Records

### Batch 1 — Update Concurrency Fix (Completed 2026-03-07)

- Requirements: `UFX-001`, `UFX-002`, `UFX-003`
- Completed in this pass:
  - Refactored `crates/eden-skills-cli/src/commands/update.rs` so Mode A refresh groups remote skills by `repo_cache_key()` and performs exactly one fetch per shared repo-cache checkout.
  - Preserved per-skill output by broadcasting the grouped fetch outcome back into individual `SkillRefreshResult` rows for both human and JSON update output.
  - Kept local-source skills out of repo-cache grouping, so locally staged copies still refresh independently instead of sharing a remote dedup key.
  - Added stale `.git/index.lock` and `.git/shallow.lock` cleanup before fetch, with warning output when aged locks are removed.
  - Added `crates/eden-skills-cli/tests/update_fix_tests.rs` covering `TM-P297-001` through `TM-P297-006`.
- Validation:
  - `cargo fmt --all -- --check` ✅
  - `cargo clippy --workspace -- -D warnings` ✅
  - `cargo test --workspace` ✅
  - Test inventory: `399`
- Notes:
  - The Batch 1 tests use the existing `EDEN_SKILLS_TEST_GIT_FETCH_LOG` pattern to count refresh fetches, ensuring the shared-repo race is asserted deterministically instead of relying on flaky timing.

### Batch 2 — Table Content Styling + Help Colorization + List Table + Hint Sync (Completed 2026-03-07)

- Requirements: `TST-001`, `TST-002`, `TST-003`, `TST-004`, `TST-005`, `TST-006`, `TST-007`, `TST-008`, `HSY-001`, `HSY-002`
- Completed in this pass:
  - Enabled `comfy-table` `custom_styling` and centralized header, skill ID, status, secondary-detail, cyan path/source, and hint-prefix styling in `crates/eden-skills-cli/src/ui.rs`.
  - Styled the Batch 2 table surfaces in `config_ops.rs`, `update.rs`, `diagnose.rs`, `plan_cmd.rs`, and `install.rs` so Skill IDs render bold+magenta, status cells follow semantic colors, and column widths remain visually aligned with ANSI content.
  - Added clap help colorization in `crates/eden-skills-cli/src/lib.rs`, including explicit `--color` propagation into clap's own help/version rendering path.
  - Reworked `list` human output to show `Path` instead of `Source`, using resolved repo-cache-backed source directories and truncating long agent lists after five entries with yellow `+N more`.
  - Normalized all Batch 2 hint/remediation guidance to magenta `~>` prefixes, including the previously inconsistent Docker bind-mount tip line in `install.rs`.
  - Added `crates/eden-skills-cli/tests/table_style_tests.rs` and `crates/eden-skills-cli/tests/hint_sync_tests.rs`, and updated affected legacy table/help assertions for the new styling contract.
- Validation:
  - `cargo fmt --all -- --check` ✅
  - `cargo clippy --workspace -- -D warnings` ✅
  - `cargo test --workspace` ✅
  - Test inventory: `412`
- Notes:
  - The Batch 2 handoff prompt claimed HSY verification would require no code changes, but the implementation still had one residual `→` tip in `install.rs`; it was treated as a code defect and fixed together with the new verification tests.

### Batch 3 — Interactive UX (Remove + Install) (Completed 2026-03-07)

- Requirements: `IUX-001`, `IUX-002`, `IUX-003`, `IUX-004`, `IUX-005`, `IUX-006`, `IUX-007`, `IUX-008`, `IUX-009`, `IUX-010`
- Completed in this pass:
  - Added shared checkbox-selection infrastructure in `crates/eden-skills-cli/src/ui.rs`, including `SkillSelectTheme`, `prompt_skill_multi_select()`, a custom terminal renderer, 57-character inline description truncation, viewport `...` overflow markers, and synchronized `dialoguer`/`console` color policy with the existing `--color` handling.
  - Replaced the legacy text-input remove flow in `crates/eden-skills-cli/src/commands/remove.rs` with shared checkbox-selector index resolution, keeping explicit-ID removal, post-selection confirmation, and graceful cancellation behavior intact.
  - Removed the Phase 2.95 `*` wildcard selection behavior from interactive remove mode so `EDEN_SKILLS_TEST_REMOVE_INPUT` now accepts only comma-separated 0-based indices or `interrupt`.
  - Replaced the install confirm-plus-name-input flow in `crates/eden-skills-cli/src/commands/install.rs` with shared checkbox-selector skill selection while preserving `--all`, `--skill`, `--list`, `--dry-run`, single-skill direct install, and non-interactive install-all fallback semantics.
  - Aligned interactive styling with the upstream `vercel-labs/skills` screenshots: no bold prompt items, cyan active unchecked checkboxes, green checked checkboxes, dim inline descriptions, and checked install items retaining their descriptions after the cursor moves away.
  - Refined the active prompt-item label color back to terminal-default white and changed selector redraw to emit a single block without a trailing newline, preventing repeated `Found N skills` headers from being pushed into scrollback when the terminal is short and later resized taller.
  - Added `crates/eden-skills-cli/tests/interactive_ux_tests.rs` and updated the affected legacy remove/install/output tests to validate Phase 2.97 checkbox-selection behavior and the retired wildcard/path-preview expectations.
- Validation:
  - `cargo fmt --all -- --check` ✅
  - `cargo clippy --workspace -- -D warnings` ✅
  - `cargo test --workspace` ✅
  - Test inventory: `422`
- Notes:
  - The original `dialoguer::MultiSelect` theme approach produced stale-frame artifacts when inline descriptions could soft-wrap. The final implementation uses a shared custom renderer instead of `dialoguer`'s built-in list drawing so viewport clearing and overflow indicators stay deterministic in real terminals.
