# Phase 2.97 Execution Tracker

Phase: Reliability, Interactive UX & Docker Safety
Status: In Progress — Batch 1 Completed
Started: 2026-03-07
Completed: —

## Batch Plan

| Batch | Name | WP | Requirements | Status |
| --- | --- | --- | --- | --- |
| 1 | Update Concurrency Fix | WP-1 | UFX-001~003 | completed |
| 2 | Table Content Styling + Help Colorization + List Table + Hint Sync | WP-2 + WP-6 | TST-001~008, HSY-001~002 | pending |
| 3 | Interactive UX (Remove + Install) | WP-3 | IUX-001~010 | pending |
| 4 | Cache Clean | WP-4 | CCL-001~007 | pending |
| 5 | Docker Management Domain | WP-5 | DMG-001~008 | pending |
| 6 | Documentation + Regression + Closeout | WP-7 | DOC-001~002, TM regression | pending |

## Dependency Constraints

- Batches 1, 2, 3, 4, 5 are independent of each other.
- Batch 6 (documentation + regression) depends on all preceding batches.

## Completion Records

### Batch 1 — Update Concurrency Fix (Completed 2026-03-07)

- Requirements: `UFX-001`, `UFX-002`, `UFX-003`
- Completed in this pass:
  - Refactored `crates/eden-skills-cli/src/commands/update.rs` so Mode A refresh groups remote skills by `repo_cache_key()` and performs exactly one fetch per shared repo-cache checkout.
  - Preserved per-skill output by broadcasting the grouped fetch outcome back into individual `SkillRefreshResult` rows for both human and JSON update output.
  - Kept local-source skills out of repo-cache grouping, so locally staged copies still refresh independently instead of sharing a remote dedup key.
  - Added stale `.git/index.lock` and `.git/shallow.lock` cleanup before fetch, with warning output when aged locks are removed.
  - Added `crates/eden-skills-cli/tests/update_fix_tests.rs` covering `TM-P297-001` through `TM-P297-006`.
- Validation:
  - `cargo fmt --all -- --check` ✅
  - `cargo clippy --workspace -- -D warnings` ✅
  - `cargo test --workspace` ✅
  - Test inventory: `399`
- Notes:
  - The Batch 1 tests use the existing `EDEN_SKILLS_TEST_GIT_FETCH_LOG` pattern to count refresh fetches, ensuring the shared-repo race is asserted deterministically instead of relying on flaky timing.
