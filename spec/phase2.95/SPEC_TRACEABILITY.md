# SPEC_TRACEABILITY.md

Requirement-to-implementation mapping for Phase 2.95.
Use this file to recover accurate context after compression.

**Status:** COMPLETE — All Phase 2.95 requirements are mapped to implementation and tests; Batch 7 regression + closeout verified.

## 1. Performance Sync Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
| --- | --- | --- | --- | --- | --- |
| PSY-001 | `SPEC_PERF_SYNC.md` 2.1 | Source sync MUST use repo-level cache at `.repos/` | `crates/eden-skills-core/src/source.rs`, `crates/eden-skills-core/src/plan.rs`, `crates/eden-skills-core/src/verify.rs`, `crates/eden-skills-core/src/safety.rs` | `crates/eden-skills-core/tests/perf_sync_tests.rs` (TM-P295-024, TM-P295-025, TM-P295-026), `crates/eden-skills-cli/tests/perf_sync_tests.rs` (TM-P295-038) | completed |
| PSY-002 | `SPEC_PERF_SYNC.md` 2.2 | Cache key from normalized URL + sanitized ref | `crates/eden-skills-core/src/source.rs` | `crates/eden-skills-core/tests/perf_sync_tests.rs` (TM-P295-027, TM-P295-028) | completed |
| PSY-003 | `SPEC_PERF_SYNC.md` 3.2 | Discovery clone MUST be reused via move | `crates/eden-skills-cli/src/commands/install.rs`, `crates/eden-skills-core/src/source.rs` | `crates/eden-skills-cli/tests/perf_sync_tests.rs` (TM-P295-029, TM-P295-030) | completed |
| PSY-004 | `SPEC_PERF_SYNC.md` 4.2 | Install sync MUST batch into one reactor call | `crates/eden-skills-cli/src/commands/install.rs` | `crates/eden-skills-cli/tests/perf_sync_tests.rs` (TM-P295-031), `crates/eden-skills-cli/tests/install_discovery_tests.rs` (TM-P29-020, TM-P29-021, TM-P29-022) | completed |
| PSY-005 | `SPEC_PERF_SYNC.md` 5.2 | Apply SHOULD skip sync for unchanged repos | `crates/eden-skills-core/src/source.rs`, `crates/eden-skills-cli/src/commands/reconcile.rs` | `crates/eden-skills-cli/tests/perf_sync_tests.rs` (TM-P295-032, TM-P295-033), `crates/eden-skills-cli/tests/exit_code_matrix.rs` (`apply_reports_skipped_source_sync_on_repeated_run`) | completed |
| PSY-006 | `SPEC_PERF_SYNC.md` 7 | update/apply/repair MUST use repo cache | `crates/eden-skills-core/src/source.rs`, `crates/eden-skills-core/src/plan.rs`, `crates/eden-skills-core/src/verify.rs`, `crates/eden-skills-core/src/safety.rs`, `crates/eden-skills-cli/src/commands/reconcile.rs`, `crates/eden-skills-cli/src/commands/update.rs`, `crates/eden-skills-cli/src/commands/diagnose.rs` | `crates/eden-skills-core/tests/perf_sync_tests.rs` (`build_plan_uses_repo_cache_source_path_for_remote_skills`), `crates/eden-skills-core/tests/symlink_canonical_tests.rs`, `crates/eden-skills-cli/tests/apply_repair.rs`, `crates/eden-skills-cli/tests/lock_diff_tests.rs`, `crates/eden-skills-cli/tests/update_ext_tests.rs` (TM-P295-034), `crates/eden-skills-cli/tests/doctor_copy.rs` (TM-P295-036) | completed |
| PSY-007 | `SPEC_PERF_SYNC.md` 6.2 | Migration MUST be gradual and non-destructive | `crates/eden-skills-core/src/source.rs`, `crates/eden-skills-cli/src/commands/install.rs` | `crates/eden-skills-core/tests/perf_sync_tests.rs` (TM-P295-035), `crates/eden-skills-cli/tests/perf_sync_tests.rs` (TM-P295-038) | completed |
| PSY-008 | `SPEC_PERF_SYNC.md` 8 | Copy-mode mtime+size fast path | `crates/eden-skills-core/src/plan.rs` | `crates/eden-skills-core/tests/plan_copy_edge_tests.rs` (TM-P295-037) | completed |

## 2. Remove All Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
| --- | --- | --- | --- | --- | --- |
| RMA-001 | `SPEC_REMOVE_ALL.md` 2.1 | `*` wildcard returns all skill IDs | `crates/eden-skills-cli/src/commands/remove.rs` | `crates/eden-skills-cli/src/commands/remove.rs` (`parse_remove_selection_returns_all_skill_ids_in_config_order_for_wildcard`), `crates/eden-skills-cli/tests/remove_enhanced_tests.rs` (TM-P295-010) | completed |
| RMA-002 | `SPEC_REMOVE_ALL.md` 2.2 | `*` combined with other tokens MUST error | `crates/eden-skills-cli/src/commands/remove.rs` | `crates/eden-skills-cli/src/commands/remove.rs` (`parse_remove_selection_rejects_wildcard_mixed_with_other_tokens`), `crates/eden-skills-cli/tests/remove_enhanced_tests.rs` (TM-P295-011) | completed |
| RMA-003 | `SPEC_REMOVE_ALL.md` 3 | Wildcard triggers strengthened confirmation | `crates/eden-skills-cli/src/commands/remove.rs` | `crates/eden-skills-cli/src/commands/remove.rs` (`remove_confirmation_prompt_uses_warning_prefix_for_wildcard`), `crates/eden-skills-cli/tests/remove_enhanced_tests.rs` (TM-P295-012, TM-P295-014, TM-P295-015) | completed |
| RMA-004 | `SPEC_REMOVE_ALL.md` 2.3 | Prompt includes `* for all` hint | `crates/eden-skills-cli/src/commands/remove.rs` | `crates/eden-skills-cli/src/commands/remove.rs` (`remove_selection_prompt_mentions_wildcard_hint`), `crates/eden-skills-cli/tests/remove_enhanced_tests.rs` (TM-P295-013) | completed |

## 3. Windows Junction Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
| --- | --- | --- | --- | --- | --- |
| WJN-001 | `SPEC_WINDOWS_JUNCTION.md` 2 | Three-level fallback: symlink → junction → copy | `crates/eden-skills-cli/src/commands/install.rs`, `crates/eden-skills-core/src/adapter.rs`, `crates/eden-skills-cli/src/commands/common.rs` | `crates/eden-skills-cli/tests/junction_tests.rs` (TM-P295-016, TM-P295-017, TM-P295-018), `crates/eden-skills-cli/tests/install_url_tests.rs` (`install_warns_when_windows_symlink_and_junction_are_unavailable_and_falls_back_to_hardcopy`) | completed |
| WJN-002 | `SPEC_WINDOWS_JUNCTION.md` 5 | `junction` crate as `cfg(windows)` dependency | `crates/eden-skills-core/Cargo.toml`, `crates/eden-skills-cli/Cargo.toml` | `crates/eden-skills-core/tests/junction_tests.rs` (TM-P295-023) | completed |
| WJN-003 | `SPEC_WINDOWS_JUNCTION.md` 3.1 | Junction NOT exposed as new InstallMode | `crates/eden-skills-cli/src/commands/install.rs` | `crates/eden-skills-cli/tests/junction_tests.rs` (TM-P295-019) | completed |
| WJN-004 | `SPEC_WINDOWS_JUNCTION.md` 4 | plan.rs detects junction reparse points | `crates/eden-skills-core/src/plan.rs`, `crates/eden-skills-core/src/verify.rs` | `crates/eden-skills-core/tests/junction_tests.rs` (TM-P295-020) | completed |
| WJN-005 | `SPEC_WINDOWS_JUNCTION.md` 3.2–3.3 | Adapter handles junction create/remove | `crates/eden-skills-core/src/adapter.rs`, `crates/eden-skills-cli/src/commands/common.rs`, `crates/eden-skills-cli/src/commands/install.rs` | `crates/eden-skills-core/tests/junction_tests.rs` (TM-P295-021) | completed |
| WJN-006 | `SPEC_WINDOWS_JUNCTION.md` 2.1 | Junction probe in install mode decision | `crates/eden-skills-cli/src/commands/install.rs` | `crates/eden-skills-cli/tests/junction_tests.rs` (TM-P295-022) | completed |

## 4. Docker Bind Mount Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
| --- | --- | --- | --- | --- | --- |
| DBM-001 | `SPEC_DOCKER_BIND.md` 3.1–3.2 | Bind mount detection via docker inspect | `crates/eden-skills-core/src/adapter.rs` | `crates/eden-skills-core/tests/adapter_tests.rs` (TM-P295-039, TM-P295-040) | completed |
| DBM-002 | `SPEC_DOCKER_BIND.md` 3.3–3.4 | Bind mount → host-side symlink / host-side uninstall | `crates/eden-skills-core/src/adapter.rs`, `crates/eden-skills-cli/src/commands/install.rs` | `crates/eden-skills-core/tests/adapter_tests.rs` (TM-P295-039, TM-P295-045) | completed |
| DBM-003 | `SPEC_DOCKER_BIND.md` 4 | `docker mount-hint` subcommand | `crates/eden-skills-cli/src/commands/docker_cmd.rs`, `crates/eden-skills-cli/src/lib.rs` | `crates/eden-skills-cli/tests/docker_bind_tests.rs` (TM-P295-041, TM-P295-042) | completed |
| DBM-004 | `SPEC_DOCKER_BIND.md` 5 | Doctor reports DOCKER_NO_BIND_MOUNT | `crates/eden-skills-cli/src/commands/diagnose.rs` | `crates/eden-skills-cli/tests/phase2_doctor.rs` (TM-P295-043) | completed |
| DBM-005 | `SPEC_DOCKER_BIND.md` 6 | Install completion bind-mount hint | `crates/eden-skills-cli/src/commands/install.rs` | `crates/eden-skills-cli/tests/docker_bind_tests.rs` (TM-P295-044) | completed |
| DBM-006 | `SPEC_DOCKER_BIND.md` 4, 8 | docs/04-docker-targets.md updated | `docs/04-docker-targets.md` | — | completed |
| DBM-007 | `SPEC_DOCKER_BIND.md` 2 | `--target docker:` auto-detects agents in container | `crates/eden-skills-core/src/adapter.rs`, `crates/eden-skills-cli/src/commands/install.rs` | `crates/eden-skills-cli/tests/install_agent_detect_tests.rs` (TM-P295-046, TM-P295-047, TM-P295-048) | completed |

## 5. Install Script Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
| --- | --- | --- | --- | --- | --- |
| ISC-001 | `SPEC_INSTALL_SCRIPT.md` 2.1 | install.sh for Linux/macOS | `install.sh`, `README.md`, `docs/01-quickstart.md` | `crates/eden-skills-cli/tests/install_script_tests.rs` (TM-P295-001, TM-P295-002) | completed |
| ISC-002 | `SPEC_INSTALL_SCRIPT.md` 2.2 | install.ps1 for Windows | `install.ps1`, `README.md`, `docs/01-quickstart.md` | `crates/eden-skills-cli/tests/install_script_tests.rs` (TM-P295-006) | completed |
| ISC-003 | `SPEC_INSTALL_SCRIPT.md` 2.1 | Platform detection and triple mapping | `install.sh`, `install.ps1` | `crates/eden-skills-cli/tests/install_script_tests.rs` (TM-P295-001, TM-P295-002, TM-P295-003, TM-P295-006) | completed |
| ISC-004 | `SPEC_INSTALL_SCRIPT.md` 2.1 | SHA-256 integrity verification | `install.sh`, `install.ps1` | `crates/eden-skills-cli/tests/install_script_tests.rs` (TM-P295-004, TM-P295-007) | completed |
| ISC-005 | `SPEC_INSTALL_SCRIPT.md` 2.1 | PATH configuration and reload guidance | `install.sh`, `install.ps1` | `crates/eden-skills-cli/tests/install_script_tests.rs` (TM-P295-005, TM-P295-006), `crates/eden-skills-cli/tests/install_script_tests.rs` (`install_sh_does_not_duplicate_existing_path_entry_in_shell_rc`) | completed |
| ISC-006 | `SPEC_INSTALL_SCRIPT.md` 3 | cargo-binstall metadata | `crates/eden-skills-cli/Cargo.toml` | `crates/eden-skills-cli/tests/install_script_tests.rs` (TM-P295-009) | completed |
| ISC-007 | `SPEC_INSTALL_SCRIPT.md` 2.1 | EDEN_SKILLS_VERSION version pinning | `install.sh`, `install.ps1` | `crates/eden-skills-cli/tests/install_script_tests.rs` (TM-P295-006, TM-P295-008) | completed |
