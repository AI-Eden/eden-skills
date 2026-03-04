# SPEC_CODE_STRUCTURE.md

Code structure decomposition and documentation coverage for
`eden-skills`.

**Related contracts:**

- `spec/phase2.8/SPEC_OUTPUT_UPGRADE.md` (output changes that motivate
  the split)

## 1. Purpose

The CLI crate's `commands.rs` has grown to ~3 768 lines, containing all
command implementations, output formatting, and shared utilities in a
single file. This spec mandates:

1. **Module decomposition** of `commands.rs` into focused sub-modules.
2. **Doc comment coverage** for public APIs across both CLI and Core
   crates, following Rust documentation best practices.

The decomposition is a **pure refactoring** — no behavioral changes, no
new features. All existing tests MUST pass without modification after
the split.

## 2. Module Decomposition

### 2.1 Target Structure

The current `commands.rs` MUST be replaced with a `commands/` directory
containing the following modules:

```text
crates/eden-skills-cli/src/
├── main.rs
├── lib.rs
├── ui.rs
└── commands/
    ├── mod.rs
    ├── install.rs
    ├── reconcile.rs
    ├── diagnose.rs
    ├── plan_cmd.rs
    ├── config_ops.rs
    ├── remove.rs
    ├── update.rs
    └── common.rs
```

### 2.2 Module Responsibilities

| Module | Contents | Approximate Scope |
| :--- | :--- | :--- |
| `mod.rs` | Public re-exports, `CommandOptions`, `InstallRequest`, `AddRequest`, `SetRequest`, `UpdateRequest` types | Type definitions and `pub use` |
| `install.rs` | `install_async`, URL/registry/local mode dispatch, `UrlInstallSource` handling, skill discovery and selection flow, dry-run output, install result output | ~800 lines |
| `reconcile.rs` | `apply_async`, `repair_async`, source sync execution, plan execution (`apply_plan_item`), lock lifecycle within apply/repair, orphan removal | ~500 lines |
| `diagnose.rs` | `doctor`, `collect_doctor_findings`, `collect_phase2_doctor_findings`, `collect_registry_stale_findings`, `collect_adapter_health_findings`, doctor output formatting | ~400 lines |
| `plan_cmd.rs` | `plan` command entry, `print_plan_text`, `print_plan_json`, `build_remove_plan_items`, `action_label` | ~150 lines |
| `config_ops.rs` | `init`, `list`, `add`, `set`, `config_export`, `config_import`, `default_config_template` | ~400 lines |
| `remove.rs` | `remove_many_async`, `resolve_remove_ids`, `validate_remove_ids`, `confirm_remove_execution`, `print_remove_summary`, `print_remove_candidates`, interactive selection | ~300 lines |
| `update.rs` | `update_async`, `RegistrySyncTask`, `RegistrySyncResult`, `sync_registry_task`, `sync_registry_task_blocking`, registry sync marker I/O | ~200 lines |
| `common.rs` | `resolve_config_path`, `load_config_with_context`, `write_normalized_config`, `parse_target_specs`, `ensure_git_available`, `ensure_docker_available_for_targets`, `resolve_effective_reactor_concurrency`, `block_on_command_future`, `copy_recursively`, `remove_path`, `ensure_parent_dir`, path/adapter helpers, `print_source_sync_summary`, `print_safety_summary` | ~500 lines |

### 2.3 Decomposition Rules

1. **No behavioral changes.** The public API surface of
   `eden_skills_cli::commands` MUST remain identical. All functions
   that are `pub` in the current `commands.rs` MUST remain `pub` and
   accessible via the same `commands::function_name` path.

2. **Internal visibility.** Functions that are currently private in
   `commands.rs` and used only within one command group SHOULD become
   `pub(super)` in their new module. Functions shared across multiple
   modules MUST be placed in `common.rs` and made `pub(super)`.

3. **Import organization.** Each sub-module MUST organize imports per
   Rust convention: `std` → external crates → workspace crates →
   `super`/`crate`.

4. **Zero test changes.** All 253+ existing tests MUST pass without
   modification. The test files reference `eden_skills_cli::commands`
   and `eden_skills_cli::run_with_args` — these paths MUST continue
   to work.

### 2.4 `mod.rs` Re-Exports

The `mod.rs` file MUST re-export all public items so that external
callers (tests, `lib.rs`) see no difference:

```rust
mod common;
mod config_ops;
mod diagnose;
mod install;
mod plan_cmd;
mod reconcile;
mod remove;
mod update;

pub use common::*;
pub use config_ops::*;
pub use diagnose::*;
pub use install::*;
pub use plan_cmd::*;
pub use reconcile::*;
pub use remove::*;
pub use update::*;
```

Builder MAY use more selective re-exports if wildcard re-exports cause
name collisions. The key constraint is that the existing test and
`lib.rs` call sites compile without changes.

## 3. Doc Comment Coverage

### 3.1 Philosophy

Following Rust best practices (Chapter 8, Apollo GraphQL handbook):

- `///` doc comments explain **what** a function/type does, **how** to
  use it, and **when** it fails (`# Errors`, `# Panics`).
- `//` inline comments explain **why** — safety guarantees, performance
  trade-offs, non-obvious design decisions, links to specs or ADRs.
- `//!` module-level docs explain **purpose** of the module, its
  exports, and its relationship to other modules.
- Comments that restate obvious code behavior MUST NOT be added.
- Comments that could be replaced by better naming or smaller functions
  SHOULD be replaced by refactoring instead.

### 3.2 CLI Crate Coverage

| File | Doc Comment Scope |
| :--- | :--- |
| `ui.rs` | `//!` module doc explaining UiContext design. `///` on `UiContext`, `UiSpinner`, `ColorWhen`, `StatusSymbol`, `configure_color_output`, `color_output_enabled`, and the new `table()` method. Inline `//` on `resolve_colors_enabled` explaining the precedence chain. |
| `commands/mod.rs` | `//!` module doc explaining the command dispatch architecture. `///` on `CommandOptions`, `InstallRequest`, `AddRequest`, `SetRequest`, `UpdateRequest`. |
| `commands/install.rs` | `//!` doc covering the three install modes. `///` on `install_async` covering input semantics, side effects (config write, lock write, git clone), and exit-code-producing errors. |
| `commands/reconcile.rs` | `//!` doc covering the apply/repair lifecycle. `///` on `apply_async` and `repair_async` covering the phase sequence (sync → safety → orphan removal → plan → execute → verify → lock). |
| `commands/diagnose.rs` | `//!` doc covering doctor's finding collection pipeline. `///` on `doctor` covering finding sources (plan conflicts, verify issues, safety reports, phase2 findings). |
| `commands/plan_cmd.rs` | `///` on `plan` covering lock-diff integration and read-only contract. |
| `commands/config_ops.rs` | `///` on `init`, `list`, `add`, `set`, `config_export`, `config_import` covering side effects and validation. |
| `commands/remove.rs` | `///` on `remove_many_async` covering atomic validation and interactive flow. |
| `commands/update.rs` | `///` on `update_async` covering reactor-based concurrent sync. |
| `commands/common.rs` | `///` on shared utilities: `resolve_config_path` (path resolution rules), `load_config_with_context` (strict-mode error wrapping), `ensure_git_available` / `ensure_docker_available_for_targets` (preflight semantics). |
| `main.rs` | `///` on `print_error` covering the error display pipeline and `split_hint` convention. |

### 3.3 Core Crate Coverage

| File | Doc Comment Scope |
| :--- | :--- |
| `lib.rs` | `//!` crate-level doc explaining eden-skills-core's role as the domain logic layer (no UI, no I/O formatting). Module dependency overview. |
| `reactor.rs` | `//!` doc covering the two-phase execution model. `///` on `SkillReactor` explaining `JoinSet` + `Semaphore` coordination. `///` on `run_phase_a` / `run_blocking` covering cancellation semantics. Inline `//` on concurrency bounds explaining the `MIN`/`MAX` rationale. |
| `lock.rs` | `//!` doc covering the lock file lifecycle (create/read/diff/write). `///` on `compute_lock_diff` explaining the three-way diff algorithm (TOML ∩ Lock → Added/Changed/Unchanged/Removed). `///` on `read_lock_file` covering missing/corrupted fallback. |
| `adapter.rs` | `//!` doc covering the `TargetAdapter` abstraction. `///` on the trait explaining `Send + Sync` bounds (required for reactor spawning). `///` on `LocalAdapter` and `DockerAdapter` covering behavioral differences. Inline `//` on Windows symlink fallback in `LocalAdapter::install`. |
| `source_format.rs` | `//!` doc covering the source format detection pipeline. `///` on `detect_install_source` explaining the precedence: registry name → GitHub URL variants → local path. Inline `//` on `looks_like_local_path` Windows handling. |
| `discovery.rs` | `//!` doc covering SKILL.md search strategy. `///` on `discover_skills` explaining depth limit (6), result cap (256), `.git` exclusion, and parent-directory sniff order. |
| `config.rs` | `///` on `validate_config` explaining the error code taxonomy (INVALID_SKILL_MODE, MISSING_REGISTRIES, etc.). `///` on `LoadedConfig` explaining the warnings collection pattern. Inline `//` on Mode A vs Mode B skill distinction. |
| `plan.rs` | `///` on `build_plan` explaining how `Action` variants are determined. `///` on the `Action` enum variants. |
| `error.rs` | `///` on `EdenError` variants explaining which exit code each maps to. `///` on domain error types (`ReactorError`, `AdapterError`, `RegistryError`) explaining when each arises. |

### 3.4 Constraints

1. Doc comments MUST NOT exceed 3 sentences for the summary line.
   Detailed explanation goes in subsequent paragraphs.
2. `# Errors` sections MUST be included for functions returning
   `Result`.
3. `TODO` comments MUST reference an issue or spec section. Bare
   `// TODO: fix this` is not acceptable.
4. The `#![deny(missing_docs)]` lint is NOT required at this stage.
   Builder SHOULD add it to `lib.rs` of the core crate as a stretch
   goal, with `#[allow(missing_docs)]` on items that are not yet
   documented.

## 4. Normative Requirements

| ID | Owner | Priority | Statement | Verification |
| :--- | :--- | :--- | :--- | :--- |
| **CST-001** | Builder | **P0** | `commands.rs` MUST be decomposed into sub-modules per Section 2.1. | `commands/` directory exists with all listed modules. |
| **CST-002** | Builder | **P0** | Decomposition MUST NOT change any CLI behavior. | All 253+ existing tests pass without modification. |
| **CST-003** | Builder | **P0** | Public API of `eden_skills_cli::commands` MUST remain unchanged. | `lib.rs` and test imports compile without changes. |
| **CST-004** | Builder | **P0** | Every CLI `commands/` sub-module MUST have a `//!` module doc. | Each file starts with `//!` documentation. |
| **CST-005** | Builder | **P0** | Every public command function MUST have a `///` doc comment with `# Errors`. | Public functions in `commands/` have doc comments. |
| **CST-006** | Builder | **P1** | Core crate modules listed in Section 3.3 MUST have `//!` module docs. | Each listed core module file starts with `//!`. |
| **CST-007** | Builder | **P1** | Core crate public functions listed in Section 3.3 MUST have `///` doc comments. | Listed public functions have doc comments. |
| **CST-008** | Builder | **P1** | `ui.rs` MUST have `//!` module doc and `///` on all public items. | All public items in `ui.rs` have doc comments. |

## 5. Backward Compatibility

| Existing Feature | Phase 2.8 Behavior |
| :--- | :--- |
| `eden_skills_cli::commands::*` paths | Unchanged via re-exports. |
| Test file structure | Unchanged. No test files need to move. |
| `lib.rs` command dispatch | Unchanged. `commands::install_async` etc. still work. |
| Core crate public API | Unchanged. Only doc comments added. |
