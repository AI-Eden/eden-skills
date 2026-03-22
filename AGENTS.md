# AGENTS.md

Agent coordination guide for `eden-skills`.
This file is designed for fast recovery after context compression.

## 1. Read Order (Compression-Safe)

1. `spec/README.md`
2. `spec/phase1/SPEC_*.md` (Phase 1 CLI behavior contracts)
3. `spec/phase2/SPEC_*.md` (Phase 2 architecture contracts)
4. `spec/phase2.5/SPEC_*.md` (Phase 2.5 MVP launch contracts)
5. `spec/phase2.7/SPEC_*.md` (Phase 2.7 UX polish & lock file contracts)
6. `spec/phase2.8/SPEC_*.md` (Phase 2.8 TUI deep optimization & code maintainability contracts)
7. `spec/phase2.95/SPEC_*.md` (Phase 2.95 performance, platform reach & UX completeness contracts)
8. `spec/phase2.97/SPEC_*.md` (Phase 2.97 reliability, interactive UX & Docker safety contracts)
9. Current phase's `SPEC_TRACEABILITY.md`
10. `STATUS.yaml`
11. `EXECUTION_TRACKER.md`
12. `ROADMAP.md`
13. `README.md`
14. `trace/` (archived phase records — read only when historical context is needed)

## 2. Authority Order

When files disagree, follow:

1. `spec/**/*.md`
2. `STATUS.yaml`
3. `EXECUTION_TRACKER.md`
4. `ROADMAP.md`
5. `README.md`

## 3. Role Boundaries

- `Builder (GPT 5.3 Codex / GPT 5.4)` owns implementation, tests, refactors, and non-strategic doc sync.
- `Architect (Claude Opus 4.6 / Claude Sonnet 4.6)` owns taxonomy/rubric/crawler strategy decisions.
- Builder must not finalize Architect-owned strategy outputs without explicit user instruction.

## 4. Change Protocol

1. Update `spec/` first for behavior changes.
2. Implement code to match spec.
3. Update tests per the current phase's `SPEC_TEST_MATRIX.md`.
4. Update the current phase's `SPEC_TRACEABILITY.md` links for changed requirements.
5. Update `STATUS.yaml` and `EXECUTION_TRACKER.md`.

## 5. Quick Start Task Routing

### Phase 1 (CLI Foundation)

- If task is CLI behavior or validation: start from `spec/phase1/SPEC_COMMANDS.md` and `spec/phase1/SPEC_SCHEMA.md`.
- If task is target path logic: start from `spec/phase1/SPEC_AGENT_PATHS.md`.
- If task is verification scope: start from `spec/phase1/SPEC_TEST_MATRIX.md`.

### Phase 2 (Hyper-Loop Core)

- If task is concurrency or async runtime: start from `spec/phase2/SPEC_REACTOR.md`.
- If task is environment adapter (Local/Docker): start from `spec/phase2/SPEC_ADAPTER.md`.
- If task is registry resolution: start from `spec/phase2/SPEC_REGISTRY.md`.
- If task is Phase 2 schema extension: start from `spec/phase2/SPEC_SCHEMA_EXT.md`.
- If task is Phase 2 new commands: start from `spec/phase2/SPEC_COMMANDS_EXT.md`.

### Phase 2.5 (MVP Launch)

- If task is install from URL or source format parsing: start from `spec/phase2.5/SPEC_INSTALL_URL.md`.
- If task is schema amendment (empty skills, init): start from `spec/phase2.5/SPEC_SCHEMA_P25.md`.
- If task is agent auto-detection: start from `spec/phase2.5/SPEC_AGENT_DETECT.md`.
- If task is CLI output beautification: start from `spec/phase2.5/SPEC_CLI_UX.md`.
- If task is binary distribution or release workflow: start from `spec/phase2.5/SPEC_DISTRIBUTION.md`.

### Phase 2.7 (UX Polish & Lock File)

- If task is lock file or orphan removal: start from `spec/phase2.7/SPEC_LOCK.md`.
- If task is help text, version info, or command grouping: start from `spec/phase2.7/SPEC_HELP_SYSTEM.md`.
- If task is color library, error messages, or `--color` flag: start from `spec/phase2.7/SPEC_OUTPUT_POLISH.md`.
- If task is batch remove or interactive remove: start from `spec/phase2.7/SPEC_REMOVE_ENH.md`.

### Phase 2.8 (TUI Deep Optimization & Code Maintainability)

- If task is table rendering or `comfy-table` integration: start from `spec/phase2.8/SPEC_TABLE_RENDERING.md`.
- If task is command output format, UiContext, or error hint format: start from `spec/phase2.8/SPEC_OUTPUT_UPGRADE.md`.
- If task is `commands.rs` decomposition or doc comments: start from `spec/phase2.8/SPEC_CODE_STRUCTURE.md`.

### Phase 2.95 (Performance, Platform Reach & UX Completeness)

- If task is source sync performance or repo-level cache: start from `spec/phase2.95/SPEC_PERF_SYNC.md`.
- If task is remove wildcard or remove-all: start from `spec/phase2.95/SPEC_REMOVE_ALL.md`.
- If task is Windows junction or symlink fallback: start from `spec/phase2.95/SPEC_WINDOWS_JUNCTION.md`.
- If task is Docker bind mount or `docker mount-hint`: start from `spec/phase2.95/SPEC_DOCKER_BIND.md`.
- If task is install script or `cargo-binstall`: start from `spec/phase2.95/SPEC_INSTALL_SCRIPT.md`.

### Phase 2.97 (Reliability, Interactive UX & Docker Safety)

- If task is update concurrency bug or refresh dedup: start from `spec/phase2.97/SPEC_UPDATE_FIX.md`.
- If task is table content styling or `custom_styling` feature: start from `spec/phase2.97/SPEC_TABLE_STYLE.md`.
- If task is interactive remove/install MultiSelect UX: start from `spec/phase2.97/SPEC_INTERACTIVE_UX.md`.
- If task is cache cleanup or `clean` command: start from `spec/phase2.97/SPEC_CACHE_CLEAN.md`.
- If task is Docker management domain or `.eden-managed`: start from `spec/phase2.97/SPEC_DOCKER_MANAGED.md`.
- If task is hint arrow prefix or `~>` styling: start from `spec/phase2.97/SPEC_HINT_SYNC.md`.

### General

- If task is progress planning: use `STATUS.yaml` first, then `EXECUTION_TRACKER.md`.
- If task needs historical phase context: check `trace/<phase>/` archives.

## 6. Guardrails

- Preserve `skills.toml` as source-of-truth config.
- Keep command semantics deterministic and idempotent.
- Do not introduce Phase 3 crawler/taxonomy implementation into any current phase specs.
- Phase 1, 2, 2.5, 2.7, 2.8, and 2.9 spec files are frozen; changes require explicit user approval.
- Phase 2.95 extends Phase 2 adapter/reactor, Phase 2.5 install flow, Phase 2.7 remove interactive mode, and Phase 2.9 update semantics.
- Phase 2.97 fixes Phase 2.95 update concurrency, replaces Phase 2.95 remove `*` wildcard with MultiSelect, extends Phase 2.8 table styling, adds cache cleanup, and introduces Docker management domain tracking.
- Completed phase execution records are archived in `trace/<phase>/`. Root `STATUS.yaml` and `EXECUTION_TRACKER.md` only contain the current execution state (the active phase, or the most recently closed phase before the next phase starts).

## Cursor Cloud specific instructions

### Project overview

`eden-skills` is a pure Rust CLI tool (no web server, no database, no external services). The workspace contains three crates: `eden-skills-cli` (binary), `eden-skills-core` (library), and `eden-skills-indexer` (Phase 3 stub). The only runtime dependency beyond the Rust toolchain is `git` on `PATH`.

### Build, lint, and test

Standard commands from CI (see `.github/workflows/ci.yml`):

```
cargo build --workspace
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
```

The full test suite (~476 tests) takes ~4 minutes due to several integration tests with 30-second timeouts (these exercise async reactor timeout behavior and are expected). All tests are deterministic and require no external services.

### Running the CLI in dev mode

```
cargo run --bin eden-skills -- <subcommand>
```

The binary is `eden-skills`. Use `--help` for the full command list. A typical dev-loop hello-world: `init` → `add --id ... --repo ... --target ...` → `list` → `apply` → `doctor`.

### Non-obvious caveats

- **Docker tests**: Some tests stub `docker` commands via shell scripts rather than requiring a real Docker daemon. The test suite passes without Docker installed.
- **`rust-toolchain.toml`** pins the `stable` channel with `rustfmt` + `clippy` components. Rustup auto-installs the correct toolchain on first `cargo` invocation.
- **No hot-reload**: This is a compiled CLI, not a long-running server. After code changes, rebuild with `cargo build --workspace` or use `cargo run` which rebuilds automatically.
