# Phase 2.9 Execution Tracker

Phase: UX Polish, Update Semantics & Output Consistency
Status: In Progress
Started: 2026-03-05

## Batch Plan

| Batch | Name | WP | Requirements | Status |
| --- | --- | --- | --- | --- |
| 1 | Foundation: Table Fix + Newline Policy | WP-1 + WP-5 | TFX-001~003, NLP-001~006 | completed |
| 2 | Output Consistency | WP-4 | OCN-001~010 | completed |
| 3 | Install UX: Card Preview + Tree Display | WP-3 pt1 | IUX-001~003, IUX-006~007 | in_progress |
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

### Batch 2 — Output Consistency (Completed 2026-03-06)

- Requirements: `OCN-001`, `OCN-002`, `OCN-003`, `OCN-004`, `OCN-005`, `OCN-006`, `OCN-007`, `OCN-008`, `OCN-009`, `OCN-010`
- Key implementation:
  - Added `UiContext::styled_path(&self, path)` and adopted it in human output path sites (`init`, `add`, `set`, `config import`, install/apply/repair result lines, plan text lines, and doctor message path substitutions).
  - Upgraded `add`, `set`, and `config import` success output to symbol-first consistency lines with abbreviated paths.
  - Unified warning rendering through `print_warning()` in `config_ops.rs`, `remove.rs`, and registry manifest validation in `common.rs`.
  - Changed remove cancellation to `· Remove cancelled` and replaced interactive remove candidate list with a `UiContext` table (`#`, `Skill`, `Source`) using abbreviated repo source values.
  - Added result-line emphasis styling: bold skill IDs plus dimmed mode labels/connectors in install/reconcile/plan text output paths.
- New tests:
  - Added `crates/eden-skills-cli/tests/output_consistency_tests.rs` with:
    - `tm_p29_028_add_shows_added_line_with_abbreviated_path`
    - `tm_p29_029_set_shows_updated_line_with_abbreviated_path`
    - `tm_p29_030_config_import_shows_imported_line_with_abbreviated_path`
    - `tm_p29_031_no_raw_warning_eprintln_remains_in_target_files`
    - `tm_p29_032_remove_cancellation_uses_skipped_symbol_line`
    - `tm_p29_033_remove_interactive_candidates_render_as_table`
    - `tm_p29_035_ui_context_exposes_styled_path_method`
- Quality gate:
  - `cargo fmt --all -- --check` ✅
  - `cargo clippy --workspace -- -D warnings` ✅
  - `cargo test --workspace` ✅
  - Test inventory: `315`
- Manual scenarios:
  - `TM-P29-034` remains pending for terminal color verification.
  - `TM-P29-004` and `TM-P29-005` remain pending from Batch 1.

### Batch 3 — Install UX: Card Preview + Tree Display (In Progress, started 2026-03-06)

- Requirements in scope: `IUX-001`, `IUX-002`, `IUX-003`, `IUX-006`, `IUX-007`
- Completed in this pass:
  - Merged discovery preview rendering into `print_discovery_preview()` and routed both `install --list` + interactive preview through the same formatter (`IUX-002`).
  - Switched discovery preview from table/em-dash style to card-style numbered lines with indented description follow-up lines and wrapping (`IUX-001`, `IUX-003`).
  - Added non-`--list` truncation behavior at 8 entries with footer hint `... and N more (use --list to see all)` (`IUX-001`).
  - Added/updated tests:
    - `tm_p29_015_install_list_shows_card_style_numbered_list`
    - `tm_p29_016_interactive_preview_matches_list_card_format`
    - `tm_p29_017_discovery_description_uses_indented_followup_line`
    - `tm_p29_018_discovery_skill_without_description_renders_name_only_line`
    - `tm_p29_019_discovery_preview_truncates_to_eight_in_interactive_mode`
  - Updated superseded legacy assertions in:
    - `tests/table_fix_tests.rs`
    - `tests/output_upgrade_b_tests.rs`
    - `tests/output_upgrade_a2_tests.rs`
- Validation:
  - `cargo fmt --all -- --check` ✅
  - `cargo clippy --workspace -- -D warnings` ✅
  - `cargo test --workspace` ✅
  - Test inventory: `319`
- Remaining in Batch 3:
  - Tree-style grouped install result output (`IUX-006`, `IUX-007`)
  - `TM-P29-027` (`install --list --json` contract confirmation)
