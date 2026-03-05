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
| TBL-003 | `SPEC_TABLE_RENDERING.md` 5.1 | `list` MUST render as table | `crates/eden-skills-cli/src/commands/config_ops.rs` (`list`, `render_skill_agents`) | `output_upgrade_b_tests::tm_p28_005_list_renders_as_table`, `output_upgrade_b_tests::tm_p28_006_list_table_non_tty_degradation`, `output_upgrade_b_tests::tm_p28_007_list_table_json_unchanged`, `list_command::list_text_prints_inventory` | done |
| TBL-004 | `SPEC_TABLE_RENDERING.md` 5.2 | `install --dry-run` targets MUST render as table | `crates/eden-skills-cli/src/commands/install.rs` (`print_install_dry_run`, dry-run branches in `install_registry_mode_async` / `install_remote_url_mode_async` / `install_local_url_mode_async`) | `output_upgrade_b_tests::tm_p28_008_install_dry_run_renders_targets_table`, `phase2_commands::install_dry_run_displays_resolution_without_side_effects` | done |
| TBL-005 | `SPEC_TABLE_RENDERING.md` 5.3 | `install --list` MUST render as numbered table | `crates/eden-skills-cli/src/commands/install.rs` (`print_discovered_skills`) | `output_upgrade_b_tests::tm_p28_009_install_list_renders_numbered_table` | done |
| TBL-006 | `SPEC_TABLE_RENDERING.md` 5.4 | `plan` > 5 actions MUST render as table | `crates/eden-skills-cli/src/commands/plan_cmd.rs` (`print_plan_text`, `print_plan_table`, `style_plan_action_cell`) | `output_upgrade_b_tests::tm_p28_010_plan_table_threshold`, `output_upgrade_b_tests::tm_p28_029_non_tty_tables_use_ascii_borders` | done |
| TBL-007 | `SPEC_TABLE_RENDERING.md` 5.5 | `update` MUST render results as table | `crates/eden-skills-cli/src/commands/update.rs` (`update_async`, `format_registry_status`) | `output_upgrade_b_tests::tm_p28_011_update_renders_registry_table`, `phase2_commands::update_clones_configured_registries` | done |

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
| OUP-008 | `SPEC_OUTPUT_UPGRADE.md` 4.7 | `list` MUST render as table | `crates/eden-skills-cli/src/commands/config_ops.rs` (`list`, `render_skill_agents`) | `output_upgrade_b_tests::tm_p28_005_list_renders_as_table`, `list_command::list_text_prints_inventory` | done |
| OUP-009 | `SPEC_OUTPUT_UPGRADE.md` 4.9 | `install --dry-run` MUST render targets table | `crates/eden-skills-cli/src/commands/install.rs` (`print_install_dry_run`) | `output_upgrade_b_tests::tm_p28_008_install_dry_run_renders_targets_table`, `phase2_commands::install_dry_run_displays_resolution_without_side_effects` | done |
| OUP-010 | `SPEC_OUTPUT_UPGRADE.md` 4.7 | `install --list` MUST render numbered table | `crates/eden-skills-cli/src/commands/install.rs` (`print_discovered_skills`) | `output_upgrade_b_tests::tm_p28_009_install_list_renders_numbered_table` | done |
| OUP-011 | `SPEC_OUTPUT_UPGRADE.md` 4.3 | `plan` > 5 actions MUST render as table | `crates/eden-skills-cli/src/commands/plan_cmd.rs` (`print_plan_text`, `print_plan_table`) | `output_upgrade_b_tests::tm_p28_010_plan_table_threshold` | done |
| OUP-012 | `SPEC_OUTPUT_UPGRADE.md` 4.8 | `update` MUST render registry table | `crates/eden-skills-cli/src/commands/update.rs` (`update_async`, `format_registry_status`) | `output_upgrade_b_tests::tm_p28_011_update_renders_registry_table`, `phase2_commands::update_clones_configured_registries` | done |
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
| CST-001 | `SPEC_CODE_STRUCTURE.md` 2.1 | `commands.rs` MUST be decomposed into sub-modules | `crates/eden-skills-cli/src/commands/` (mod.rs, install.rs, reconcile.rs, diagnose.rs, plan_cmd.rs, config_ops.rs, remove.rs, update.rs, common.rs) | TM-P28-001 | done |
| CST-002 | `SPEC_CODE_STRUCTURE.md` 2.3 | Decomposition MUST NOT change behavior | All 253 original tests pass without modification | TM-P28-001, TM-P28-002 | done |
| CST-003 | `SPEC_CODE_STRUCTURE.md` 2.4 | Public API MUST remain unchanged | `mod.rs` re-exports preserve `eden_skills_cli::commands::*` paths | TM-P28-002 | done |
| CST-004 | `SPEC_CODE_STRUCTURE.md` 3.2 | CLI `commands/` modules MUST have `//!` docs | All 9 `commands/*.rs` files have `//!` module docs | TM-P28-003, TM-P28-034 | done |
| CST-005 | `SPEC_CODE_STRUCTURE.md` 3.2 | Public command functions MUST have `///` docs | `install_async`, `apply_async`, `repair_async`, `doctor`, `plan`, `init`, `list`, `add`, `set`, `config_export`, `config_import`, `remove_many_async`, `update_async` all have `///` + `# Errors` | TM-P28-034 | done |
| CST-006 | `SPEC_CODE_STRUCTURE.md` 3.3 | Core modules MUST have `//!` docs | `reactor.rs`, `lock.rs`, `adapter.rs`, `source_format.rs`, `discovery.rs`, `config.rs`, `plan.rs`, `error.rs` all have `//!` module docs | TM-P28-035 | done |
| CST-007 | `SPEC_CODE_STRUCTURE.md` 3.3 | Core public functions MUST have `///` docs | `SkillReactor`, `run_phase_a`, `run_blocking`, `compute_lock_diff`, `read_lock_file`, `TargetAdapter`, `LocalAdapter`, `DockerAdapter`, `detect_install_source`, `discover_skills`, `validate_config`, `LoadedConfig`, `build_plan`, `Action`, `EdenError`, `ReactorError`, `AdapterError`, `RegistryError` all documented | TM-P28-035 | done |
| CST-008 | `SPEC_CODE_STRUCTURE.md` 3.2 | `ui.rs` MUST have docs on all public items | `//!` module doc, `///` on `UiContext`, `UiSpinner`, `ColorWhen`, `StatusSymbol`, `configure_color_output`, `color_output_enabled`, `table()`, `spinner()`, `abbreviate_home_path`, `abbreviate_repo_url` | TM-P28-036 | done |

## 4. Test Matrix Coverage

| SCENARIO_ID | Source | Scenario | Automated Test | Status |
|---|---|---|---|---|
| TM-P28-001 | `SPEC_TEST_MATRIX.md` 2 | Commands module split regression | All 253 original tests pass after decomposition | done |
| TM-P28-002 | `SPEC_TEST_MATRIX.md` 2 | Public API unchanged | `lib.rs` and test imports compile without changes | done |
| TM-P28-003 | `SPEC_TEST_MATRIX.md` 2 | Module doc comments present | `doc_coverage_tests::tm_p28_003_commands_modules_have_module_docs`, `doc_coverage_tests::tm_p28_003_ui_has_module_doc`, `doc_coverage_tests::tm_p28_003_core_lib_has_crate_doc` | done |
| TM-P28-004 | `SPEC_TEST_MATRIX.md` 3 | comfy-table dependency present | `table_infra_tests::comfy_table_dependency_is_declared_in_cli_cargo_toml` | done |
| TM-P28-005 | `SPEC_TEST_MATRIX.md` 3 | List renders as table | `output_upgrade_b_tests::tm_p28_005_list_renders_as_table` | done |
| TM-P28-006 | `SPEC_TEST_MATRIX.md` 3 | List table non-TTY degradation | `output_upgrade_b_tests::tm_p28_006_list_table_non_tty_degradation` | done |
| TM-P28-007 | `SPEC_TEST_MATRIX.md` 3 | List table JSON unchanged | `output_upgrade_b_tests::tm_p28_007_list_table_json_unchanged` | done |
| TM-P28-008 | `SPEC_TEST_MATRIX.md` 3 | Install dry-run renders targets table | `output_upgrade_b_tests::tm_p28_008_install_dry_run_renders_targets_table` | done |
| TM-P28-009 | `SPEC_TEST_MATRIX.md` 3 | Install list renders numbered table | `output_upgrade_b_tests::tm_p28_009_install_list_renders_numbered_table` | done |
| TM-P28-010 | `SPEC_TEST_MATRIX.md` 3 | Plan table threshold | `output_upgrade_b_tests::tm_p28_010_plan_table_threshold` | done |
| TM-P28-011 | `SPEC_TEST_MATRIX.md` 3 | Update renders registry table | `output_upgrade_b_tests::tm_p28_011_update_renders_registry_table` | done |
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
| TM-P28-029 | `SPEC_TEST_MATRIX.md` 6 | Non-TTY tables use ASCII borders | `output_upgrade_b_tests::tm_p28_029_non_tty_tables_use_ascii_borders` | done |
| TM-P28-030 | `SPEC_TEST_MATRIX.md` 6 | Color never disables table styling | `output_upgrade_b_tests::tm_p28_030_color_never_disables_table_styling` | done |
| TM-P28-031 | `SPEC_TEST_MATRIX.md` 6 | JSON mode never renders tables | `output_upgrade_b_tests::tm_p28_031_json_mode_never_renders_tables` | done |
| TM-P28-032 | `SPEC_TEST_MATRIX.md` 7 | Home path abbreviated | `table_infra_tests::abbreviate_home_path_replaces_home_prefix_and_preserves_non_home_paths` | done |
| TM-P28-033 | `SPEC_TEST_MATRIX.md` 7 | Repo URL abbreviated | `table_infra_tests::abbreviate_repo_url_extracts_github_owner_and_repo` | done |
| TM-P28-034 | `SPEC_TEST_MATRIX.md` 8 | CLI module docs | `doc_coverage_tests::tm_p28_034_public_command_functions_have_doc_comments` | done |
| TM-P28-035 | `SPEC_TEST_MATRIX.md` 8 | Core module docs | `doc_coverage_tests::tm_p28_035_core_modules_have_module_docs` | done |
| TM-P28-036 | `SPEC_TEST_MATRIX.md` 8 | UiContext documented | `doc_coverage_tests::tm_p28_036_ui_public_items_have_doc_comments` | done |
| TM-P28-037 | `SPEC_TEST_MATRIX.md` 9 | Full regression | `cargo test --workspace` — 299 tests pass across all Phase 1/2/2.5/2.7/2.8 suites | done |
| TM-P28-038 | `SPEC_TEST_MATRIX.md` 9 | JSON regression | All `--json` test assertions unmodified: `doctor_json_contract`, `plan_json_contract`, `install_json_output_keeps_contract`, `list_table_json_unchanged`, `json_mode_never_renders_tables` | done |
| TM-P28-039 | `SPEC_TEST_MATRIX.md` 9 | Exit code regression | `exit_code_matrix` — all 11 tests pass, exit codes 0/1/2/3 semantics unchanged | done |
| TM-P28-040 | `SPEC_TEST_MATRIX.md` 9 | No hardcoded ANSI regression | `rg '\u{1b}\[' crates/` — zero matches in source code | done |
