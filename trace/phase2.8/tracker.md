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
4. Batch 4 (WP-2 part 2 — Category A-2 User-Facing Commands) is complete with quality gate pass:
   - Requirements: `OUP-003`, `OUP-005`, `OUP-006`, `OUP-007`, `OUP-018`, `OUP-020`
   - Scenarios: `TM-P28-018`, `TM-P28-019`, `TM-P28-020`, `TM-P28-023`, `TM-P28-024`, `TM-P28-025`
   - Additions: new `output_upgrade_a2_tests.rs` with TM-aligned assertions for doctor styled header/cards/conditional summary table, init next-steps guidance block, and install per-target/discovery output
   - Changes: `diagnose.rs` now emits UiContext-based `Doctor` header, severity-symbol findings cards, dimmed `→` remediation lines, and a conditional `Sev | Code | Skill` summary table for 4+ findings
   - Changes: `config_ops.rs` now emits `✓ Created config at ~/.eden-skills/skills.toml` style output plus a 3-line `Next steps:` block with dimmed command descriptions
   - Changes: `install.rs` now emits numbered `Found` discovery summaries, per-target `Install  ✓ skill → path (mode)` lines, and final `N skills installed to M agents, K conflicts` summary output
   - Compatibility updates: refreshed legacy doctor text assertion in `doctor_output.rs` to match card-based human output while keeping `doctor --json` contract unchanged
   - Gate: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` (283 total tests)
5. Batch 5 (WP-2 part 3 — Category B Table-Based New Designs) is complete with quality gate pass:
   - Requirements: `TBL-003`, `TBL-004`, `TBL-005`, `TBL-006`, `TBL-007`, `OUP-008`, `OUP-009`, `OUP-010`, `OUP-011`, `OUP-012`
   - Scenarios: `TM-P28-005`, `TM-P28-006`, `TM-P28-007`, `TM-P28-008`, `TM-P28-009`, `TM-P28-010`, `TM-P28-011`, `TM-P28-029`, `TM-P28-030`, `TM-P28-031`
   - Additions: new `output_upgrade_b_tests.rs` with TM-aligned assertions for list/install/plan/update table output and non-TTY, `--color never`, and `--json` table-exclusion guarantees
   - Changes: `config_ops.rs` `list` now emits action-prefixed `Skills` header + `Skill | Mode | Source | Agents` table, with source repo abbreviation and metadata-only suffix handling
   - Changes: `install.rs` now renders dry-run targets as `Agent | Path | Mode` table under a metadata header and renders `install --list` as numbered `# | Name | Description` table with >8 truncation footer
   - Changes: `plan_cmd.rs` now switches to table mode (`Action | Skill | Target | Mode`) when action count exceeds 5 and preserves text mode for 5 or fewer actions
   - Changes: `update.rs` now renders registry sync results as `Registry | Status | Detail` table with action-prefix header and failed-count timing footer, while preserving JSON contract
   - Compatibility updates: refreshed `list_command.rs` and `phase2_commands.rs` assertions for new human output while keeping existing behavior and side-effect guarantees intact
   - Gate: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` (293 total tests)
6. Batch 6 (WP-3b — Doc Comments) is complete with quality gate pass:
   - Requirements: `CST-004`, `CST-005`, `CST-006`, `CST-007`, `CST-008`
   - Scenarios: `TM-P28-003`, `TM-P28-034`, `TM-P28-035`, `TM-P28-036`
   - Additions: `//!` module docs on all 9 CLI `commands/` sub-modules, CLI `lib.rs`, and 8 core modules (`reactor.rs`, `lock.rs`, `adapter.rs`, `source_format.rs`, `discovery.rs`, `config.rs`, `plan.rs`, `error.rs`)
   - Additions: `//!` crate-level doc on core `lib.rs`
   - Additions: `///` doc comments with `# Errors` on all public command functions (`install_async`, `apply_async`, `repair_async`, `doctor`, `plan`, `init`, `list`, `add`, `set`, `config_export`, `config_import`, `remove_many_async`, `update_async`, `remove`, `remove_async`, `apply`, `repair`)
   - Additions: `///` doc comments on shared utilities in `common.rs` (`resolve_config_path`, `load_config_with_context`, `ensure_git_available`, `ensure_docker_available_for_targets`)
   - Additions: `///` doc comments on core public types and functions (`SkillReactor`, `run_phase_a`, `run_blocking`, `compute_lock_diff`, `TargetAdapter`, `LocalAdapter`, `DockerAdapter`, `detect_install_source`, `discover_skills`, `validate_config`, `LoadedConfig`, `build_plan`, `Action`, `EdenError`, `ReactorError`, `AdapterError`, `RegistryError`)
   - Additions: `//!` module doc and `///` on all public items in `ui.rs` (`UiContext`, `UiSpinner`, `ColorWhen`, `StatusSymbol`, `configure_color_output`, `color_output_enabled`, `table()`, `spinner()`, `abbreviate_home_path`, `abbreviate_repo_url`)
   - Additions: `///` on `print_error` in `main.rs`
   - Tests: new `doc_coverage_tests.rs` with 6 tests (TM-P28-003 ×3, TM-P28-034, TM-P28-035, TM-P28-036)
   - Gate: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` (299 total tests)
