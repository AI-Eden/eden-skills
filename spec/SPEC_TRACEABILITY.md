# SPEC_TRACEABILITY.md

Requirement-to-implementation mapping for Phase 1.
Use this file to recover accurate context after compression.

## 1. Command Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
|---|---|---|---|---|---|
| CMD-PLAN-001 | `SPEC_COMMANDS.md` 3.1 | `plan` computes dry-run graph without mutation | `crates/eden-skills-core/src/plan.rs` | `crates/eden-skills-cli/tests/doctor_copy.rs` (`copy_mode_plan_detects_source_change`) | implemented |
| CMD-PLAN-002 | `SPEC_COMMANDS.md` 3.1 | `plan` output includes `skill_id/source_path/target_path/install_mode/action/reasons` | `crates/eden-skills-core/src/plan.rs` + `crates/eden-skills-cli/src/commands.rs` | `crates/eden-skills-core/tests/plan_json_contract.rs` + `crates/eden-skills-cli/tests/plan_json_contract.rs` | implemented |
| CMD-PLAN-003 | `SPEC_COMMANDS.md` 3.1.2 | copy-mode plan handles symlinks/IO errors deterministically (conflict + stable reason) | `crates/eden-skills-core/src/plan.rs` | `crates/eden-skills-core/tests/plan_copy_edge_tests.rs` | implemented |
| CMD-APPLY-001 | `SPEC_COMMANDS.md` 3.2 | `apply` syncs source repos before install mutations | `crates/eden-skills-core/src/source.rs` + `crates/eden-skills-cli/src/commands.rs` | `crates/eden-skills-cli/tests/apply_repair.rs` | implemented |
| CMD-SYNC-001 | `SPEC_COMMANDS.md` 3.2.1 | source sync reports `cloned/updated/skipped/failed` and stage diagnostics (`clone/fetch/checkout`) | `crates/eden-skills-core/src/source.rs` + `crates/eden-skills-cli/src/commands.rs` | `crates/eden-skills-core/tests/source_sync_tests.rs` + `crates/eden-skills-cli/tests/exit_code_matrix.rs` | implemented |
| CMD-SYNC-002 | `SPEC_COMMANDS.md` 3.2.1 | source sync failure aggregation is config-ordered; source sync failure exit (`1`) takes precedence over strict conflict exit (`3`) | `crates/eden-skills-core/src/source.rs` + `crates/eden-skills-cli/src/commands.rs` | `crates/eden-skills-core/tests/source_sync_tests.rs` (`sync_sources_continues_after_failure_and_aggregates_results`) + `crates/eden-skills-cli/tests/exit_code_matrix.rs` (`apply_aggregates_multiskill_source_failures_in_config_order`, `apply_strict_source_sync_failure_takes_precedence_over_conflict_exit_code`, `repair_strict_source_sync_failure_takes_precedence_over_conflict_exit_code`) | implemented |
| CMD-APPLY-002 | `SPEC_COMMANDS.md` 3.2 | `apply` executes only create/update actions | `crates/eden-skills-cli/src/commands.rs` | `crates/eden-skills-cli/tests/apply_repair.rs` | implemented |
| CMD-APPLY-003 | `SPEC_COMMANDS.md` 3.5.1 + 3.5.3 | `apply` persists safety metadata and skips target mutation when `no_exec_metadata_only=true` | `crates/eden-skills-core/src/safety.rs` + `crates/eden-skills-cli/src/commands.rs` | `crates/eden-skills-cli/tests/safety_gate.rs` (`apply_no_exec_metadata_only_skips_target_mutation_and_writes_metadata`, `apply_sync_failure_still_writes_safety_metadata`) | implemented |
| CMD-DOCTOR-001 | `SPEC_COMMANDS.md` 3.3 | `doctor` reports drift/conflict and strict-mode failure, including issue code/severity/remediation | `crates/eden-skills-cli/src/commands.rs` | `crates/eden-skills-cli/tests/doctor_copy.rs` + `crates/eden-skills-cli/tests/doctor_output.rs` + `crates/eden-skills-cli/tests/doctor_json_contract.rs` | implemented |
| CMD-DOCTOR-002 | `SPEC_COMMANDS.md` 3.5.4 | `doctor` emits safety findings (`LICENSE_*`, `RISK_REVIEW_REQUIRED`, `NO_EXEC_METADATA_ONLY`) | `crates/eden-skills-core/src/safety.rs` + `crates/eden-skills-cli/src/commands.rs` | `crates/eden-skills-cli/tests/safety_gate.rs` (`doctor_reports_safety_findings`) | implemented |
| CMD-DOCTOR-003 | `SPEC_COMMANDS.md` 3.3.1 | `doctor --strict` preserves finding payload parity with non-strict mode (exit semantics differ only) | `crates/eden-skills-cli/src/commands.rs` + `crates/eden-skills-cli/src/main.rs` | `crates/eden-skills-cli/tests/doctor_strict_parity.rs` | implemented |
| CMD-REPAIR-001 | `SPEC_COMMANDS.md` 3.4 | `repair` recreates/relinks recoverable targets | `crates/eden-skills-cli/src/commands.rs` | `crates/eden-skills-cli/tests/apply_repair.rs` (`repair_recovers_broken_symlink`) | implemented |
| CMD-REPAIR-002 | `SPEC_COMMANDS.md` 3.5.1 + 3.5.3 | `repair` persists safety metadata and skips target mutation when `no_exec_metadata_only=true` | `crates/eden-skills-core/src/safety.rs` + `crates/eden-skills-cli/src/commands.rs` | `crates/eden-skills-cli/tests/safety_gate.rs` (`apply_no_exec_metadata_only_skips_target_mutation_and_writes_metadata`) | implemented |
| CMD-SAFETY-001 | `SPEC_COMMANDS.md` 3.5.2 | safety detection classifies license and risk labels with deterministic outputs | `crates/eden-skills-core/src/safety.rs` | `crates/eden-skills-core/tests/safety_tests.rs` | implemented |
| CMD-INIT-001 | `SPEC_COMMANDS.md` 4.1 | `init` creates default config when absent; fails unless `--force` when file exists | `crates/eden-skills-cli/src/commands.rs` + `crates/eden-skills-cli/src/lib.rs` | `crates/eden-skills-cli/tests/init_command.rs` | implemented |
| CMD-ADD-001 | `SPEC_COMMANDS.md` 4.2 | `add` appends a new skill entry and validates config before write | `crates/eden-skills-cli/src/commands.rs` + `crates/eden-skills-cli/src/lib.rs` | `crates/eden-skills-cli/tests/config_lifecycle.rs` | implemented |
| CMD-REMOVE-001 | `SPEC_COMMANDS.md` 4.3 | `remove <skill_id>` removes only the matching skill and errors on missing id | `crates/eden-skills-cli/src/commands.rs` + `crates/eden-skills-cli/src/lib.rs` | `crates/eden-skills-cli/tests/config_lifecycle.rs` | implemented |
| CMD-SET-001 | `SPEC_COMMANDS.md` 4.4 | `set <skill_id> ...` mutates only requested fields; validates config before write | `crates/eden-skills-cli/src/commands.rs` + `crates/eden-skills-cli/src/lib.rs` | `crates/eden-skills-cli/tests/config_lifecycle.rs` | implemented |
| CMD-LIST-001 | `SPEC_COMMANDS.md` 4.5 | `list` displays skill inventory and key metadata (text + JSON) | `crates/eden-skills-cli/src/commands.rs` + `crates/eden-skills-cli/src/lib.rs` | `crates/eden-skills-cli/tests/list_command.rs` + `crates/eden-skills-cli/tests/list_json_contract.rs` | implemented |
| CMD-CONFIG-EXPORT-001 | `SPEC_COMMANDS.md` 4.6 | `config export` emits full normalized TOML config | `crates/eden-skills-cli/src/commands.rs` + `crates/eden-skills-cli/src/lib.rs` | `crates/eden-skills-cli/tests/config_export.rs` | implemented |
| CMD-CONFIG-IMPORT-001 | `SPEC_COMMANDS.md` 4.7 | `config import` validates and imports config (dry-run supported) | `crates/eden-skills-cli/src/commands.rs` + `crates/eden-skills-cli/src/lib.rs` | `crates/eden-skills-cli/tests/config_import.rs` | implemented |
| CMD-EXIT-001 | `SPEC_COMMANDS.md` 5 | exit codes 0/1/2/3 mapped by error class | `crates/eden-skills-cli/src/lib.rs` | `crates/eden-skills-cli/tests/invalid_config_exit.rs` + `crates/eden-skills-cli/tests/exit_code_matrix.rs` | implemented |

## 2. Schema Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
|---|---|---|---|---|---|
| SCH-ROOT-001 | `SPEC_SCHEMA.md` 2 | `version` must be `1` and `skills` non-empty | `crates/eden-skills-core/src/config.rs` | `crates/eden-skills-core/tests/config_tests.rs` | implemented |
| SCH-TARGET-001 | `SPEC_SCHEMA.md` 2 | `agent=custom` requires `path` | `crates/eden-skills-core/src/config.rs` | `crates/eden-skills-core/tests/config_tests.rs` (`reject_custom_target_without_path`) | implemented |
| SCH-STRICT-001 | `SPEC_SCHEMA.md` 4 | unknown top-level keys warn by default, fail in strict mode | `crates/eden-skills-core/src/config.rs` | `crates/eden-skills-cli/tests/invalid_config_exit.rs` (`strict_mode_unknown_top_level_key_returns_exit_code_2`) | implemented |
| SCH-URL-001 | `SPEC_SCHEMA.md` 2 | source repo URL allows https/ssh/file | `crates/eden-skills-core/src/config.rs` | `crates/eden-skills-core/tests/config_tests.rs` (`repo_url_allows_https_ssh_scp_like_and_file`, `repo_url_rejects_non_git_schemes`) | implemented |
| SCH-SAFETY-001 | `SPEC_SCHEMA.md` 3 | `safety.no_exec_metadata_only` defaults to false and controls no-exec behavior | `crates/eden-skills-core/src/config.rs` + `crates/eden-skills-cli/src/commands.rs` | `crates/eden-skills-cli/tests/safety_gate.rs` | implemented |

## 3. Path and Target Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
|---|---|---|---|---|---|
| PATH-RESOLVE-001 | `SPEC_AGENT_PATHS.md` 3 | target path precedence: `path` -> `expected_path` -> defaults | `crates/eden-skills-core/src/paths.rs` | `crates/eden-skills-core/tests/paths_tests.rs` | implemented |
| PATH-DERIVE-001 | `SPEC_AGENT_PATHS.md` 3.1 | effective install path is `<target_root>/<skill_id>` | `crates/eden-skills-core/src/plan.rs` | `crates/eden-skills-cli/tests/apply_repair.rs` | implemented |
| PATH-NORM-001 | `SPEC_AGENT_PATHS.md` 4 | path normalization, `~` expansion, and canonical path comparisons for symlink verification | `crates/eden-skills-core/src/paths.rs` + `crates/eden-skills-core/src/verify.rs` | `crates/eden-skills-core/tests/paths_tests.rs` + `crates/eden-skills-core/tests/symlink_canonical_tests.rs` | implemented |

## 4. Test Matrix Coverage

| SCENARIO_ID | Source | Scenario | Automated Test | Status |
|---|---|---|---|---|
| TM-001 | `SPEC_TEST_MATRIX.md` 2.1 | Fresh install | `crates/eden-skills-cli/tests/apply_repair.rs` (`fresh_and_repeated_apply_symlink`) | covered |
| TM-002 | `SPEC_TEST_MATRIX.md` 2.2 | Repeated apply | `crates/eden-skills-cli/tests/apply_repair.rs` (`fresh_and_repeated_apply_symlink`) | covered |
| TM-003 | `SPEC_TEST_MATRIX.md` 2.3 | Broken symlink recovery | `crates/eden-skills-cli/tests/apply_repair.rs` (`repair_recovers_broken_symlink`) | covered |
| TM-004 | `SPEC_TEST_MATRIX.md` 2.4 | Source moved or missing | `crates/eden-skills-cli/tests/doctor_copy.rs` (`doctor_strict_detects_missing_source`) | covered |
| TM-005 | `SPEC_TEST_MATRIX.md` 2.5 | Copy mode verification | `crates/eden-skills-cli/tests/doctor_copy.rs` (`copy_mode_plan_detects_source_change`) | covered |
| TM-006 | `SPEC_TEST_MATRIX.md` 2.6 | Invalid config validation errors | `crates/eden-skills-cli/tests/invalid_config_exit.rs` (`invalid_config_returns_exit_code_2_and_field_path`) | covered |
| TM-007 | `SPEC_TEST_MATRIX.md` 2.7 | Permission denied target path | `crates/eden-skills-cli/tests/apply_repair.rs` (`apply_fails_on_permission_denied_target_path`) | covered |

## 5. Incremental Source Sync Coverage

| SCENARIO_ID | Source | Scenario | Automated Test | Status |
|---|---|---|---|---|
| TM-SYNC-001 | `SPEC_TEST_MATRIX.md` 7 | repeated `apply` reports source sync `skipped > 0` and `failed = 0` when upstream unchanged | `crates/eden-skills-cli/tests/exit_code_matrix.rs` (`apply_reports_skipped_source_sync_on_repeated_run`) | covered |
| TM-SYNC-002 | `SPEC_TEST_MATRIX.md` 7 | source sync reports `updated > 0` after upstream commit advance | `crates/eden-skills-core/tests/source_sync_tests.rs` (`sync_sources_tracks_cloned_skipped_and_updated_counts`) | covered |
| TM-SYNC-003 | `SPEC_TEST_MATRIX.md` 7 | clone failure emits actionable diagnostics with `stage=clone` | `crates/eden-skills-cli/tests/exit_code_matrix.rs` (`apply_returns_exit_code_1_on_runtime_git_failure`) + `crates/eden-skills-core/tests/source_sync_tests.rs` (`sync_sources_reports_clone_failure_stage`) | covered |
| TM-SYNC-004 | `SPEC_TEST_MATRIX.md` 7 | fetch failure emits actionable diagnostics with `stage=fetch` | `crates/eden-skills-cli/tests/exit_code_matrix.rs` (`apply_returns_exit_code_1_with_fetch_failure_diagnostics`) + `crates/eden-skills-core/tests/source_sync_tests.rs` (`sync_sources_reports_fetch_failure_stage`) | covered |
| TM-SYNC-005 | `SPEC_TEST_MATRIX.md` 7 | checkout failure emits actionable diagnostics with `stage=checkout` | `crates/eden-skills-cli/tests/exit_code_matrix.rs` (`apply_returns_exit_code_1_with_checkout_failure_diagnostics`) + `crates/eden-skills-core/tests/source_sync_tests.rs` (`sync_sources_reports_checkout_failure_stage`) | covered |
| TM-SYNC-006 | `SPEC_TEST_MATRIX.md` 7 | multi-skill source sync diagnostics preserve config order | `crates/eden-skills-cli/tests/exit_code_matrix.rs` (`apply_aggregates_multiskill_source_failures_in_config_order`) | covered |
| TM-SYNC-007 | `SPEC_TEST_MATRIX.md` 7 | multi-skill source sync reports mixed success/failure counters without fail-fast | `crates/eden-skills-core/tests/source_sync_tests.rs` (`sync_sources_continues_after_failure_and_aggregates_results`) | covered |

## 6. Incremental Strict-Mode Interaction Coverage

| SCENARIO_ID | Source | Scenario | Automated Test | Status |
|---|---|---|---|---|
| TM-STRICT-001 | `SPEC_TEST_MATRIX.md` 8 | `apply --strict` returns exit code `1` when source sync failures exist, even with conflict candidates | `crates/eden-skills-cli/tests/exit_code_matrix.rs` (`apply_strict_source_sync_failure_takes_precedence_over_conflict_exit_code`) | covered |
| TM-STRICT-002 | `SPEC_TEST_MATRIX.md` 8 | `repair --strict` returns exit code `1` when source sync failures exist, even with conflict candidates | `crates/eden-skills-cli/tests/exit_code_matrix.rs` (`repair_strict_source_sync_failure_takes_precedence_over_conflict_exit_code`) | covered |
