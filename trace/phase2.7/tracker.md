# Phase 2.7 Builder State

Archived from EXECUTION_TRACKER.md at Phase 2.8 archive migration.

## Batch Progress

1. Batch 1 (WP-1 part 1 — Lock File Core) is complete with quality gate pass:
   - Requirements: `LCK-002`, `LCK-003`, `LCK-004`, `LCK-005`, `LCK-006`, `LCK-009`
   - Scenarios: `TM-P27-001`, `TM-P27-002`, `TM-P27-003`, `TM-P27-006`, `TM-P27-007`, `TM-P27-008`, `TM-P27-009`, `TM-P27-012`
   - New module: `eden-skills-core/src/lock.rs` (LockFile/LockSkillEntry/LockTarget types, lock path derivation, read/write with missing/corrupted fallback, sorted serialization)
   - CLI integration: lock file written after init, apply, repair, install, and remove commands
   - Tests: 14 core + 9 CLI integration = 23 new tests
   - Gate: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` (212 total tests)
2. Batch 2 (WP-1 part 2 — Diff-Driven Reconciliation) is complete with quality gate pass:
   - Requirements: `LCK-001`, `LCK-007`, `LCK-008`, `LCK-010`
   - Scenarios: `TM-P27-004`, `TM-P27-005`, `TM-P27-010`, `TM-P27-011`, `TM-P27-015`
   - Additions: `Action::Remove` variant, lock diff algorithm (`compute_lock_diff`), orphan removal (`uninstall_orphaned_lock_entries`), noop optimization (`filter_config_for_sync`), resolved_commit capture (`collect_resolved_commits`)
   - Fixed pre-existing Windows bug: `looks_like_local_path` now handles Windows absolute paths via `Path::is_absolute()`
   - Tests: 8 core diff + 6 CLI integration = 14 new tests
   - Gate: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` (226 total tests)
3. Batch 3 (WP-2 — Help System) is complete with quality gate pass:
   - Requirements: `HLP-001`, `HLP-002`, `HLP-003`, `HLP-004`, `HLP-005`, `HLP-006`, `HLP-007`
   - Scenarios: `TM-P27-016`, `TM-P27-017`, `TM-P27-018`, `TM-P27-019`, `TM-P27-020`, `TM-P27-021`
   - Additions: `#[command(version)]` and `-V` for root CLI; `about`/`long_about`/`after_help` for root; `next_help_heading` and `about` for all subcommands; `help` annotations for all arguments; short flags `-s`/`-t`/`-y`/`-V`; `install --copy` with `InstallRequest.copy` and `yes` wiring
   - Tests: `help_system_tests.rs` (6 new tests covering version, root help, subcommand about, argument help, short flags, install --copy)
   - Gate: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` (232 total tests)
4. WP-1 tail completion (`TM-P27-013`, `TM-P27-014`) is complete with quality gate pass:
   - Scenarios: `TM-P27-013` (repair updates lock), `TM-P27-014` (apply remove with docker target in lock)
   - Additions: `lock_lifecycle_tests::repair_updates_lock_file_after_fixing_broken_symlink`, `lock_diff_tests::apply_removes_orphaned_docker_target_from_lock`
   - Implementation updates: `LockTarget.environment` added to lock schema (`local` default, omitted when local), lock diff target comparison now includes environment, and orphan remove path now uses lock target environment for adapter selection
   - Gate: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` (234 total tests)
5. Batch 4 (WP-3 — Output Polish) is complete with quality gate pass:
   - Requirements: `OUT-001`, `OUT-002`, `OUT-003`, `OUT-004`, `OUT-005`, `OUT-006`, `OUT-007`, `OUT-008`
   - Scenarios: `TM-P27-022` through `TM-P27-031`
   - Additions: `output_polish_tests.rs` (10 integration tests + 1 Windows-gated test), `--color` root flag (`auto|always|never`), `owo-colors` migration, `console` direct dependency removal, contextual `config file not found` and `skill not found` hints, preflight checks for missing `git`/`docker` executables
   - Gate: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` (244 total tests)
   - Post-completion hotfix: isolated `lock_lifecycle_tests::install_creates_lock_file` from user HOME writes and added `init_command::init_does_not_create_storage_root_directory`; targeted regression suites passed, current workspace test inventory is `245`.
6. Batch 5 (WP-4 — Remove Enhancements) is complete with quality gate pass:
   - Requirements: `RMV-001`, `RMV-002`, `RMV-003`, `RMV-004`, `RMV-005`
   - Scenarios: `TM-P27-032` through `TM-P27-039`
   - Additions: remove command now accepts multiple positional IDs, validates unknown IDs atomically before mutation, supports no-arg interactive selection on TTY, fails no-arg non-TTY with usage hint, and supports confirmation skipping via `-y` for remove/install paths
   - Output contract: remove JSON payload now includes `removed` array for batch output
   - Tests: `remove_enhanced_tests.rs` (8 integration tests; TM-P27-032~039)
   - Gate: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` (253 total tests)
7. Batch 6 (Regression + Closeout) is complete with quality gate pass:
   - Scenario: `TM-P27-040` (full Phase 1/2/2.5/2.7 regression gate)
   - Gate: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` (all passed)
   - Closeout sync: updated `spec/phase2.7/SPEC_TRACEABILITY.md` (`TM-P27-040` marked done), `STATUS.yaml` (Phase 2.7 closeout completed), and EXECUTION_TRACKER.md
8. Post-closeout agent-detection hardening is complete:
   - Scope: detect parent-only global roots (for example `~/.config/opencode/`) without creating directories during detection
   - Behavior: `install` now auto-detects these agents and creates missing `skills/` target directories during install/apply
   - Reinstall contract: repeated `install` for an existing skill backfills newly detected agent targets (no extra flag, no `repair` required)
   - Coverage: added core + CLI regression tests for parent-only `opencode` detection and install fan-out
   - Spec sync: updated `SPEC_AGENT_DETECT`, `SPEC_TEST_MATRIX`, and `SPEC_TRACEABILITY` for parent-root fallback contract
