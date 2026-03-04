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
3. Batch 3 (WP-2 part 1 — Category A-1 Core State Commands) is complete with quality gate pass:
   - Requirements: `OUP-001`, `OUP-002`, `OUP-004`, `OUP-013`, `OUP-014`, `OUP-015`, `OUP-016`, `OUP-017`, `OUP-019`
   - Scenarios: `TM-P28-012`, `TM-P28-013`, `TM-P28-014`, `TM-P28-015`, `TM-P28-016`, `TM-P28-017`, `TM-P28-021`, `TM-P28-022`, `TM-P28-026`, `TM-P28-027`, `TM-P28-028`
   - Additions: new `output_upgrade_a1_tests.rs` with TM-aligned assertions for apply/repair styled output, plan text format, arrow hint/path abbreviation, and warning formatting
   - Changes: `reconcile.rs` now emits UiContext-based `Syncing`/`Safety`/`Summary` sections, per-target `Install`/`Remove` lines, and `✓ Verification passed` for both `apply` and `repair`
   - Changes: `plan_cmd.rs` now emits `Plan` header, palette-colored right-aligned action labels, unicode `→` target separator, and `✓ 0 actions (up to date)` empty state
   - Changes: `main.rs` and `common.rs` now align error/warning output with arrow hints and two-space indented styled warnings, plus home-path abbreviation in user-facing config path errors
   - Compatibility updates: refreshed string assertions in `output_polish_tests.rs`, `exit_code_matrix.rs`, and `phase25_schema_tests.rs` for the new human-output format
   - Gate: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` (277 total tests)
