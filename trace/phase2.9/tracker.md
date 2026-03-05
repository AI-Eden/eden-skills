# Phase 2.9 Execution Tracker

Phase: UX Polish, Update Semantics & Output Consistency
Status: Completed
Started: 2026-03-05
Completed: 2026-03-06

## Batch Plan

| Batch | Name | WP | Requirements | Status |
| --- | --- | --- | --- | --- |
| 1 | Foundation: Table Fix + Newline Policy | WP-1 + WP-5 | TFX-001~003, NLP-001~006 | completed |
| 2 | Output Consistency | WP-4 | OCN-001~010 | completed |
| 3 | Install UX: Card Preview + Tree Display | WP-3 pt1 | IUX-001~003, IUX-006~007, IUX-009 | completed |
| 4 | Install UX: Step Progress + Apply/Repair Integration | WP-3 pt2 | IUX-004~005, IUX-008 | completed |
| 5 | Update Extension | WP-2 | UPD-001~008 | completed |
| 6 | Regression + Closeout | — | TM regression | completed |

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
  - `TM-P29-004` and `TM-P29-005` completed in terminal visual verification and matched expected content-width table behavior.

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
  - `TM-P29-034` completed and confirmed cyan path styling in color-enabled human output.
  - `TM-P29-004` and `TM-P29-005` were also confirmed complete in the same manual verification pass.

### Batch 3 — Install UX: Card Preview + Tree Display (Completed 2026-03-06)

- Requirements in scope: `IUX-001`, `IUX-002`, `IUX-003`, `IUX-006`, `IUX-007`, `IUX-009`
- Completed in this pass:
  - Merged discovery preview rendering into `print_discovery_preview()` and routed both `install --list` + interactive preview through the same formatter (`IUX-002`).
  - Switched discovery preview from table/em-dash style to card-style numbered lines with indented description follow-up lines and wrapping (`IUX-001`, `IUX-003`).
  - Added non-`--list` truncation behavior at 8 entries with footer hint `... and N more (use --list to see all)` (`IUX-001`).
  - Added `install --list --json` contract output as a discovered-skill JSON array (`name`, `description`, `subpath`) while preserving no-install side effects (`IUX-001` / TM-P29-027).
  - Replaced flat install result arrows with grouped tree output (`✓ skill` header + `├─`/`└─` target lines) and inserted summary separator blank line (`IUX-006`, `IUX-007`).
  - Upgraded URL-mode `install --dry-run` preview to two titled tables (`Skill / Version / Source`, `Install Targets`) with 4-space table indentation, default 8-row truncation, and `--dry-run --list` full expansion (`IUX-009`).
  - Simplified `Install Targets` preview to `Agent / Path / Mode` columns and de-duplicated identical target rows across multi-skill dry-run previews.
  - Added SIGINT cursor-restore handling in `main.rs` and graceful prompt-interrupt cancellation (`· Install cancelled`) to avoid runtime error noise on Ctrl+C during interactive prompts.
  - Added/updated tests:
    - `tm_p29_015_install_list_shows_card_style_numbered_list`
    - `tm_p29_016_interactive_preview_matches_list_card_format`
    - `tm_p29_017_discovery_description_uses_indented_followup_line`
    - `tm_p29_018_discovery_skill_without_description_renders_name_only_line`
    - `tm_p29_019_discovery_preview_truncates_to_eight_in_interactive_mode`
    - `tm_p29_023_install_results_use_tree_display_with_connectors`
    - `tm_p29_024_tree_groups_skill_name_once_per_skill_group`
    - `tm_p29_027_install_list_json_contract_returns_discovered_skill_array`
    - `dry_run_multi_skill_preview_defaults_to_eight_skill_rows`
    - `dry_run_multi_skill_with_list_shows_all_skill_rows`
    - `interactive_confirm_interrupt_cancels_without_error_output`
  - Updated superseded legacy assertions in:
    - `tests/table_fix_tests.rs`
    - `tests/output_upgrade_b_tests.rs`
    - `tests/output_upgrade_a2_tests.rs`
- Validation:
  - `cargo fmt --all -- --check` ✅
  - `cargo clippy --workspace -- -D warnings` ✅
  - `cargo test --workspace` ✅
  - Test inventory: `325`
- Manual scenarios:
  - `TM-P29-025` completed with terminal visual verification for tree connector/path/mode coloring.
  - SIGINT cursor-restore path completed with real terminal interruption checks.

### Batch 4 — Install UX: Step Progress + Apply/Repair Integration (Completed 2026-03-06)

- Requirements in scope: `IUX-004`, `IUX-005`, `IUX-008`
- Completed in this pass:
  - Added install source-sync step progress runner (`SourceSyncProgress`) for URL/registry install flows with TTY step markers (`[pos/len]`) and per-step message updates.
  - Replaced install source-sync key-value output with compact completion summary line: `Syncing  N synced, M failed`.
  - Added shared summary helper `print_source_sync_step_summary_human()` in `commands/common.rs`.
  - Updated install sync loops to aggregate synced/failed counts and emit one summary line per install run (TTY and non-TTY human mode).
  - Ported `apply`/`repair` install output from flat arrow lines to grouped tree output in `reconcile.rs`:
    - skill header once (`✓ skill-id`)
    - target children as `├─` / `└─`
    - dimmed connectors/mode labels and styled paths preserved.
  - Added/updated tests:
    - `tm_p29_020_source_sync_shows_step_style_progress_in_tty`
    - `tm_p29_021_source_sync_prints_summary_line_after_completion`
    - `tm_p29_022_non_tty_source_sync_skips_progress_bar_and_keeps_summary`
    - `tm_p29_026_apply_and_repair_use_tree_style_install_lines`
    - Updated regression expectation: `tm_p28_014_apply_per_skill_install_lines`
  - Follow-up hardening:
    - Added cooperative SIGINT prompt handling (`signal::PromptInterruptGuard`) so Ctrl+C during interactive prompt reads is deferred to command-level cancellation handling.
    - Aligned `remove` interruption semantics with `install` for both confirmation and selection prompts.
    - Added interruption regression tests:
      - `remove_confirm_interrupt_is_handled_as_graceful_cancellation`
      - `remove_selection_interrupt_is_handled_as_graceful_cancellation`
- Validation:
  - `cargo fmt --all -- --check` ✅
  - `cargo clippy --workspace -- -D warnings` ✅
  - `cargo test --workspace` ✅
  - Test inventory: `331`
- Manual scenarios status:
  - `TM-P29-004`, `TM-P29-005`, `TM-P29-020`, `TM-P29-025`, and `TM-P29-034` completed and match expected behavior.
  - SIGINT cursor-restore behavior in a real terminal session is verified.

### Batch 5 — Update Extension (Completed 2026-03-06)

- Requirements in scope: `UPD-001`, `UPD-002`, `UPD-003`, `UPD-004`, `UPD-005`, `UPD-006`, `UPD-007`, `UPD-008`
- Completed in this pass:
  - Added `--apply` flag to `update` CLI args and request wiring (`lib.rs`, `commands/mod.rs`).
  - Extended `update` to run two sequential refresh layers:
    - registry sync (existing behavior),
    - Mode A skill refresh (`git fetch --depth 1 origin <ref>`) with reactor concurrency.
  - Implemented Mode A refresh status model and rendering:
    - table statuses: `new commit`, `up-to-date`, `missing`, `failed`,
    - `Refresh  N skills checked` section in human output,
    - plain text status cells (no ANSI styling attributes in table cells).
  - Added empty-state guidance for no registries + no skills:
    - `Update  no skills or registries configured`
    - `→ Run 'eden-skills install <owner/repo>' to get started.`
  - Extended `update --json` output additively with `skills` array (`id`, `status`, `local_sha`, `remote_sha`) and conditional `applied` field when `--apply` is set.
  - Implemented `update --apply` reconcile path for changed/missing Mode A skills:
    - source sync, safety analysis, plan application, verification, lock write.
    - added fetch-head materialization (`reset --hard FETCH_HEAD`) during apply path to realize fetched commits before reconciliation.
  - Added new integration test file:
    - `crates/eden-skills-cli/tests/update_ext_tests.rs`
    - covers `TM-P29-006` through `TM-P29-014`.
- Validation:
  - `cargo fmt --all` ✅
  - `cargo clippy --workspace -- -D warnings` ✅
  - `cargo test --workspace` ✅
  - Test inventory: `340`
- Manual scenarios status (from prior batches):
  - `TM-P29-004`, `TM-P29-005`, `TM-P29-020`, `TM-P29-025`, and `TM-P29-034` completed in post-batch manual verification.
  - SIGINT cursor-restore behavior in real terminal sessions is verified.

### Batch 6 — Regression + Closeout (Completed 2026-03-06)

- Closeout validation:
  - `cargo fmt --all -- --check` ✅
  - `cargo clippy --workspace -- -D warnings` ✅
  - `cargo test --workspace` ✅ (test inventory: `340`)
  - `rg '\x1b\[' crates/` ✅ (no hardcoded ANSI escape sequence matches)
- Contract regression checks:
  - JSON contracts remain unchanged; JSON contract suites continue to pass with no schema regressions.
  - Exit code semantics remain unchanged (`0`/`1`/`2`/`3`); `exit_code_matrix` and strict-mode command suites stay green.
- Closeout sync:
  - `trace/phase2.9/status.yaml` updated to `closeout_completed` with Batch 6 completion record.
  - `README.md` current-status section updated to include Phase 2.9 completion.
