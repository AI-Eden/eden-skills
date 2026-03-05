# SPEC_TRACEABILITY.md

Requirement-to-implementation mapping for Phase 2.8.
Use this file to recover accurate context after compression.

**Status:** DRAFT — populated with requirement IDs and test scenario
mappings. Implementation and test columns will be filled during
Builder execution.

## 1. Table Rendering Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
|---|---|---|---|---|---|
| TBL-001 | `SPEC_TABLE_RENDERING.md` 2.1 | `comfy-table` MUST be added as dependency | `crates/eden-skills-cli/Cargo.toml` (`comfy-table = "7"`) | `table_infra_tests::comfy_table_dependency_is_declared_in_cli_cargo_toml` (TM-P28-004) | done |
| TBL-002 | `SPEC_TABLE_RENDERING.md` 3.1 | `UiContext` MUST provide `table()` factory | `crates/eden-skills-cli/src/ui.rs` (`UiContext::table`, `abbreviate_home_path`, `abbreviate_repo_url`) | `table_infra_tests::ui_context_table_uses_utf8_borders_and_bold_headers_on_tty`, `table_infra_tests::ui_context_table_uses_ascii_borders_and_wraps_to_80_on_non_tty`, `table_infra_tests::abbreviate_home_path_replaces_home_prefix_and_preserves_non_home_paths` (TM-P28-032), `table_infra_tests::abbreviate_repo_url_extracts_github_owner_and_repo` (TM-P28-033) | done |
| TBL-003 | `SPEC_TABLE_RENDERING.md` 5.1 | `list` MUST render as table | | TM-P28-005, TM-P28-006, TM-P28-007 | pending |
| TBL-004 | `SPEC_TABLE_RENDERING.md` 5.2 | `install --dry-run` targets MUST render as table | | TM-P28-008 | pending |
| TBL-005 | `SPEC_TABLE_RENDERING.md` 5.3 | `install --list` MUST render as numbered table | | TM-P28-009 | pending |
| TBL-006 | `SPEC_TABLE_RENDERING.md` 5.4 | `plan` > 5 actions MUST render as table | | TM-P28-010 | pending |
| TBL-007 | `SPEC_TABLE_RENDERING.md` 5.5 | `update` MUST render results as table | | TM-P28-011 | pending |

## 2. Output Upgrade Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
|---|---|---|---|---|---|
| OUP-001 | `SPEC_OUTPUT_UPGRADE.md` 4.1 | `apply` human-mode MUST match spec | `crates/eden-skills-cli/src/commands/reconcile.rs` (`apply_async`, `print_install_applied_line`, `print_install_skipped_line`, `print_remove_lines`), `crates/eden-skills-cli/src/commands/common.rs` (`print_source_sync_summary_human`, `print_safety_summary_human`, `style_count_for_action`) | `output_upgrade_a1_tests::tm_p28_012_apply_source_sync_is_styled`, `output_upgrade_a1_tests::tm_p28_013_apply_safety_summary_is_styled`, `output_upgrade_a1_tests::tm_p28_014_apply_per_skill_install_lines`, `output_upgrade_a1_tests::tm_p28_015_apply_summary_is_styled`, `output_upgrade_a1_tests::tm_p28_016_apply_verification_is_styled` | done |
| OUP-002 | `SPEC_OUTPUT_UPGRADE.md` 4.1 | `repair` human-mode MUST match spec | `crates/eden-skills-cli/src/commands/reconcile.rs` (`repair_async`, styled install/summary/verification output) | `output_upgrade_a1_tests::tm_p28_017_repair_output_matches_apply_format` | done |
| OUP-003 | `SPEC_OUTPUT_UPGRADE.md` 4.2 | `doctor` MUST show finding cards | `crates/eden-skills-cli/src/commands/diagnose.rs` (`doctor`, `print_doctor_text`, `doctor_severity_symbol`, `doctor_remediation_prefix`) | `output_upgrade_a2_tests::tm_p28_018_doctor_header_styled`, `output_upgrade_a2_tests::tm_p28_019_doctor_findings_cards` | done |
| OUP-004 | `SPEC_OUTPUT_UPGRADE.md` 4.3 | `plan` MUST show header and colored actions | `crates/eden-skills-cli/src/commands/plan_cmd.rs` (`plan`, `print_plan_text`, `style_plan_action_label`) | `output_upgrade_a1_tests::tm_p28_021_plan_header_and_colored_actions`, `output_upgrade_a1_tests::tm_p28_022_plan_empty_state` | done |
| OUP-005 | `SPEC_OUTPUT_UPGRADE.md` 4.4 | `init` MUST show `✓` and Next steps | `crates/eden-skills-cli/src/commands/config_ops.rs` (`init`, `print_init_next_step`) | `output_upgrade_a2_tests::tm_p28_023_init_next_steps` | done |
| OUP-006 | `SPEC_OUTPUT_UPGRADE.md` 4.5 | `install` URL-mode MUST emit per-skill lines | `crates/eden-skills-cli/src/commands/install.rs` (`install_remote_url_mode_async`, `install_local_url_mode_async`, `execute_install_plan`, `install_local_source_skill`, `print_install_result_lines`) | `output_upgrade_a2_tests::tm_p28_024_install_per_skill_results` | done |
| OUP-007 | `SPEC_OUTPUT_UPGRADE.md` 4.6 | `install` discovery MUST use numbered list | `crates/eden-skills-cli/src/commands/install.rs` (`resolve_local_install_selection`, `print_discovery_summary`) | `output_upgrade_a2_tests::tm_p28_025_install_discovery_numbered` | done |
| OUP-008 | `SPEC_OUTPUT_UPGRADE.md` 4.7 | `list` MUST render as table | | TM-P28-005 | pending |
| OUP-009 | `SPEC_OUTPUT_UPGRADE.md` 4.9 | `install --dry-run` MUST render targets table | | TM-P28-008 | pending |
| OUP-010 | `SPEC_OUTPUT_UPGRADE.md` 4.7 | `install --list` MUST render numbered table | | TM-P28-009 | pending |
| OUP-011 | `SPEC_OUTPUT_UPGRADE.md` 4.3 | `plan` > 5 actions MUST render as table | | TM-P28-010 | pending |
| OUP-012 | `SPEC_OUTPUT_UPGRADE.md` 4.8 | `update` MUST render registry table | | TM-P28-011 | pending |
| OUP-013 | `SPEC_OUTPUT_UPGRADE.md` 5.1 | Error hint MUST use `→` not `hint:` | `crates/eden-skills-cli/src/main.rs` (`print_error`) | `output_upgrade_a1_tests::tm_p28_026_error_hint_uses_arrow`, `output_polish_tests::error_output_uses_error_prefix_and_hint_for_missing_config`, `output_polish_tests::remove_unknown_skill_includes_available_skills_hint` | done |
| OUP-014 | `SPEC_OUTPUT_UPGRADE.md` 5.3 | Error paths MUST be abbreviated with `~` | `crates/eden-skills-cli/src/commands/common.rs` (`load_config_with_context`), `crates/eden-skills-cli/src/main.rs` (`abbreviate_message_paths`) | `output_upgrade_a1_tests::tm_p28_027_error_path_is_abbreviated` | done |
| OUP-015 | `SPEC_OUTPUT_UPGRADE.md` 2.1 | All commands MUST use `UiContext` for human output | `crates/eden-skills-cli/src/commands/reconcile.rs` (`apply_async`, `repair_async`), `crates/eden-skills-cli/src/commands/plan_cmd.rs` (`plan`), `crates/eden-skills-cli/src/commands/common.rs` (`print_warning`) | `output_upgrade_a1_tests::tm_p28_012_apply_source_sync_is_styled`, `output_upgrade_a1_tests::tm_p28_017_repair_output_matches_apply_format`, `output_upgrade_a1_tests::tm_p28_021_plan_header_and_colored_actions`, `output_upgrade_a1_tests::tm_p28_022_plan_empty_state` | done |
| OUP-016 | `SPEC_OUTPUT_UPGRADE.md` 3 | Action colors MUST follow palette | `crates/eden-skills-cli/src/commands/plan_cmd.rs` (`style_plan_action_label`), `crates/eden-skills-cli/src/commands/common.rs` (`style_count_for_action`) | `output_upgrade_a1_tests::tm_p28_021_plan_header_and_colored_actions` | done |
| OUP-017 | `SPEC_OUTPUT_UPGRADE.md` 6 | Warnings MUST use yellow bold with indent | `crates/eden-skills-cli/src/commands/common.rs` (`print_warning`), `crates/eden-skills-cli/src/commands/plan_cmd.rs`, `crates/eden-skills-cli/src/commands/reconcile.rs` | `output_upgrade_a1_tests::tm_p28_028_warning_format_is_styled` | done |
| OUP-018 | `SPEC_OUTPUT_UPGRADE.md` 4.5 | Install summary MUST include skill/agent/conflict count | `crates/eden-skills-cli/src/commands/install.rs` (`print_install_result_summary`, `unique_agent_count`) | `output_upgrade_a2_tests::tm_p28_024_install_per_skill_results` | done |
| OUP-019 | `SPEC_OUTPUT_UPGRADE.md` 4.1 | Apply/repair MUST use action prefixes for sync/safety/summary | `crates/eden-skills-cli/src/commands/common.rs` (`print_source_sync_summary_human`, `print_safety_summary_human`), `crates/eden-skills-cli/src/commands/reconcile.rs` (`Summary` action prefix output) | `output_upgrade_a1_tests::tm_p28_012_apply_source_sync_is_styled`, `output_upgrade_a1_tests::tm_p28_013_apply_safety_summary_is_styled`, `output_upgrade_a1_tests::tm_p28_015_apply_summary_is_styled` | done |
| OUP-020 | `SPEC_OUTPUT_UPGRADE.md` 4.2 | Doctor summary table MUST show when findings > 3 | `crates/eden-skills-cli/src/commands/diagnose.rs` (`print_doctor_text` summary table branch via `UiContext::table`) | `output_upgrade_a2_tests::tm_p28_020_doctor_summary_table_conditional` | done |

## 3. Code Structure Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
|---|---|---|---|---|---|
| CST-001 | `SPEC_CODE_STRUCTURE.md` 2.1 | `commands.rs` MUST be decomposed into sub-modules | | TM-P28-001 | pending |
| CST-002 | `SPEC_CODE_STRUCTURE.md` 2.3 | Decomposition MUST NOT change behavior | | TM-P28-001, TM-P28-002 | pending |
| CST-003 | `SPEC_CODE_STRUCTURE.md` 2.4 | Public API MUST remain unchanged | | TM-P28-002 | pending |
| CST-004 | `SPEC_CODE_STRUCTURE.md` 3.2 | CLI `commands/` modules MUST have `//!` docs | | TM-P28-003, TM-P28-034 | pending |
| CST-005 | `SPEC_CODE_STRUCTURE.md` 3.2 | Public command functions MUST have `///` docs | | TM-P28-034 | pending |
| CST-006 | `SPEC_CODE_STRUCTURE.md` 3.3 | Core modules MUST have `//!` docs | | TM-P28-035 | pending |
| CST-007 | `SPEC_CODE_STRUCTURE.md` 3.3 | Core public functions MUST have `///` docs | | TM-P28-035 | pending |
| CST-008 | `SPEC_CODE_STRUCTURE.md` 3.2 | `ui.rs` MUST have docs on all public items | | TM-P28-036 | pending |

## 4. Test Matrix Coverage

| SCENARIO_ID | Source | Scenario | Automated Test | Status |
|---|---|---|---|---|
| TM-P28-001 | `SPEC_TEST_MATRIX.md` 2 | Commands module split regression | | pending |
| TM-P28-002 | `SPEC_TEST_MATRIX.md` 2 | Public API unchanged | | pending |
| TM-P28-003 | `SPEC_TEST_MATRIX.md` 2 | Module doc comments present | | pending |
| TM-P28-004 | `SPEC_TEST_MATRIX.md` 3 | comfy-table dependency present | `table_infra_tests::comfy_table_dependency_is_declared_in_cli_cargo_toml` | done |
| TM-P28-005 | `SPEC_TEST_MATRIX.md` 3 | List renders as table | | pending |
| TM-P28-006 | `SPEC_TEST_MATRIX.md` 3 | List table non-TTY degradation | | pending |
| TM-P28-007 | `SPEC_TEST_MATRIX.md` 3 | List table JSON unchanged | | pending |
| TM-P28-008 | `SPEC_TEST_MATRIX.md` 3 | Install dry-run renders targets table | | pending |
| TM-P28-009 | `SPEC_TEST_MATRIX.md` 3 | Install list renders numbered table | | pending |
| TM-P28-010 | `SPEC_TEST_MATRIX.md` 3 | Plan table threshold | | pending |
| TM-P28-011 | `SPEC_TEST_MATRIX.md` 3 | Update renders registry table | | pending |
| TM-P28-012 | `SPEC_TEST_MATRIX.md` 4 | Apply source sync styled | `output_upgrade_a1_tests::tm_p28_012_apply_source_sync_is_styled` | done |
| TM-P28-013 | `SPEC_TEST_MATRIX.md` 4 | Apply safety summary styled | `output_upgrade_a1_tests::tm_p28_013_apply_safety_summary_is_styled` | done |
| TM-P28-014 | `SPEC_TEST_MATRIX.md` 4 | Apply per-skill install lines | `output_upgrade_a1_tests::tm_p28_014_apply_per_skill_install_lines` | done |
| TM-P28-015 | `SPEC_TEST_MATRIX.md` 4 | Apply summary styled | `output_upgrade_a1_tests::tm_p28_015_apply_summary_is_styled` | done |
| TM-P28-016 | `SPEC_TEST_MATRIX.md` 4 | Apply verification styled | `output_upgrade_a1_tests::tm_p28_016_apply_verification_is_styled` | done |
| TM-P28-017 | `SPEC_TEST_MATRIX.md` 4 | Repair output matches apply format | `output_upgrade_a1_tests::tm_p28_017_repair_output_matches_apply_format` | done |
| TM-P28-018 | `SPEC_TEST_MATRIX.md` 4 | Doctor header styled | `output_upgrade_a2_tests::tm_p28_018_doctor_header_styled` | done |
| TM-P28-019 | `SPEC_TEST_MATRIX.md` 4 | Doctor findings cards | `output_upgrade_a2_tests::tm_p28_019_doctor_findings_cards` | done |
| TM-P28-020 | `SPEC_TEST_MATRIX.md` 4 | Doctor summary table conditional | `output_upgrade_a2_tests::tm_p28_020_doctor_summary_table_conditional` | done |
| TM-P28-021 | `SPEC_TEST_MATRIX.md` 4 | Plan header and colored actions | `output_upgrade_a1_tests::tm_p28_021_plan_header_and_colored_actions` | done |
| TM-P28-022 | `SPEC_TEST_MATRIX.md` 4 | Plan empty state | `output_upgrade_a1_tests::tm_p28_022_plan_empty_state` | done |
| TM-P28-023 | `SPEC_TEST_MATRIX.md` 4 | Init next steps | `output_upgrade_a2_tests::tm_p28_023_init_next_steps` | done |
| TM-P28-024 | `SPEC_TEST_MATRIX.md` 4 | Install per-skill results | `output_upgrade_a2_tests::tm_p28_024_install_per_skill_results` | done |
| TM-P28-025 | `SPEC_TEST_MATRIX.md` 4 | Install discovery numbered | `output_upgrade_a2_tests::tm_p28_025_install_discovery_numbered` | done |
| TM-P28-026 | `SPEC_TEST_MATRIX.md` 5 | Error hint uses arrow | `output_upgrade_a1_tests::tm_p28_026_error_hint_uses_arrow` | done |
| TM-P28-027 | `SPEC_TEST_MATRIX.md` 5 | Error path abbreviated | `output_upgrade_a1_tests::tm_p28_027_error_path_is_abbreviated` | done |
| TM-P28-028 | `SPEC_TEST_MATRIX.md` 5 | Warning format styled | `output_upgrade_a1_tests::tm_p28_028_warning_format_is_styled` | done |
| TM-P28-029 | `SPEC_TEST_MATRIX.md` 6 | Non-TTY tables use ASCII borders | | pending |
| TM-P28-030 | `SPEC_TEST_MATRIX.md` 6 | Color never disables table styling | | pending |
| TM-P28-031 | `SPEC_TEST_MATRIX.md` 6 | JSON mode never renders tables | | pending |
| TM-P28-032 | `SPEC_TEST_MATRIX.md` 7 | Home path abbreviated | `table_infra_tests::abbreviate_home_path_replaces_home_prefix_and_preserves_non_home_paths` | done |
| TM-P28-033 | `SPEC_TEST_MATRIX.md` 7 | Repo URL abbreviated | `table_infra_tests::abbreviate_repo_url_extracts_github_owner_and_repo` | done |
| TM-P28-034 | `SPEC_TEST_MATRIX.md` 8 | CLI module docs | | pending |
| TM-P28-035 | `SPEC_TEST_MATRIX.md` 8 | Core module docs | | pending |
| TM-P28-036 | `SPEC_TEST_MATRIX.md` 8 | UiContext documented | | pending |
| TM-P28-037 | `SPEC_TEST_MATRIX.md` 9 | Full regression | | pending |
| TM-P28-038 | `SPEC_TEST_MATRIX.md` 9 | JSON regression | | pending |
| TM-P28-039 | `SPEC_TEST_MATRIX.md` 9 | Exit code regression | | pending |
| TM-P28-040 | `SPEC_TEST_MATRIX.md` 9 | No hardcoded ANSI regression | | pending |
