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
8. Current phase's `SPEC_TRACEABILITY.md`
9. `STATUS.yaml`
10. `EXECUTION_TRACKER.md`
11. `ROADMAP.md`
12. `README.md`
13. `trace/` (archived phase records — read only when historical context is needed)

## 2. Authority Order

When files disagree, follow:

1. `spec/**/*.md`
2. `STATUS.yaml`
3. `EXECUTION_TRACKER.md`
4. `ROADMAP.md`
5. `README.md`

## 3. Role Boundaries

- `Builder (Codex)` owns implementation, tests, refactors, and non-strategic doc sync.
- `Architect (Claude)` owns taxonomy/rubric/crawler strategy decisions.
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

### General

- If task is progress planning: use `STATUS.yaml` first, then `EXECUTION_TRACKER.md`.
- If task needs historical phase context: check `trace/<phase>/` archives.

## 6. Guardrails

- Preserve `skills.toml` as source-of-truth config.
- Keep command semantics deterministic and idempotent.
- Do not introduce Phase 3 crawler/taxonomy implementation into any current phase specs.
- Phase 1, 2, 2.5, 2.7, 2.8, and 2.9 spec files are frozen; changes require explicit user approval.
- Phase 2.95 extends Phase 2 adapter/reactor, Phase 2.5 install flow, Phase 2.7 remove interactive mode, and Phase 2.9 update semantics.
- Completed phase execution records are archived in `trace/<phase>/`. Root `STATUS.yaml` and `EXECUTION_TRACKER.md` only contain the active phase.
