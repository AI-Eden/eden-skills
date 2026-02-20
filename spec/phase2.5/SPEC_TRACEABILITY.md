# SPEC_TRACEABILITY.md

Requirement-to-implementation mapping for Phase 2.5.
Use this file to recover accurate context after compression.

## 1. Install URL Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
|---|---|---|---|---|---|
| MVP-001 | `SPEC_INSTALL_URL.md` 3.1 | `install` MUST accept GitHub shorthand (`owner/repo`) | `crates/eden-skills-core/src/source_format.rs` (`detect_install_source`) | `crates/eden-skills-core/tests/source_format_tests.rs` (`github_shorthand_expands_to_https_git_url`) | completed |
| MVP-002 | `SPEC_INSTALL_URL.md` 3.1 | `install` MUST accept full GitHub/GitLab HTTPS URLs | `crates/eden-skills-core/src/source_format.rs` (`normalize_full_url`) | `crates/eden-skills-core/tests/source_format_tests.rs` (`full_github_url_appends_git_suffix_when_missing`) | completed |
| MVP-003 | `SPEC_INSTALL_URL.md` 3.4 | `install` MUST accept GitHub tree URLs and extract repo, ref, subpath | `crates/eden-skills-core/src/source_format.rs` (`parse_github_tree_url`) | `crates/eden-skills-core/tests/source_format_tests.rs` (`github_tree_url_extracts_repo_ref_and_subpath`) | completed |
| MVP-004 | `SPEC_INSTALL_URL.md` 3.1 | `install` MUST accept Git SSH URLs | `crates/eden-skills-core/src/source_format.rs` (`is_ssh_url`, `detect_install_source`) | `crates/eden-skills-core/tests/source_format_tests.rs` (`git_ssh_url_is_accepted_as_url_mode`) | completed |
| MVP-005 | `SPEC_INSTALL_URL.md` 3.1 | `install` MUST accept local paths | `crates/eden-skills-cli/src/commands.rs` (`install_url_mode_async`, `install_local_source_skill`) | `crates/eden-skills-cli/tests/install_url_tests.rs` (`local_path_install_persists_absolute_repo_and_skips_clone`) | completed |
| MVP-006 | `SPEC_INSTALL_URL.md` 3.2 | Source format detection MUST follow documented precedence | `crates/eden-skills-core/src/source_format.rs` (`detect_install_source`) | `crates/eden-skills-core/tests/source_format_tests.rs` (`local_path_is_detected_before_shorthand`, `unmatched_source_falls_back_to_registry_name`) | completed |
| MVP-007 | `SPEC_INSTALL_URL.md` 3.3 | Skill ID MUST be auto-derived with `--id` override | `crates/eden-skills-core/src/source_format.rs` (`derive_skill_id_from_source_repo`), `crates/eden-skills-cli/src/commands.rs` (`upsert_mode_a_skill`) | `crates/eden-skills-core/tests/source_format_tests.rs` (`auto_derived_id_uses_repo_tail_without_git_suffix`), `crates/eden-skills-cli/tests/install_url_tests.rs` (`install_url_mode_respects_id_override`, `install_url_mode_upserts_existing_id_instead_of_duplicating`) | completed |
| MVP-008 | `SPEC_INSTALL_URL.md` 6.2 | `install` MUST auto-create config if not exists | `crates/eden-skills-cli/src/commands.rs` (`ensure_install_config_exists`) | `crates/eden-skills-cli/tests/install_url_tests.rs` (`local_path_install_persists_absolute_repo_and_skips_clone`, `install_fails_when_config_parent_directory_is_missing`) | completed |
| MVP-009 | `SPEC_INSTALL_URL.md` 4.2~4.6 | `install` MUST discover SKILL.md via standard directories, plugin manifests, and bounded recursive fallback | `crates/eden-skills-core/src/discovery.rs` (`discover_skills`, `discover_from_plugin_manifests`, `discover_recursive_fallback`), `crates/eden-skills-cli/src/commands.rs` (`install_local_url_mode_async`, `install_remote_url_mode_async`, `discover_remote_skills_via_temp_clone`) | `crates/eden-skills-core/tests/discovery_tests.rs`, `crates/eden-skills-cli/tests/install_discovery_tests.rs` | completed |
| MVP-010 | `SPEC_INSTALL_URL.md` 5.6 | `--list` MUST display discovered skills without installing | `crates/eden-skills-cli/src/commands.rs` (`print_discovered_skills`, URL-mode list branches, list no-config path in `install_async`) | `crates/eden-skills-cli/tests/install_discovery_tests.rs` (`list_flag_prints_discovered_skills_without_modifying_config`, `remote_url_list_does_not_create_config_or_install_targets`) | completed |
| MVP-011 | `SPEC_INSTALL_URL.md` 5.1 | `--all` MUST install all discovered skills without confirmation | `crates/eden-skills-cli/src/commands.rs` (`resolve_local_install_selection`) | `crates/eden-skills-cli/tests/install_discovery_tests.rs` (`skills_directory_discovery_with_all_installs_all_skills`, `packages_directory_discovery_with_all_installs_all_skills`, `remote_url_with_all_installs_all_discovered_skills`) | completed |
| MVP-012 | `SPEC_INSTALL_URL.md` 5.2 | `--skill` MUST install only named skills and fail on unknown names without root fallback | `crates/eden-skills-cli/src/commands.rs` (`resolve_local_install_selection`, `select_named_skills`, URL-mode empty-discovery guards) | `crates/eden-skills-cli/tests/install_discovery_tests.rs` (`skill_flags_install_only_selected_skills`, `remote_url_with_skill_installs_only_selected_skill`, `unknown_skill_name_returns_error_with_available_names`, `single_discovered_skill_with_unmatched_skill_flag_returns_error`, `missing_skill_markdown_with_skill_flag_returns_error_instead_of_root_fallback`, `remote_url_missing_skill_markdown_with_skill_flag_returns_error`) | completed |
| MVP-013 | `SPEC_INSTALL_URL.md` 5.3 | Interactive mode MUST show skills and prompt for confirmation | `crates/eden-skills-cli/src/commands.rs` (`print_discovery_summary`, `prompt_install_all`, `prompt_skill_names`) | `crates/eden-skills-cli/tests/install_discovery_tests.rs` (`interactive_tty_confirm_yes_installs_all`, `interactive_tty_confirm_no_then_selects_named_skills`, `interactive_summary_truncates_when_more_than_eight_skills`) | completed |
| MVP-014 | `SPEC_INSTALL_URL.md` 5.5 | Non-TTY MUST default to `--all` behavior | `crates/eden-skills-cli/src/ui.rs` (`UiContext::interactive_enabled`), `crates/eden-skills-cli/src/commands.rs` (`resolve_local_install_selection`) | `crates/eden-skills-cli/tests/install_discovery_tests.rs` (`non_tty_defaults_to_install_all_for_multi_skill_repo`) | completed |
| MVP-015 | `SPEC_INSTALL_URL.md` 4.3 | Single-skill repos MUST skip confirmation | `crates/eden-skills-cli/src/commands.rs` (`resolve_local_install_selection`) | `crates/eden-skills-cli/tests/install_discovery_tests.rs` (`single_root_skill_installs_without_confirmation_prompt`) | completed |

## 2. Schema Amendment Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
|---|---|---|---|---|---|
| SCH-P25-001 | `SPEC_SCHEMA_P25.md` 2.2 | `skills` array MAY be empty | `crates/eden-skills-core/src/config.rs` (`RawConfig::into_config`, `validate_config`) | `crates/eden-skills-core/tests/config_tests.rs`, `crates/eden-skills-cli/tests/phase25_schema_tests.rs` | completed |
| SCH-P25-002 | `SPEC_SCHEMA_P25.md` 3.2 | `init` template MUST produce minimal config without dummy skills | `crates/eden-skills-cli/src/commands.rs` (`default_config_template`) | `crates/eden-skills-cli/tests/init_command.rs` | completed |
| SCH-P25-003 | `SPEC_SCHEMA_P25.md` 5 | Phase 1 and Phase 2 configs remain valid | `crates/eden-skills-core/src/config.rs` (backward-compatible non-empty skill validation path retained) | `crates/eden-skills-core/tests/config_tests.rs`, `cargo test --workspace` regression gate | completed |

## 3. Agent Detection Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
|---|---|---|---|---|---|
| AGT-001 | `SPEC_AGENT_DETECT.md` 2.2 | `install` MUST auto-detect installed agents | `crates/eden-skills-core/src/agents.rs` (`detect_installed_agent_targets`), `crates/eden-skills-cli/src/commands.rs` (`resolve_url_mode_install_targets`) | `crates/eden-skills-cli/tests/install_agent_detect_tests.rs` (`install_without_target_detects_multiple_agent_directories`) | completed |
| AGT-002 | `SPEC_AGENT_DETECT.md` 2.1 | Detection MUST check documented agent directories | `crates/eden-skills-core/src/agents.rs` (`AGENT_RULES`, `detect_installed_agent_targets_from_home`) | `crates/eden-skills-core/tests/agent_detect_tests.rs` (`detects_all_documented_agent_directories`) | completed |
| AGT-003 | `SPEC_AGENT_DETECT.md` 3 | Explicit `--target` MUST override auto-detection | `crates/eden-skills-cli/src/commands.rs` (`resolve_url_mode_install_targets`, URL-mode install branches) | `crates/eden-skills-cli/tests/install_agent_detect_tests.rs` (`explicit_target_override_skips_auto_detection`) | completed |
| AGT-004 | `SPEC_AGENT_DETECT.md` 2.3 | No agents detected MUST fall back to claude-code with warning | `crates/eden-skills-cli/src/commands.rs` (`resolve_url_mode_install_targets`) | `crates/eden-skills-cli/tests/install_agent_detect_tests.rs` (`install_without_target_falls_back_to_claude_with_warning`) | completed |

## 4. CLI UX Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
|---|---|---|---|---|---|
| UX-001 | `SPEC_CLI_UX.md` 4 | CLI MUST use colored output with status symbols | `crates/eden-skills-cli/src/ui.rs` (`UiContext::colors_enabled`, `UiContext::status_symbol`), `crates/eden-skills-cli/src/commands.rs` (`print_install_success`, URL-mode/local-mode install summaries) | `crates/eden-skills-cli/tests/cli_ux_tests.rs` (`tty_install_output_contains_ansi_color_and_status_symbols`) | completed |
| UX-002 | `SPEC_CLI_UX.md` 6 | CLI MUST use spinner for network operations | `crates/eden-skills-cli/src/ui.rs` (`UiContext::spinner`, `UiSpinner`), `crates/eden-skills-cli/src/commands.rs` (`install_remote_url_mode_async` clone spinner integration) | `crates/eden-skills-cli/tests/cli_ux_tests.rs` (`tty_remote_install_clone_phase_shows_spinner_and_completion_status`) | completed |
| UX-003 | `SPEC_CLI_UX.md` 4.1 | CLI MUST use `✓`/`✗`/`·`/`!` symbols for results | `crates/eden-skills-cli/src/ui.rs` (`StatusSymbol`, `UiContext::status_symbol`), `crates/eden-skills-cli/src/commands.rs` (`ensure_install_config_exists`, `print_install_success`, install summary branches) | `crates/eden-skills-cli/tests/cli_ux_tests.rs` (`tty_install_output_contains_ansi_color_and_status_symbols`, `no_color_disables_ansi_but_keeps_functional_status_output`) | completed |
| UX-004 | `SPEC_CLI_UX.md` 3.4 | CLI MUST respect `NO_COLOR`/`FORCE_COLOR`/`CI` env vars | `crates/eden-skills-cli/src/ui.rs` (`UiContext::from_env`, `colors_enabled`, `interactive_enabled`) | `crates/eden-skills-cli/tests/cli_ux_tests.rs` (`no_color_disables_ansi_but_keeps_functional_status_output`, `force_color_enables_ansi_even_on_non_tty`, `ci_env_disables_ansi_even_when_tty_is_forced`) | completed |
| UX-005 | `SPEC_CLI_UX.md` 3.3 | `--json` output MUST remain identical to Phase 1/2 | `crates/eden-skills-cli/src/commands.rs` (install JSON branches unchanged, non-JSON side outputs gated when `--json`) | `crates/eden-skills-cli/tests/cli_ux_tests.rs` (`install_json_output_keeps_contract_and_omits_visual_elements`) | completed |
| UX-006 | `SPEC_CLI_UX.md` 3.2 | Non-TTY MUST disable colors and spinners | `crates/eden-skills-cli/src/ui.rs` (`spinner_enabled`, `interactive_enabled`), `crates/eden-skills-cli/src/commands.rs` (`resolve_local_install_selection` TTY gating, remote clone spinner path) | `crates/eden-skills-cli/tests/cli_ux_tests.rs` (`non_tty_remote_install_disables_spinner_output`), `crates/eden-skills-cli/tests/install_discovery_tests.rs` (`non_tty_defaults_to_install_all_for_multi_skill_repo`) | completed |
| UX-007 | `SPEC_CLI_UX.md` 5.1 | Interactive prompts MUST use `dialoguer` | `crates/eden-skills-cli/src/commands.rs` (`prompt_install_all` via `dialoguer::Confirm`, `prompt_skill_names` via `dialoguer::Input`) | `crates/eden-skills-cli/tests/install_discovery_tests.rs` (`interactive_tty_confirm_yes_installs_all`, `interactive_tty_confirm_no_then_selects_named_skills`) | completed |

## 5. Distribution Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
|---|---|---|---|---|---|
| DST-001 | `SPEC_DISTRIBUTION.md` 2.1 | CLI MUST be installable via `cargo install eden-skills` | -- | -- | planned |
| DST-002 | `SPEC_DISTRIBUTION.md` 2.2 | GitHub Actions MUST produce release binaries | -- | -- | planned |
| DST-003 | `SPEC_DISTRIBUTION.md` 3.2 | Release archives MUST include SHA-256 checksums | -- | -- | planned |

## 6. Test Matrix Coverage

| SCENARIO_ID | Source | Scenario | Automated Test | Status |
|---|---|---|---|---|
| TM-P25-001 | `SPEC_TEST_MATRIX.md` 2 | Empty skills array validation | `crates/eden-skills-core/tests/config_tests.rs` (`load_valid_config_when_skills_array_is_missing`, `load_valid_config_when_skills_array_is_explicitly_empty`) | completed |
| TM-P25-002 | `SPEC_TEST_MATRIX.md` 2 | Empty config plan | `crates/eden-skills-cli/tests/phase25_schema_tests.rs` (`plan_with_empty_config_succeeds_and_reports_zero_actions`, `plan_json_with_empty_config_emits_empty_array`) | completed |
| TM-P25-003 | `SPEC_TEST_MATRIX.md` 2 | Empty config apply | `crates/eden-skills-cli/tests/phase25_schema_tests.rs` (`apply_with_empty_config_succeeds_with_zero_summary`) | completed |
| TM-P25-004 | `SPEC_TEST_MATRIX.md` 2 | Init template minimal | `crates/eden-skills-cli/tests/init_command.rs` (`init_creates_config_when_missing`, `init_overwrites_when_force_is_set`) | completed |
| TM-P25-005 | `SPEC_TEST_MATRIX.md` 2 | Backward compatibility | `crates/eden-skills-core/tests/config_tests.rs` (`load_phase1_style_config_with_five_skills_for_backward_compatibility`), `cargo test --workspace` | completed |
| TM-P25-006 | `SPEC_TEST_MATRIX.md` 3 | GitHub shorthand | `crates/eden-skills-core/tests/source_format_tests.rs` (`github_shorthand_expands_to_https_git_url`) | completed |
| TM-P25-007 | `SPEC_TEST_MATRIX.md` 3 | Full GitHub URL | `crates/eden-skills-core/tests/source_format_tests.rs` (`full_github_url_appends_git_suffix_when_missing`) | completed |
| TM-P25-008 | `SPEC_TEST_MATRIX.md` 3 | GitHub tree URL | `crates/eden-skills-core/tests/source_format_tests.rs` (`github_tree_url_extracts_repo_ref_and_subpath`) | completed |
| TM-P25-009 | `SPEC_TEST_MATRIX.md` 3 | Git SSH URL | `crates/eden-skills-core/tests/source_format_tests.rs` (`git_ssh_url_is_accepted_as_url_mode`) | completed |
| TM-P25-010 | `SPEC_TEST_MATRIX.md` 3 | Local path | `crates/eden-skills-cli/tests/install_url_tests.rs` (`local_path_install_persists_absolute_repo_and_skips_clone`) | completed |
| TM-P25-011 | `SPEC_TEST_MATRIX.md` 3 | Source format precedence | `crates/eden-skills-core/tests/source_format_tests.rs` (`local_path_is_detected_before_shorthand`), `crates/eden-skills-cli/tests/install_url_tests.rs` (`local_path_precedence_wins_over_shorthand_shape`) | completed |
| TM-P25-012 | `SPEC_TEST_MATRIX.md` 3 | Registry fallback | `crates/eden-skills-core/tests/source_format_tests.rs` (`unmatched_source_falls_back_to_registry_name`), `crates/eden-skills-cli/tests/install_url_tests.rs` (`registry_name_input_still_uses_registry_mode`) | completed |
| TM-P25-013 | `SPEC_TEST_MATRIX.md` 4 | Auto-derived ID | `crates/eden-skills-core/tests/source_format_tests.rs` (`auto_derived_id_uses_repo_tail_without_git_suffix`) | completed |
| TM-P25-014 | `SPEC_TEST_MATRIX.md` 4 | ID override | `crates/eden-skills-cli/tests/install_url_tests.rs` (`install_url_mode_respects_id_override`) | completed |
| TM-P25-015 | `SPEC_TEST_MATRIX.md` 4 | ID upsert | `crates/eden-skills-cli/tests/install_url_tests.rs` (`install_url_mode_upserts_existing_id_instead_of_duplicating`) | completed |
| TM-P25-016 | `SPEC_TEST_MATRIX.md` 5 | Single SKILL.md at root | `crates/eden-skills-cli/tests/install_discovery_tests.rs` (`single_root_skill_installs_without_confirmation_prompt`) | completed |
| TM-P25-017 | `SPEC_TEST_MATRIX.md` 5 | Multiple skills in `skills/` | `crates/eden-skills-cli/tests/install_discovery_tests.rs` (`skills_directory_discovery_with_all_installs_all_skills`) | completed |
| TM-P25-018 | `SPEC_TEST_MATRIX.md` 5 | Multiple skills in `packages/` | `crates/eden-skills-cli/tests/install_discovery_tests.rs` (`packages_directory_discovery_with_all_installs_all_skills`) | completed |
| TM-P25-019 | `SPEC_TEST_MATRIX.md` 5 | No SKILL.md found | `crates/eden-skills-cli/tests/install_discovery_tests.rs` (`missing_skill_markdown_installs_root_with_warning`) | completed |
| TM-P25-020 | `SPEC_TEST_MATRIX.md` 5 | List flag | `crates/eden-skills-cli/tests/install_discovery_tests.rs` (`list_flag_prints_discovered_skills_without_modifying_config`, `remote_url_list_does_not_create_config_or_install_targets`) | completed |
| TM-P25-021 | `SPEC_TEST_MATRIX.md` 6 | Install all flag | `crates/eden-skills-cli/tests/install_discovery_tests.rs` (`skills_directory_discovery_with_all_installs_all_skills`, `remote_url_with_all_installs_all_discovered_skills`) | completed |
| TM-P25-022 | `SPEC_TEST_MATRIX.md` 6 | Install specific skills | `crates/eden-skills-cli/tests/install_discovery_tests.rs` (`skill_flags_install_only_selected_skills`, `remote_url_with_skill_installs_only_selected_skill`) | completed |
| TM-P25-023 | `SPEC_TEST_MATRIX.md` 6 | Unknown skill name | `crates/eden-skills-cli/tests/install_discovery_tests.rs` (`unknown_skill_name_returns_error_with_available_names`, `single_discovered_skill_with_unmatched_skill_flag_returns_error`, `missing_skill_markdown_with_skill_flag_returns_error_instead_of_root_fallback`, `remote_url_missing_skill_markdown_with_skill_flag_returns_error`) | completed |
| TM-P25-024 | `SPEC_TEST_MATRIX.md` 6 | Interactive confirmation (TTY) | `crates/eden-skills-cli/tests/install_discovery_tests.rs` (`interactive_tty_confirm_yes_installs_all`, `interactive_tty_confirm_no_then_selects_named_skills`) | completed |
| TM-P25-025 | `SPEC_TEST_MATRIX.md` 6 | Non-TTY default | `crates/eden-skills-cli/tests/install_discovery_tests.rs` (`non_tty_defaults_to_install_all_for_multi_skill_repo`) | completed |
| TM-P25-026 | `SPEC_TEST_MATRIX.md` 7 | Multi-agent detection | `crates/eden-skills-cli/tests/install_agent_detect_tests.rs` (`install_without_target_detects_multiple_agent_directories`) | completed |
| TM-P25-027 | `SPEC_TEST_MATRIX.md` 7 | No agent fallback | `crates/eden-skills-cli/tests/install_agent_detect_tests.rs` (`install_without_target_falls_back_to_claude_with_warning`) | completed |
| TM-P25-028 | `SPEC_TEST_MATRIX.md` 7 | Target override | `crates/eden-skills-cli/tests/install_agent_detect_tests.rs` (`explicit_target_override_skips_auto_detection`) | completed |
| TM-P25-029 | `SPEC_TEST_MATRIX.md` 8 | Fresh system install | `crates/eden-skills-cli/tests/install_url_tests.rs` (`local_path_install_persists_absolute_repo_and_skips_clone`) | completed |
| TM-P25-030 | `SPEC_TEST_MATRIX.md` 8 | Missing parent directory | `crates/eden-skills-cli/tests/install_url_tests.rs` (`install_fails_when_config_parent_directory_is_missing`) | completed |
| TM-P25-031 | `SPEC_TEST_MATRIX.md` 9 | TTY color output | `crates/eden-skills-cli/tests/cli_ux_tests.rs` (`tty_install_output_contains_ansi_color_and_status_symbols`) | completed |
| TM-P25-032 | `SPEC_TEST_MATRIX.md` 9 | NO_COLOR compliance | `crates/eden-skills-cli/tests/cli_ux_tests.rs` (`no_color_disables_ansi_but_keeps_functional_status_output`) | completed |
| TM-P25-033 | `SPEC_TEST_MATRIX.md` 9 | JSON mode unchanged | `crates/eden-skills-cli/tests/cli_ux_tests.rs` (`install_json_output_keeps_contract_and_omits_visual_elements`) | completed |
| TM-P25-034 | `SPEC_TEST_MATRIX.md` 9 | Spinner during clone | `crates/eden-skills-cli/tests/cli_ux_tests.rs` (`tty_remote_install_clone_phase_shows_spinner_and_completion_status`) | completed |
| TM-P25-035 | `SPEC_TEST_MATRIX.md` 10 | Cargo install | -- | planned |
| TM-P25-036 | `SPEC_TEST_MATRIX.md` 10 | Release binary | -- | planned |
| TM-P25-037 | `SPEC_TEST_MATRIX.md` 12 | Agent-convention directory discovery | `crates/eden-skills-core/tests/discovery_tests.rs` (`discovers_skills_under_agent_convention_directories`), `crates/eden-skills-cli/tests/install_discovery_tests.rs` (`agent_convention_skill_directory_supports_skill_flag_selection`) | completed |
| TM-P25-038 | `SPEC_TEST_MATRIX.md` 12 | Marketplace manifest discovery | `crates/eden-skills-core/tests/discovery_tests.rs` (`discovers_skills_from_claude_plugin_marketplace_manifest`) | completed |
| TM-P25-039 | `SPEC_TEST_MATRIX.md` 12 | Plugin manifest discovery | `crates/eden-skills-core/tests/discovery_tests.rs` (`discovers_skills_from_claude_plugin_manifest`) | completed |
| TM-P25-040 | `SPEC_TEST_MATRIX.md` 12 | Recursive fallback discovery | `crates/eden-skills-core/tests/discovery_tests.rs` (`falls_back_to_recursive_search_when_standard_locations_have_no_skills`), `crates/eden-skills-cli/tests/install_discovery_tests.rs` (`recursive_fallback_discovery_supports_skill_flag_selection`) | completed |
