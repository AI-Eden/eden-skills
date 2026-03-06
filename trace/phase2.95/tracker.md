# Phase 2.95 Execution Tracker

Phase: Performance, Platform Reach & UX Completeness
Status: Closeout Completed
Started: 2026-03-06
Completed: 2026-03-07

## Batch Plan

| Batch | Name | WP | Requirements | Status |
| --- | --- | --- | --- | --- |
| 1 | Install Scripts | WP-5 | ISC-001~007 | completed |
| 2 | Remove All Symbol | WP-2 | RMA-001~004 | completed |
| 3 | Windows Junction Fallback | WP-3 | WJN-001~006 | completed |
| 4 | Performance Part 1: Repo-Level Cache | WP-1 pt1 | PSY-001~003, PSY-006~007 | completed |
| 5 | Performance Part 2: Batch Sync + Migration | WP-1 pt2 | PSY-004~006, PSY-008 | completed |
| 6 | Docker Bind Mount + Agent Auto-Detection | WP-4 | DBM-001~007 | completed |
| 7 | Regression + Closeout | — | TM regression | completed |

## Dependency Constraints

- Batches 1, 2, 3, 4, 6 are independent of each other.
- Batch 5 depends on Batch 4 (repo-level cache infrastructure).
- Batch 7 depends on all preceding batches.

## Completion Records

### Batch 1 — Install Scripts (Completed 2026-03-06)

- Requirements: `ISC-001`, `ISC-002`, `ISC-003`, `ISC-004`, `ISC-005`, `ISC-006`, `ISC-007`
- Completed in this pass:
  - Added root-level installers `install.sh` and `install.ps1` for GitHub Releases downloads with target detection, SHA-256 verification, install-dir support, git warning, and post-install `--version` verification.
  - Added test-only installer overrides (`EDEN_SKILLS_RELEASE_API_URL`, `EDEN_SKILLS_RELEASE_BASE_URL`, `EDEN_SKILLS_TEST_UNAME_S`, `EDEN_SKILLS_TEST_UNAME_M`) so script behavior can be validated without live network traffic.
  - Added `cargo-binstall` metadata to `crates/eden-skills-cli/Cargo.toml`.
  - Updated `README.md` and `docs/01-quickstart.md` to make the shell/PowerShell one-liners the primary install path, while preserving `cargo install` as the fallback.
  - Added `crates/eden-skills-cli/tests/install_script_tests.rs` covering `TM-P295-001` through `TM-P295-009`.
- Validation:
  - `cargo fmt --all -- --check` ✅
  - `cargo clippy --workspace -- -D warnings` ✅
  - `cargo test --workspace` ✅
  - Test inventory: `349`
- Notes:
  - Live `cargo binstall` end-to-end downloading was not executed in this environment; the release URL contract is asserted via automated manifest tests.

### Batch 2 — Remove All Symbol (Completed 2026-03-06)

- Requirements: `RMA-001`, `RMA-002`, `RMA-003`, `RMA-004`
- Completed in this pass:
  - Updated `crates/eden-skills-cli/src/commands/remove.rs` so interactive `remove` recognizes `*` as a wildcard that expands to all configured skill IDs in config order.
  - Rejected mixed wildcard selections such as `* 2` with a dedicated invalid-arguments error, without changing positional `remove <id>...` semantics.
  - Strengthened the wildcard confirmation flow with warning-style wording, preserved `-y/--yes` skip behavior, and updated the prompt hint to advertise `* for all`.
  - Added `TM-P295-010` through `TM-P295-015` coverage in `crates/eden-skills-cli/tests/remove_enhanced_tests.rs`, plus focused unit tests in `crates/eden-skills-cli/src/commands/remove.rs` for wildcard parsing and prompt helpers.
- Validation:
  - `cargo fmt --all -- --check` ✅
  - `cargo clippy --workspace -- -D warnings` ✅
  - `cargo test --workspace` ✅
  - Test inventory: `358`

### Batch 3 — Windows Junction Fallback (Completed 2026-03-06)

- Requirements: `WJN-001`, `WJN-002`, `WJN-003`, `WJN-004`, `WJN-005`, `WJN-006`
- Completed in this pass:
  - Added `junction = "1"` as a Windows-only dependency in both `crates/eden-skills-core/Cargo.toml` and `crates/eden-skills-cli/Cargo.toml`.
  - Extended `crates/eden-skills-cli/src/commands/install.rs` so the default Windows install decision now follows `symlink -> junction -> copy`, emits the new junction/hardcopy warnings, and supports test hooks for forced junction support plus probe logging.
  - Updated `crates/eden-skills-core/src/adapter.rs` and `crates/eden-skills-cli/src/commands/common.rs` so directory symlink creation falls back to NTFS junctions on Windows permission denial and existing junctions are removed safely before reinstall.
  - Updated `crates/eden-skills-core/src/plan.rs`, `crates/eden-skills-core/src/verify.rs`, and the local-source install conflict checks in `crates/eden-skills-cli/src/commands/install.rs` so junction-backed targets are treated like symlink-mode installs instead of false conflicts.
  - Added `crates/eden-skills-cli/tests/junction_tests.rs` and `crates/eden-skills-core/tests/junction_tests.rs` covering `TM-P295-016` through `TM-P295-023`, and refreshed the older hardcopy fallback regression test in `crates/eden-skills-cli/tests/install_url_tests.rs`.
- Validation:
  - `cargo fmt --all` ✅
  - `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` ✅
  - `cargo test --workspace` ✅
  - Test inventory: `363`

### Batch 4 — Performance Part 1: Repo-Level Cache (Completed 2026-03-06)

- Requirements: `PSY-001`, `PSY-002`, `PSY-003`, `PSY-006` (partial), `PSY-007`
- Completed in this pass:
  - Refactored `crates/eden-skills-core/src/source.rs` to normalize remote repo URLs, sanitize refs, derive shared repo-cache keys, group sync work by `(repo_url, ref)`, and store remote checkouts under `storage_root/.repos/{cache_key}`.
  - Added `resolve_skill_storage_root()` and `resolve_skill_source_path()` so repo-cache paths are used consistently by `crates/eden-skills-core/src/plan.rs`, `crates/eden-skills-core/src/verify.rs`, `crates/eden-skills-core/src/safety.rs`, `crates/eden-skills-cli/src/commands/reconcile.rs`, and `crates/eden-skills-cli/src/commands/update.rs`.
  - Updated `crates/eden-skills-cli/src/commands/install.rs` to preserve the remote discovery checkout, move it into the repo cache when possible, and fall back to a fresh cache clone when the move fails across filesystems.
  - Preserved local-source install semantics: local installs still stage under `storage_root/{skill_id}`, and old per-skill remote directories are left untouched instead of being deleted.
  - Added `crates/eden-skills-core/tests/perf_sync_tests.rs` and `crates/eden-skills-cli/tests/perf_sync_tests.rs` for `TM-P295-024` through `TM-P295-030`, `TM-P295-035`, and `TM-P295-038`, then updated affected existing tests to the repo-cache layout.
  - Post-merge Windows CI follow-up 1: fixed clone-count observability on Windows by adding the explicit `EDEN_SKILLS_TEST_GIT_CLONE_LOG` hook in `crates/eden-skills-core/src/source.rs` and `crates/eden-skills-cli/src/commands/install.rs`, and simplified `crates/eden-skills-cli/tests/perf_sync_tests.rs` to stop depending on PATH/git.cmd interception.
  - Post-merge Windows CI follow-up 2: fixed `crates/eden-skills-core/tests/junction_tests.rs` so the WJN-004 junction noop fixture resolves its expected source path through `resolve_skill_source_path()` instead of the pre-Batch-4 per-skill layout.
- Validation:
  - `cargo fmt --all -- --check` ✅
  - `cargo clippy --workspace -- -D warnings` ✅
  - `cargo test --workspace --all-targets` ✅
  - `cargo check --workspace --all-targets --target x86_64-pc-windows-msvc` ✅
  - Test inventory: `375`
- Notes:
  - Batch 5 followed in the next pass to finish remote-install batching, repo-level skip reporting, and the copy fast path on top of the Batch 4 repo-cache foundation.
  - The two Windows CI regressions after Batch 4 were both test-layer issues; no Batch 4 repo-cache or junction runtime contract change was required.

### Batch 5 — Performance Part 2: Batch Sync + Migration (Completed 2026-03-06)

- Requirements: `PSY-004`, `PSY-005`, `PSY-006`, `PSY-008`
- Completed in this pass:
  - Updated `crates/eden-skills-cli/src/commands/install.rs` so remote URL-mode installs batch the selected config into a single `sync_sources_async()` call before sequential install-plan execution, while preserving the existing TTY/non-TTY sync progress output contract.
  - Extended `crates/eden-skills-core/src/source.rs` with repo-level skip support for grouped sync tasks and updated `crates/eden-skills-cli/src/commands/reconcile.rs` so `apply` reports skipped repo sync tasks for unchanged lock entries while `repair` still fetches every repo.
  - Completed the remaining repo-cache migration coverage by adding `TM-P295-034` in `crates/eden-skills-cli/tests/update_ext_tests.rs` and `TM-P295-036` in `crates/eden-skills-cli/tests/doctor_copy.rs`, confirming `update` and `doctor` resolve remote sources from repo-cache-backed paths instead of legacy per-skill directories.
  - Added the `mtime + size` copy fast path in `crates/eden-skills-core/src/plan.rs`, added `TM-P295-037` in `crates/eden-skills-core/tests/plan_copy_edge_tests.rs`, and refreshed older copy-plan fixtures so unreadable-file conflicts still test the non-fast-path branch.
  - Added `TM-P295-031`, `TM-P295-032`, and `TM-P295-033` in `crates/eden-skills-cli/tests/perf_sync_tests.rs` to verify batched install fetches, apply skip summaries, and repair’s no-skip behavior.
  - Post-merge Windows CI follow-up: replaced the PATH/git wrapper fetch counter in `crates/eden-skills-cli/tests/perf_sync_tests.rs` with an explicit `EDEN_SKILLS_TEST_GIT_FETCH_LOG` hook in `crates/eden-skills-core/src/source.rs`, matching the earlier clone-count stabilization pattern from Batch 4.
- Validation:
  - `cargo fmt --all -- --check` ✅
  - `cargo clippy --workspace -- -D warnings` ✅
  - `cargo test --workspace --all-targets` ✅
  - `cargo check --workspace --all-targets --target x86_64-pc-windows-msvc` ✅
  - Test inventory: `381`
- Notes:
  - Batch 6 followed in the next pass with Docker bind mount support plus docker target auto-detection.

### Batch 6 — Docker Bind Mount + Agent Auto-Detection (Completed 2026-03-06)

- Requirements: `DBM-001`, `DBM-002`, `DBM-003`, `DBM-004`, `DBM-005`, `DBM-006`, `DBM-007`
- Completed in this pass:
  - Extended `crates/eden-skills-core/src/adapter.rs` with `docker inspect` mount parsing, host-path mapping for bind-mounted installs, container `$HOME` resolution, container agent auto-detection, and bind-mount-aware uninstall handling.
  - Updated `crates/eden-skills-cli/src/commands/install.rs` so `--target docker:<container>` now auto-detects installed agents inside the container, preserves existing manual Docker targets when install is rerun without an explicit Docker target, executes Docker-target installs through `DockerAdapter`, and prints a live-sync bind-mount hint after `docker cp` fallback.
  - Added the new `eden-skills docker mount-hint <container>` flow in `crates/eden-skills-cli/src/commands/docker_cmd.rs` and wired the command through `crates/eden-skills-cli/src/lib.rs`.
  - Updated `crates/eden-skills-cli/src/commands/diagnose.rs` to emit `DOCKER_NO_BIND_MOUNT` info findings for running Docker targets that lack writable bind mounts.
  - Refreshed `docs/04-docker-targets.md` to document auto-detection, bind-mount behavior, `docker mount-hint`, and the new doctor guidance.
  - Added `TM-P295-039` through `TM-P295-048` coverage across `crates/eden-skills-core/tests/adapter_tests.rs`, `crates/eden-skills-cli/tests/docker_bind_tests.rs`, `crates/eden-skills-cli/tests/install_agent_detect_tests.rs`, and `crates/eden-skills-cli/tests/phase2_doctor.rs`.
- Validation:
  - `cargo fmt --all -- --check` ✅
  - `cargo clippy --workspace -- -D warnings` ✅
  - `cargo test --workspace --all-targets` ✅
  - `cargo check --workspace --all-targets --target x86_64-pc-windows-msvc` ✅
  - Test inventory: `392`
- Notes:
  - Batch 7 regression + closeout followed in the next pass.

### Batch 7 — Regression + Closeout (Completed 2026-03-07)

- Closeout validation:
  - `cargo fmt --all -- --check` ✅
  - `cargo clippy --workspace -- -D warnings` ✅
  - `cargo test --workspace --all-targets` ✅ (test inventory: `393`)
  - `cargo check --workspace --all-targets --target x86_64-pc-windows-msvc` ✅
  - `rg '\x1b\[' crates/` ✅ (no hardcoded ANSI escape sequence matches)
- Contract regression checks:
  - JSON contracts remain unchanged; existing JSON contract suites continue to pass with no schema assertion updates.
  - Exit code semantics remain unchanged (`0`/`1`/`2`/`3`); `exit_code_matrix` and strict-mode command suites stay green.
- Closeout sync:
  - `spec/phase2.95/SPEC_TRACEABILITY.md` now reflects complete implementation/test mappings and closeout status.
  - `trace/phase2.95/status.yaml` updated to `closeout_completed` with the Batch 7 completion record.
  - `README.md`, `spec/README.md`, `STATUS.yaml`, `EXECUTION_TRACKER.md`, and `AGENTS.md` synced to the Phase 2.95 closeout state.
- Minor cleanup:
  - Removed cross-target warning noise in `crates/eden-skills-cli/tests/install_script_tests.rs` by gating Unix-only imports/helpers behind `cfg(not(windows))`.
  - Post-closeout follow-up: `install.sh` now appends the PATH export to the shell-selected rc file when needed, skips duplicate PATH entries, and prints reload guidance; `README.md`, `docs/01-quickstart.md`, and installer spec/tests were synced to match.
