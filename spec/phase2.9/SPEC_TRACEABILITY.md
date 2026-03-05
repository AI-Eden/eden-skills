# SPEC_TRACEABILITY.md

Requirement-to-implementation mapping for Phase 2.9.
Use this file to recover accurate context after compression.

**Status:** ACTIVE — Batch 1 through Batch 4 rows are populated.

## 1. Table Fix Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
| --- | --- | --- | --- | --- | --- |
| TFX-001 | `SPEC_TABLE_FIX.md` 3.1 | TTY tables MUST use content-driven layout (`Disabled`) and keep table header/cell text plain (no ANSI styling attributes) | `crates/eden-skills-cli/src/ui.rs` (`UiContext::table`), `crates/eden-skills-cli/src/commands/plan_cmd.rs` (`plan_action_cell`), `crates/eden-skills-cli/src/commands/update.rs` (`registry_status_cell`), `crates/eden-skills-cli/src/commands/diagnose.rs` (`doctor_severity_cell`) | `crates/eden-skills-cli/tests/table_fix_tests.rs` (`tm_p29_001_tty_table_factory_uses_content_driven_width`, `tm_p29_004_table_cells_use_plain_text_renderers`), `crates/eden-skills-cli/tests/table_infra_tests.rs` (`ui_context_table_uses_utf8_borders_plain_headers_and_content_driven_width_on_tty`), TM-P29-004 (manual), TM-P29-005 (manual) | completed |
| TFX-002 | `SPEC_TABLE_FIX.md` 3.2–3.3 | Fixed-width columns MUST have `UpperBoundary` constraints | `crates/eden-skills-cli/src/commands/config_ops.rs`, `crates/eden-skills-cli/src/commands/diagnose.rs`, `crates/eden-skills-cli/src/commands/plan_cmd.rs`, `crates/eden-skills-cli/src/commands/update.rs`, `crates/eden-skills-cli/src/commands/install.rs` | `crates/eden-skills-cli/tests/table_fix_tests.rs` (`tm_p29_003_fixed_columns_apply_upper_boundary_constraints_at_call_sites`) | completed |
| TFX-003 | `SPEC_TABLE_FIX.md` 3.1 | Non-TTY tables MUST use `Dynamic` + width 80 | `crates/eden-skills-cli/src/ui.rs` (`UiContext::table`) | `crates/eden-skills-cli/tests/table_fix_tests.rs` (`tm_p29_002_non_tty_table_factory_keeps_dynamic_with_width_80`) | completed |

## 2. Update Extension Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
| --- | --- | --- | --- | --- | --- |
| UPD-001 | `SPEC_UPDATE_EXT.md` 2.1–2.2 | `update` MUST refresh Mode A skill sources | | TM-P29-006, TM-P29-013 | pending |
| UPD-002 | `SPEC_UPDATE_EXT.md` 2.2 | `update` without `--apply` MUST NOT mutate local state | | TM-P29-007 | pending |
| UPD-003 | `SPEC_UPDATE_EXT.md` 2.3 | `update --apply` MUST reconcile changed skills | | TM-P29-008 | pending |
| UPD-004 | `SPEC_UPDATE_EXT.md` 3.1 | Skill refresh results MUST render as table | | TM-P29-010 | pending |
| UPD-005 | `SPEC_UPDATE_EXT.md` 3.5 | Status values MUST render as plain labels in table cells (no ANSI styling attributes) | | TM-P29-011 | pending |
| UPD-006 | `SPEC_UPDATE_EXT.md` 3.3 | No registries + no skills: install guidance | | TM-P29-009 | pending |
| UPD-007 | `SPEC_UPDATE_EXT.md` 3.6 | `--json` MUST include `skills` array | | TM-P29-012 | pending |
| UPD-008 | `SPEC_UPDATE_EXT.md` 4 | Skill refresh MUST use reactor concurrency | | TM-P29-014 | pending |

## 3. Install UX Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
| --- | --- | --- | --- | --- | --- |
| IUX-001 | `SPEC_INSTALL_UX.md` 2.2 | Discovery preview MUST use card-style numbered list | `crates/eden-skills-cli/src/commands/install.rs` (`print_discovery_preview`, `print_discovery_json`, `install_local_url_mode_async`, `install_remote_url_mode_async`, `resolve_local_install_selection`) | `crates/eden-skills-cli/tests/install_discovery_tests.rs` (`tm_p29_015_install_list_shows_card_style_numbered_list`, `tm_p29_019_discovery_preview_truncates_to_eight_in_interactive_mode`, `tm_p29_027_install_list_json_contract_returns_discovered_skill_array`) | completed |
| IUX-002 | `SPEC_INSTALL_UX.md` 2.1 | Merge two discovery functions into one | `crates/eden-skills-cli/src/commands/install.rs` (`print_discovery_preview`) | `crates/eden-skills-cli/tests/install_discovery_tests.rs` (`tm_p29_016_interactive_preview_matches_list_card_format`) | completed |
| IUX-003 | `SPEC_INSTALL_UX.md` 2.2 | Descriptions dimmed and indented | `crates/eden-skills-cli/src/commands/install.rs` (`print_discovery_preview`, `wrap_discovery_description`) | `crates/eden-skills-cli/tests/install_discovery_tests.rs` (`tm_p29_017_discovery_description_uses_indented_followup_line`, `tm_p29_018_discovery_skill_without_description_renders_name_only_line`) | completed |
| IUX-004 | `SPEC_INSTALL_UX.md` 3.2 | Step-style progress `[pos/len]` in TTY | `crates/eden-skills-cli/src/commands/install.rs` (`SourceSyncProgress`, `install_registry_mode_async`, `install_remote_url_mode_async`) | `crates/eden-skills-cli/tests/install_discovery_tests.rs` (`tm_p29_020_source_sync_shows_step_style_progress_in_tty`) | completed |
| IUX-005 | `SPEC_INSTALL_UX.md` 3.3 | Styled sync summary after completion | `crates/eden-skills-cli/src/commands/common.rs` (`print_source_sync_step_summary_human`), `crates/eden-skills-cli/src/commands/install.rs` (`SourceSyncProgress::finish`) | `crates/eden-skills-cli/tests/install_discovery_tests.rs` (`tm_p29_021_source_sync_prints_summary_line_after_completion`, `tm_p29_022_non_tty_source_sync_skips_progress_bar_and_keeps_summary`) | completed |
| IUX-006 | `SPEC_INSTALL_UX.md` 4.1–4.3 | Tree-style grouped install results | `crates/eden-skills-cli/src/commands/install.rs` (`print_install_result_lines`, `group_install_targets`) | `crates/eden-skills-cli/tests/install_discovery_tests.rs` (`tm_p29_023_install_results_use_tree_display_with_connectors`, `tm_p29_024_tree_groups_skill_name_once_per_skill_group`) | completed |
| IUX-007 | `SPEC_INSTALL_UX.md` 4.4 | Tree coloring: cyan paths, dimmed connectors | `crates/eden-skills-cli/src/commands/install.rs` (`style_tree_connector`, `style_mode_label`, `UiContext::styled_path`) | TM-P29-025 (manual) | completed |
| IUX-008 | `SPEC_INSTALL_UX.md` 4.7 | `apply`/`repair` use tree-style display | `crates/eden-skills-cli/src/commands/reconcile.rs` (`apply_async`, `repair_async`, `print_install_result_lines`, `group_install_targets`) | `crates/eden-skills-cli/tests/phase2_commands.rs` (`tm_p29_026_apply_and_repair_use_tree_style_install_lines`) | completed |
| IUX-009 | `SPEC_INSTALL_UX.md` 4.8 | Dry-run multi-skill preview uses titled indented skill/target tables with default 8-row truncation and `--list` full expansion; target table is `Agent/Path/Mode` only | `crates/eden-skills-cli/src/commands/install.rs` (`print_install_dry_run`, `build_dry_run_preview_data`, `print_titled_table`, `print_indented_block`) | `crates/eden-skills-cli/tests/install_discovery_tests.rs` (`dry_run_multi_skill_preview_defaults_to_eight_skill_rows`, `dry_run_multi_skill_with_list_shows_all_skill_rows`) | completed |

## 4. Output Consistency Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
| --- | --- | --- | --- | --- | --- |
| OCN-001 | `SPEC_OUTPUT_CONSISTENCY.md` 3.1 | `add` shows `✓ Added` | `crates/eden-skills-cli/src/commands/config_ops.rs` (`add`, `style_quoted_skill_id`) | `crates/eden-skills-cli/tests/output_consistency_tests.rs` (`tm_p29_028_add_shows_added_line_with_abbreviated_path`) | completed |
| OCN-002 | `SPEC_OUTPUT_CONSISTENCY.md` 3.2 | `set` shows `✓ Updated` | `crates/eden-skills-cli/src/commands/config_ops.rs` (`set`, `style_quoted_skill_id`) | `crates/eden-skills-cli/tests/output_consistency_tests.rs` (`tm_p29_029_set_shows_updated_line_with_abbreviated_path`) | completed |
| OCN-003 | `SPEC_OUTPUT_CONSISTENCY.md` 3.3 | `config import` shows `✓ Imported` | `crates/eden-skills-cli/src/commands/config_ops.rs` (`config_import`) | `crates/eden-skills-cli/tests/output_consistency_tests.rs` (`tm_p29_030_config_import_shows_imported_line_with_abbreviated_path`) | completed |
| OCN-004 | `SPEC_OUTPUT_CONSISTENCY.md` 3.4–3.8 | All warnings through `print_warning()` | `crates/eden-skills-cli/src/commands/config_ops.rs`, `crates/eden-skills-cli/src/commands/remove.rs`, `crates/eden-skills-cli/src/commands/common.rs` (`validate_registry_manifest_for_resolution`) | `crates/eden-skills-cli/tests/output_consistency_tests.rs` (`tm_p29_031_no_raw_warning_eprintln_remains_in_target_files`) | completed |
| OCN-005 | `SPEC_OUTPUT_CONSISTENCY.md` 3.5 | `remove` cancel uses skipped symbol | `crates/eden-skills-cli/src/commands/remove.rs` (`remove_many_async`) | `crates/eden-skills-cli/tests/output_consistency_tests.rs` (`tm_p29_032_remove_cancellation_uses_skipped_symbol_line`) | completed |
| OCN-006 | `SPEC_OUTPUT_CONSISTENCY.md` 3.6 | `remove` candidates render as table | `crates/eden-skills-cli/src/commands/remove.rs` (`print_remove_candidates`) | `crates/eden-skills-cli/tests/output_consistency_tests.rs` (`tm_p29_033_remove_interactive_candidates_render_as_table`) | completed |
| OCN-007 | `SPEC_OUTPUT_CONSISTENCY.md` 4.1 | File paths styled cyan | `crates/eden-skills-cli/src/ui.rs` (`UiContext::styled_path`), `crates/eden-skills-cli/src/commands/config_ops.rs`, `crates/eden-skills-cli/src/commands/install.rs`, `crates/eden-skills-cli/src/commands/reconcile.rs`, `crates/eden-skills-cli/src/commands/plan_cmd.rs`, `crates/eden-skills-cli/src/commands/diagnose.rs` | TM-P29-034 (manual) | completed |
| OCN-008 | `SPEC_OUTPUT_CONSISTENCY.md` 4.4 | Skill names bold in result lines | `crates/eden-skills-cli/src/commands/install.rs` (`style_skill_id`), `crates/eden-skills-cli/src/commands/reconcile.rs` (`style_skill_id`), `crates/eden-skills-cli/src/commands/remove.rs` (`style_skill_id`) | TM-P29-034 (manual) | completed |
| OCN-009 | `SPEC_OUTPUT_CONSISTENCY.md` 4.4 | Mode labels and connectors dimmed | `crates/eden-skills-cli/src/commands/install.rs` (`style_mode_label`, `style_tree_connector`), `crates/eden-skills-cli/src/commands/reconcile.rs` (`style_mode_label`, `style_tree_connector`), `crates/eden-skills-cli/src/commands/plan_cmd.rs` (`style_mode_label`, `style_arrow`) | TM-P29-034 (manual) | completed |
| OCN-010 | `SPEC_OUTPUT_CONSISTENCY.md` 4.2 | `UiContext::styled_path()` exists | `crates/eden-skills-cli/src/ui.rs` (`UiContext::styled_path`) | `crates/eden-skills-cli/tests/output_consistency_tests.rs` (`tm_p29_035_ui_context_exposes_styled_path_method`) | completed |

## 5. Newline Policy Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
| --- | --- | --- | --- | --- | --- |
| NLP-001 | `SPEC_NEWLINE_POLICY.md` 2.1 | No trailing blank line after output | Output-path audit across `crates/eden-skills-cli/src/commands/*` and targeted newline regressions in `main.rs`/`lib.rs` | `crates/eden-skills-cli/tests/newline_policy_tests.rs` (`tm_p29_039_list_doctor_plan_outputs_end_without_trailing_blank_lines`, `tm_p29_040_install_remove_update_outputs_end_without_trailing_blank_lines`) | completed |
| NLP-002 | `SPEC_NEWLINE_POLICY.md` 2.3 | Error: blank line only when hint exists | `crates/eden-skills-cli/src/main.rs` (`print_error`) | `crates/eden-skills-cli/tests/newline_policy_tests.rs` (`tm_p29_036_error_without_hint_has_no_trailing_blank_line`, `tm_p29_037_error_with_hint_has_single_separator_blank_line`) | completed |
| NLP-003 | `SPEC_NEWLINE_POLICY.md` 3.2 | Clap errors `.trim_end()` | `crates/eden-skills-cli/src/lib.rs` (clap parse error normalization) | `crates/eden-skills-cli/tests/newline_policy_tests.rs` (`tm_p29_038_clap_error_has_no_trailing_blank_lines`) | completed |
| NLP-004 | `SPEC_NEWLINE_POLICY.md` 2.2 | Section spacing per policy table | Section-spacing audit on human-mode output paths in `crates/eden-skills-cli/src/commands/*` | `crates/eden-skills-cli/tests/newline_policy_tests.rs` (`tm_p29_039_list_doctor_plan_outputs_end_without_trailing_blank_lines`) | completed |
| NLP-005 | `SPEC_NEWLINE_POLICY.md` 3.4 | Full output-path audit | End-of-output newline audit for `list`, `doctor`, `plan`, `install`, `remove`, `update` command paths | `crates/eden-skills-cli/tests/newline_policy_tests.rs` (`tm_p29_040_install_remove_update_outputs_end_without_trailing_blank_lines`) | completed |
| NLP-006 | `SPEC_NEWLINE_POLICY.md` 3.4 | No trailing empty `println!()` before `Ok(())` | End-of-function blank-line audit across `crates/eden-skills-cli/src/commands/*` | `crates/eden-skills-cli/tests/newline_policy_tests.rs` (`tm_p29_039_list_doctor_plan_outputs_end_without_trailing_blank_lines`, `tm_p29_040_install_remove_update_outputs_end_without_trailing_blank_lines`) | completed |
