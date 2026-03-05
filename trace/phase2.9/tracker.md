# Phase 2.9 Execution Tracker

Phase: UX Polish, Update Semantics & Output Consistency
Status: In Progress
Started: 2026-03-05

## Batch Plan

| Batch | Name | WP | Requirements | Status |
| --- | --- | --- | --- | --- |
| 1 | Foundation: Table Fix + Newline Policy | WP-1 + WP-5 | TFX-001~003, NLP-001~006 | completed |
| 2 | Output Consistency | WP-4 | OCN-001~010 | pending |
| 3 | Install UX: Card Preview + Tree Display | WP-3 pt1 | IUX-001~003, IUX-006~007 | pending |
| 4 | Install UX: Step Progress + Apply/Repair Integration | WP-3 pt2 | IUX-004~005, IUX-008 | pending |
| 5 | Update Extension | WP-2 | UPD-001~008 | pending |
| 6 | Regression + Closeout | — | TM regression | pending |

## Completion Records

### Batch 1 — Foundation: Table Fix + Newline Policy (Completed 2026-03-05)

- Requirements: `TFX-001`, `TFX-002`, `TFX-003`, `NLP-001`, `NLP-002`, `NLP-003`, `NLP-004`, `NLP-005`, `NLP-006`
- Key implementation:
  - `UiContext::table()` now uses content-driven tty layout (`ContentArrangement::Disabled`) and preserves non-tty `Dynamic` + width `80`.
  - Added fixed-column `UpperBoundary` constraints at all Batch 1 call sites (`list`, `doctor`, `plan`, `update`, `install --list`, `install --dry-run`).
  - Updated `print_error` spacing policy in `main.rs` and clap parse error `.trim_end()` normalization in `lib.rs`.
  - Added new integration coverage: `table_fix_tests.rs` and `newline_policy_tests.rs`.
  - Follow-up polish: reverted tty dry-run compact-width forcing after regression where `Mode` wrapped as `Mo`/`de` in real terminal output, then moved to content-driven tty table sizing.
  - Follow-up hardening: removed ANSI styling from all table headers/cells to keep table text plain in all color modes (`ui.table` headers, `plan`/`update`/`doctor` table cells).
  - Synced phase2.9 specs and kick prompt to encode the plain-table-text rule for current/future table renderers.
- Regression adjustments:
  - Updated `list_command.rs` text assertion to accept wrapped `Mode` cell output under tighter width constraints.
- Quality gate:
  - `cargo fmt --all -- --check` ✅
  - `cargo clippy --workspace -- -D warnings` ✅
  - `cargo test --workspace` ✅
  - Test inventory: `307`
- Manual scenarios:
  - `TM-P29-004` and `TM-P29-005` remain manual terminal-visual checks.
