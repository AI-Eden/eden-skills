# Phase 2.8 Builder State

## Batch Progress

1. Batch 1 (WP-3a — commands.rs Decomposition) is complete with quality gate pass:
   - Requirements: `CST-001`, `CST-002`, `CST-003`
   - Scenarios: `TM-P28-001`, `TM-P28-002`
   - Decomposed monolithic `commands.rs` (~3768 lines) into `commands/` directory with 8 sub-modules (`mod.rs`, `install.rs`, `reconcile.rs`, `diagnose.rs`, `plan_cmd.rs`, `config_ops.rs`, `remove.rs`, `update.rs`, `common.rs`)
   - Public API unchanged via `mod.rs` re-exports; all 253 existing tests pass without modification
   - Gate: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` (253 total tests)
2. Batch 2 (WP-1 — Table Infrastructure) is complete with quality gate pass:
   - Requirements: `TBL-001`, `TBL-002`
   - Scenarios: `TM-P28-004`, `TM-P28-032`, `TM-P28-033`
   - Additions: `comfy-table = "7"` in CLI dependencies; `UiContext::table()` in `ui.rs` with TTY UTF-8 / non-TTY ASCII preset policy, non-TTY width fallback `80`, and dynamic wrapping
   - Additions: `abbreviate_home_path()` and `abbreviate_repo_url()` in `ui.rs` for semantic abbreviation of long path/repository text
   - Tests: new `table_infra_tests.rs` covering dependency declaration, table factory behavior (TTY vs non-TTY), and abbreviation helpers
   - Gate: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` (266 total tests)
