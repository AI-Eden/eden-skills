# SPEC_TRACEABILITY.md

Requirement-to-implementation mapping for Phase 1.
Use this file to recover accurate context after compression.

## 1. Command Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
|---|---|---|---|---|---|
| CMD-PLAN-001 | `SPEC_COMMANDS.md` 3.1 | `plan` computes dry-run graph without mutation | `crates/eden-skills-core/src/plan.rs` | `crates/eden-skills-cli/tests/doctor_copy.rs` (`copy_mode_plan_detects_source_change`) | implemented |
| CMD-PLAN-002 | `SPEC_COMMANDS.md` 3.1 | `plan` output includes `skill_id/source_path/target_path/install_mode/action/reasons` | `crates/eden-skills-core/src/plan.rs` + `crates/eden-skills-cli/src/commands.rs` | `crates/eden-skills-core/tests/plan_json_contract.rs` | implemented |
| CMD-APPLY-001 | `SPEC_COMMANDS.md` 3.2 | `apply` syncs source repos before install mutations | `crates/eden-skills-core/src/source.rs` + `crates/eden-skills-cli/src/commands.rs` | `crates/eden-skills-cli/tests/apply_repair.rs` | implemented |
| CMD-APPLY-002 | `SPEC_COMMANDS.md` 3.2 | `apply` executes only create/update actions | `crates/eden-skills-cli/src/commands.rs` | `crates/eden-skills-cli/tests/apply_repair.rs` | implemented |
| CMD-DOCTOR-001 | `SPEC_COMMANDS.md` 3.3 | `doctor` reports drift/conflict and strict-mode failure, including issue code/severity/remediation | `crates/eden-skills-cli/src/commands.rs` | `crates/eden-skills-cli/tests/doctor_copy.rs` + `crates/eden-skills-cli/tests/doctor_output.rs` + `crates/eden-skills-cli/tests/doctor_json_contract.rs` | implemented |
| CMD-REPAIR-001 | `SPEC_COMMANDS.md` 3.4 | `repair` recreates/relinks recoverable targets | `crates/eden-skills-cli/src/commands.rs` | `crates/eden-skills-cli/tests/apply_repair.rs` (`repair_recovers_broken_symlink`) | implemented |
| CMD-EXIT-001 | `SPEC_COMMANDS.md` 5 | exit codes 0/1/2/3 mapped by error class | `crates/eden-skills-cli/src/lib.rs` | `crates/eden-skills-cli/tests/invalid_config_exit.rs` + `crates/eden-skills-cli/tests/exit_code_matrix.rs` | implemented |

## 2. Schema Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
|---|---|---|---|---|---|
| SCH-ROOT-001 | `SPEC_SCHEMA.md` 2 | `version` must be `1` and `skills` non-empty | `crates/eden-skills-core/src/config.rs` | `crates/eden-skills-core/tests/config_tests.rs` | implemented |
| SCH-TARGET-001 | `SPEC_SCHEMA.md` 2 | `agent=custom` requires `path` | `crates/eden-skills-core/src/config.rs` | `crates/eden-skills-core/tests/config_tests.rs` (`reject_custom_target_without_path`) | implemented |
| SCH-STRICT-001 | `SPEC_SCHEMA.md` 4 | unknown top-level keys warn by default, fail in strict mode | `crates/eden-skills-core/src/config.rs` | `crates/eden-skills-cli/tests/invalid_config_exit.rs` (`strict_mode_unknown_top_level_key_returns_exit_code_2`) | implemented |
| SCH-URL-001 | `SPEC_SCHEMA.md` 2 | source repo URL allows https/ssh/file | `crates/eden-skills-core/src/config.rs` | (covered indirectly; direct negative test pending) | partial |

## 3. Path and Target Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
|---|---|---|---|---|---|
| PATH-RESOLVE-001 | `SPEC_AGENT_PATHS.md` 3 | target path precedence: `path` -> `expected_path` -> defaults | `crates/eden-skills-core/src/paths.rs` | `crates/eden-skills-core/tests/paths_tests.rs` | implemented |
| PATH-DERIVE-001 | `SPEC_AGENT_PATHS.md` 3.1 | effective install path is `<target_root>/<skill_id>` | `crates/eden-skills-core/src/plan.rs` | `crates/eden-skills-cli/tests/apply_repair.rs` | implemented |
| PATH-NORM-001 | `SPEC_AGENT_PATHS.md` 4 | path normalization and `~` expansion | `crates/eden-skills-core/src/paths.rs` | `crates/eden-skills-core/tests/paths_tests.rs` | implemented |

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
