# Phase 2.95 Execution Tracker

Phase: Performance, Platform Reach & UX Completeness
Status: Batch 2 Completed
Started: 2026-03-06

## Batch Plan

| Batch | Name | WP | Requirements | Status |
| --- | --- | --- | --- | --- |
| 1 | Install Scripts | WP-5 | ISC-001~007 | completed |
| 2 | Remove All Symbol | WP-2 | RMA-001~004 | completed |
| 3 | Windows Junction Fallback | WP-3 | WJN-001~006 | pending |
| 4 | Performance Part 1: Repo-Level Cache | WP-1 pt1 | PSY-001~003, PSY-006~007 | pending |
| 5 | Performance Part 2: Batch Sync + Migration | WP-1 pt2 | PSY-004~006, PSY-008 | pending |
| 6 | Docker Bind Mount + Agent Auto-Detection | WP-4 | DBM-001~007 | pending |
| 7 | Regression + Closeout | — | TM regression | pending |

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
