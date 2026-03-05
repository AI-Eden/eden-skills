# SPEC_TRACEABILITY.md

Requirement-to-implementation mapping for Phase 2.9.
Use this file to recover accurate context after compression.

**Status:** DRAFT — populated with requirement IDs. Implementation
and test columns will be filled during Builder execution.

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
| IUX-001 | `SPEC_INSTALL_UX.md` 2.2 | Discovery preview MUST use card-style numbered list | | TM-P29-015, TM-P29-019 | pending |
| IUX-002 | `SPEC_INSTALL_UX.md` 2.1 | Merge two discovery functions into one | | TM-P29-016 | pending |
| IUX-003 | `SPEC_INSTALL_UX.md` 2.2 | Descriptions dimmed and indented | | TM-P29-017, TM-P29-018 | pending |
| IUX-004 | `SPEC_INSTALL_UX.md` 3.2 | Step-style progress `[pos/len]` in TTY | | TM-P29-020 | pending |
| IUX-005 | `SPEC_INSTALL_UX.md` 3.3 | Styled sync summary after completion | | TM-P29-021, TM-P29-022 | pending |
| IUX-006 | `SPEC_INSTALL_UX.md` 4.1–4.3 | Tree-style grouped install results | | TM-P29-023, TM-P29-024 | pending |
| IUX-007 | `SPEC_INSTALL_UX.md` 4.4 | Tree coloring: cyan paths, dimmed connectors | | TM-P29-025 | pending |
| IUX-008 | `SPEC_INSTALL_UX.md` 4.7 | `apply`/`repair` use tree-style display | | TM-P29-026 | pending |

## 4. Output Consistency Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
| --- | --- | --- | --- | --- | --- |
| OCN-001 | `SPEC_OUTPUT_CONSISTENCY.md` 3.1 | `add` shows `✓ Added` | | TM-P29-028 | pending |
| OCN-002 | `SPEC_OUTPUT_CONSISTENCY.md` 3.2 | `set` shows `✓ Updated` | | TM-P29-029 | pending |
| OCN-003 | `SPEC_OUTPUT_CONSISTENCY.md` 3.3 | `config import` shows `✓ Imported` | | TM-P29-030 | pending |
| OCN-004 | `SPEC_OUTPUT_CONSISTENCY.md` 3.4–3.8 | All warnings through `print_warning()` | | TM-P29-031 | pending |
| OCN-005 | `SPEC_OUTPUT_CONSISTENCY.md` 3.5 | `remove` cancel uses skipped symbol | | TM-P29-032 | pending |
| OCN-006 | `SPEC_OUTPUT_CONSISTENCY.md` 3.6 | `remove` candidates render as table | | TM-P29-033 | pending |
| OCN-007 | `SPEC_OUTPUT_CONSISTENCY.md` 4.1 | File paths styled cyan | | TM-P29-034 | pending |
| OCN-008 | `SPEC_OUTPUT_CONSISTENCY.md` 4.4 | Skill names bold in result lines | | TM-P29-034 | pending |
| OCN-009 | `SPEC_OUTPUT_CONSISTENCY.md` 4.4 | Mode labels and connectors dimmed | | TM-P29-034 | pending |
| OCN-010 | `SPEC_OUTPUT_CONSISTENCY.md` 4.2 | `UiContext::styled_path()` exists | | TM-P29-035 | pending |

## 5. Newline Policy Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
| --- | --- | --- | --- | --- | --- |
| NLP-001 | `SPEC_NEWLINE_POLICY.md` 2.1 | No trailing blank line after output | Output-path audit across `crates/eden-skills-cli/src/commands/*` and targeted newline regressions in `main.rs`/`lib.rs` | `crates/eden-skills-cli/tests/newline_policy_tests.rs` (`tm_p29_039_list_doctor_plan_outputs_end_without_trailing_blank_lines`, `tm_p29_040_install_remove_update_outputs_end_without_trailing_blank_lines`) | completed |
| NLP-002 | `SPEC_NEWLINE_POLICY.md` 2.3 | Error: blank line only when hint exists | `crates/eden-skills-cli/src/main.rs` (`print_error`) | `crates/eden-skills-cli/tests/newline_policy_tests.rs` (`tm_p29_036_error_without_hint_has_no_trailing_blank_line`, `tm_p29_037_error_with_hint_has_single_separator_blank_line`) | completed |
| NLP-003 | `SPEC_NEWLINE_POLICY.md` 3.2 | Clap errors `.trim_end()` | `crates/eden-skills-cli/src/lib.rs` (clap parse error normalization) | `crates/eden-skills-cli/tests/newline_policy_tests.rs` (`tm_p29_038_clap_error_has_no_trailing_blank_lines`) | completed |
| NLP-004 | `SPEC_NEWLINE_POLICY.md` 2.2 | Section spacing per policy table | Section-spacing audit on human-mode output paths in `crates/eden-skills-cli/src/commands/*` | `crates/eden-skills-cli/tests/newline_policy_tests.rs` (`tm_p29_039_list_doctor_plan_outputs_end_without_trailing_blank_lines`) | completed |
| NLP-005 | `SPEC_NEWLINE_POLICY.md` 3.4 | Full output-path audit | End-of-output newline audit for `list`, `doctor`, `plan`, `install`, `remove`, `update` command paths | `crates/eden-skills-cli/tests/newline_policy_tests.rs` (`tm_p29_040_install_remove_update_outputs_end_without_trailing_blank_lines`) | completed |
| NLP-006 | `SPEC_NEWLINE_POLICY.md` 3.4 | No trailing empty `println!()` before `Ok(())` | End-of-function blank-line audit across `crates/eden-skills-cli/src/commands/*` | `crates/eden-skills-cli/tests/newline_policy_tests.rs` (`tm_p29_039_list_doctor_plan_outputs_end_without_trailing_blank_lines`, `tm_p29_040_install_remove_update_outputs_end_without_trailing_blank_lines`) | completed |
