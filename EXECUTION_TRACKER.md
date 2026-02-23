# EXECUTION_TRACKER.md

Execution tracker linked to `ROADMAP.md`, `README.md`, `STATUS.yaml`, and `AGENTS.md`.
This file quantifies implementation progress and enforces model responsibility boundaries.

## 1. Snapshot

- Date: 2026-02-24
- Workspace: `eden-skills`
- Primary implementation model this cycle: `GPT-5 Codex (Builder)`

## 2. Responsibility Boundaries

- `GPT-5 Codex (Builder)` MUST focus on executable implementation, tests, refactors, and non-strategic docs sync.
- `Claude Opus (Architect)` SHOULD own architecture RFCs, taxonomy design, curation rubric design, and model-calibration policy.
- `GPT-5 Codex (Builder)` MUST NOT finalize Claude-owned strategy outputs without explicit user instruction.
- Cross-model edits SHOULD happen by contract-first handoff through `spec/` and this tracker.

## 3. Roadmap Progress (Quantified)

Legend:

- `[x]` completed
- `[~]` in progress
- `[ ]` not started

### 3.1 Roadmap Action Items

- [x] Initialize Repo (`Cargo workspace`, crates, toolchain)
- [x] Freeze Specs (`spec/README.md`, `spec/phase1/SPEC_*` baseline established)
- [x] Draft Config baseline (historical milestone; root sample `skills.toml` removed for MVP cleanliness)
- [x] Rust CLI Build (`plan/apply/doctor/repair` implemented; source sync/no-exec/strict-vs-verify precedence hardening completed; closeout audit completed)
- [x] Test Matrix completion (all 7 scenarios automated; CI hosted pass verified)
- [ ] Crawler RFC (Claude-owned)
- [ ] Curation RFC (Claude-owned)
- [x] Safety Gate MVP mechanics (safety metadata persistence, risk labels, no-exec metadata-only enforcement)
- [x] CLI UX RFC (`init/list/config export/config import/add/remove/set` implemented)
- [x] CLI framework refactor to `clap` (subcommands + flags)

Progress score (roadmap action items, Builder scope): `10 / 10 = 100%`

### 3.2 Phase 1 Mandatory Command Status (Spec)

- [x] `plan` baseline implemented
- [x] `apply` baseline implemented
- [x] `doctor` baseline implemented
- [x] `repair` baseline implemented

Progress score (mandatory command availability): `4 / 4 = 100%`

Quality note: baseline availability and test-matrix coverage are complete; production hardening remains ongoing.
Runtime note: in restricted sandboxes, default `storage.root` (`~/.local/share/...`) may be non-writable and cause `apply` failure unless overridden.

### 3.3 Verification and Testing

- [x] TOML parsing, defaults, and validation tests present
- [x] CLI global arg parsing tests present
- [x] `SPEC_TEST_MATRIX.md` scenario automation (7/7 scenarios covered by tests)
- [x] CI gate setup for Linux + macOS smoke (`.github/workflows/ci.yml`), hosted run verified (`CI` run `22000208004`)
- [x] Windows runner enabled in CI matrix (`windows-latest`) for Track A Batch 2, hosted run verified (`CI` run `22139248260`, job `cargo test (windows-latest)`).
- [x] Phase 2 closeout matrix re-verified on all targets (`CI` run `22176017545`: `ubuntu-latest`, `macos-latest`, `windows-latest`).

Current automated tests: `232` (workspace unit/integration-style tests).

## 4. Completed by GPT-5 Codex (Builder)

- [x] Migrated config format from YAML to TOML across code/spec/docs.
- [x] Implemented `thiserror`-based error types and CLI exit-code mapping.
- [x] Implemented config loading/validation (`strict` unknown-key behavior).
- [x] Implemented path resolution strategy (`~`, relative paths, agent defaults).
- [x] Implemented `plan` dry-run action generation.
- [x] Implemented baseline `apply`.
- [x] Implemented baseline `doctor`.
- [x] Implemented baseline `repair`.
- [x] Implemented source repository sync (`clone/update`) in apply/repair path.
- [x] Added local/offline git source support via `file://` URLs.
- [x] Implemented copy-mode content diff detection in plan engine.
- [x] Added end-to-end tests for fresh install, repeated apply, broken symlink repair, missing-source detection, and copy-mode update detection.
- [x] Replaced plan `--json` stub output with structured `serde_json` serialization (stable lowercase enums + reasons array).
- [x] Added permission-denied target-path test for `apply`.
- [x] Added invalid-config exit-code integration tests (`exit=2` + field-path assertions).
- [x] Upgraded `doctor` findings output to include issue `code`, `severity`, and `remediation` hints (text + JSON).
- [x] Declared stable `doctor --json` output schema in spec and added a contract test.
- [x] Expanded exit-code integration tests to cover runtime failure (`1`) and strict conflict (`3`) paths.
- [x] Strengthened path resolution tests (precedence + normalization + tilde expansion).
- [x] Declared stable `plan --json` output schema in spec and added a contract test.
- [x] Hardened symlink verification and plan matching to use canonical paths for comparisons.
- [x] Migrated CLI parsing to `clap` and introduced `init` command with `--force`.
- [x] Implemented `list` command (text + JSON inventory output).
- [x] Implemented `config export` command (normalized TOML output + JSON wrapper).
- [x] Implemented `config import` command (validated import + `--dry-run` preview).
- [x] Implemented `add/remove/set` lifecycle commands (deterministic TOML writes + validation + tests).
- [x] Implemented Safety Gate MVP mechanics (`.eden-safety.toml`, license/risk detection, no-exec metadata-only execution path).
- [x] Hardened copy-mode delta detection for edge cases (streaming compare + symlink/IO conflict reasons).
- [x] Added doctor strict/non-strict payload parity and JSON required-field stability tests.
- [x] Declared stable `list --json` output schema in spec and added a contract test.
- [x] Added CI smoke workflow for Linux + macOS (`cargo fmt/clippy/test`).
- [x] Verified hosted CI run success on both `ubuntu-latest` and `macos-latest` with clippy gate (`https://github.com/AI-Eden/eden-skills/actions/runs/22000208004`).
- [x] Updated CI workflow quality gate from `cargo check --workspace` to `cargo clippy --workspace`.
- [x] Refactored test layout to Rust mixed strategy: small unit tests in source + scenario/integration tests in per-crate `tests/`.
- [x] Introduced command-model spec for lifecycle commands (`init/add/remove/set/list/config export/import`).
- [x] Hardened source sync behavior with deterministic `cloned/updated/skipped/failed` reporting and actionable clone/fetch/checkout diagnostics.
- [x] Hardened multi-skill source sync behavior and strict-mode interaction precedence (config-ordered failure aggregation; source sync runtime failure precedence over strict conflict exit).
- [x] Hardened mixed-skill no-exec verification and strict conflict interactions (verify skip scoping + strict conflict exclusion for no-exec conflicts).
- [x] Harmonized strict conflict and post-mutation verification precedence across `apply` and `repair`.
- [x] Completed Phase 2 Track A Batch 1 (`WIN-001~WIN-004`): USERPROFILE fallback, portable test paths, Windows parity tests, and Windows ACL-based test helpers.
- [x] Completed Phase 2 Track A Batch 2 (`WIN-005`): enabled `windows-latest` in CI matrix and verified hosted Windows run (`https://github.com/AI-Eden/eden-skills/actions/runs/22139248260`).
- [x] Completed Phase 2 Track B Batch 3 (`ARC-001/002/005/006/008`): tokio runtime entrypoint, `SkillReactor` (JoinSet + Semaphore, default concurrency 10), two-phase barrier, `spawn_blocking` integration for git sync, and structured Phase 2 domain errors via `thiserror`.
- [x] Completed Phase 2 Track B Batch 4 (`ARC-101/102/103/106/108/109`): added `TargetAdapter` contract (`Send + Sync`), implemented `LocalAdapter` and `DockerAdapter` (docker CLI via tokio process), deterministic adapter environment parsing/factory selection, and adapter contract tests (including missing docker CLI and stopped container health-check failure).
- [x] Completed Phase 2 Track B Batch 5 (`ARC-201/202/207`): added registry core module (`registry.rs`) for multi-registry parsing, priority-ordered fallback resolution, and semver-based version matching, with dedicated registry tests.
- [x] Completed Phase 2 Track B Batch 6 (`SCH-P2-001/002/003/004/006`, `CMD-P2-001/002/003`): extended schema parsing for `[registries]` + Mode B + target `environment` with stable Phase 2 validation codes, implemented `update`/`install` commands, and wired `apply`/`repair` to resolve Mode B skills from cached registries before source sync, with dedicated Phase 2 schema/command tests.
- [x] Completed Phase 2 Track B Batch 7 (`ARC-003/004/007/104/105/107/110/203/204/205/206`, `SCH-P2-005`, `CMD-P2-004/005/006`): added `[reactor].concurrency` schema and CLI override chain, install dry-run mode, Phase 2 doctor findings (`REGISTRY_STALE`/`DOCKER_NOT_FOUND`/`ADAPTER_HEALTH_FAIL`), registry manifest/shallow/offline hardening, cancellation-aware reactor execution, adapter uninstall contract plus remove-time target cleanup, and Windows symlink remediation hints.
- [x] Completed Phase 2.5 Batch 1 (`SCH-P25-001/002/003`, `TM-P25-001~005`): allowed empty/omitted `skills` arrays in config loading and validation, updated `init` to generate minimal config, added empty-config plan/apply tests, and updated lifecycle baseline tests for empty-init semantics.
- [x] Completed Phase 2.5 Batch 2 (`MVP-001~008`, `TM-P25-006~015`): added source format detection (local/tree/full URL/SSH/shorthand/registry fallback), URL-mode install branch with registry-mode compatibility, skill ID derive/override/upsert, local-path no-clone install flow, and config auto-creation with missing-parent IO guard.
- [x] Completed Phase 2.5 Batch 3 (`MVP-009~015`, `TM-P25-016~025`): added SKILL.md discovery (`root`, `skills/*`, `packages/*`), URL local-path multi-skill selection (`--list`/`--all`/`--skill`), TTY interactive confirmation flow, non-TTY default-all behavior, and fallback install-as-directory warning when no SKILL.md is found.
- [x] Completed Phase 2.5 Batch 4 (`AGT-001~004`, `TM-P25-026~028`): added data-driven agent-directory detection (`claude/cursor/codex/windsurf`), wired URL-mode install target auto-detection, implemented no-agent fallback warning/default target, and enforced `--target` override bypass for detection while preserving registry-mode install target semantics.
- [x] Completed Phase 2.5 Batch 5 (`UX-001~007`, `TM-P25-031~034`): added shared CLI UI context (`ui.rs`) for color/symbol/spinner policy, integrated TTY clone spinner for URL install, enforced `NO_COLOR`/`FORCE_COLOR`/`CI` behavior and non-TTY prompt degradation, and preserved install JSON output contracts.
- [x] Closed Phase 2.5 Batch 3 follow-up gaps after interruption: extended multi-skill discovery/selection semantics to remote URL sources, made URL-mode `--list` no-side-effect for config/targets, and implemented interactive discovery truncation output for repos with more than 8 skills.
- [x] Completed Phase 2.5 discovery compatibility hardening (post-Batch 5): shipped P0 guardrail to block silent root fallback on failed `--skill` selection, expanded ecosystem discovery roots, added `.claude-plugin` manifest discovery, and added bounded recursive fallback (`max_depth=6`, `max_results=256`).
- [x] Completed Phase 2.5 default-config bootstrap hardening (post-Batch 5): switched default config path to `~/.eden-skills/skills.toml`, allowed `install` to auto-create missing default parent directory, and preserved missing-parent failure for non-default `--config` paths.
- [x] Completed Phase 2.5 Batch 6 (`DST-001~003`, `TM-P25-035~036`): made CLI crate publishable as `eden-skills` for `cargo install`, added a multi-platform release workflow with `v*` tag trigger and 5-target packaging, added SHA-256 release checksum generation, and added dedicated distribution TDD coverage (`distribution_tests`).
- [x] Completed Phase 2.5 post-Batch 6 agent support expansion: aligned supported `--target` aliases with skills.sh Supported Agents set, adopted project-path-derived global path defaults for new agents, expanded auto-detection rule coverage, staged local-path installs into canonical storage root before fan-out, added remove-time cleanup scans across known local agent roots + canonical storage, migrated default `storage.root` to `~/.eden-skills/skills`, added Windows no-symlink hardcopy fallback warning behavior for `install`, and added regression tests for alias parsing/path resolution/detection/cleanup.
- [x] Completed Phase 2.5 closeout readiness + tagged release dry-run: fixed `--help` non-zero exit regression that would break release smoke checks, added release-smoke contract test coverage, and validated local host-target release packaging + checksum + smoke sequence.
- [x] Completed Phase 2.7 Batch 3 (WP-2 — Help System): `HLP-001`~`HLP-007`; `--version`/`-V`, root `about`/`long_about`/`after_help`, subcommand `about`, argument `help`, `next_help_heading` groupings, short flags `-s`/`-t`/`-y`/`-V`, `install --copy`; `help_system_tests.rs` (TM-P27-016~021).

## 5. Pending Tasks with Planned LLM Ownership

### 5.1 Builder-Owned (GPT-5 Codex)

- [x] Harden copy-mode delta detection for edge cases (symlink-in-tree, large-file strategy, permission anomalies).
- [x] Expand integration assertions depth (doctor strict/non-strict parity and stable JSON contract fields).
- [x] Implement Safety Gate MVP mechanics (license check wiring, risk flag scan, no-exec mode plumbing).
- [x] Align CI workflow quality gate with local clippy-first process (`cargo clippy --workspace`).
- [x] Migrate CLI argument parsing to `clap` subcommands/flags.
- [x] Implement lifecycle commands incrementally: `init` -> `list` -> `config export` -> `config import` -> `add/remove/set`.
- [x] Harden source sync edge cases and error reporting (`clone/fetch/checkout` diagnostics + deterministic skipped/updated reporting).
- [x] Harden multi-skill partial-failure and strict-mode interactions for `apply`/`repair`.
- [x] Harden multi-skill no-exec and verify interactions for `apply`/`repair`.
- [x] Harmonize strict conflict and verify-failure precedence for `apply`/`repair`.
- [x] Complete Phase 1 Builder closeout audit (command-spec parity, traceability completeness, test-matrix consistency).
- [x] Complete Phase 2 Track B Batch 3 P0 Reactor (`ARC-001/002/005/006/008`) with tests and quality gate.
- [x] Complete Phase 2 Track B Batch 4 P0 Adapter (`ARC-101/102/103/106/108/109`) with tests and quality gate.
- [x] Complete Phase 2 Track B Batch 5 P0 Registry (`ARC-201/202/207`) with tests and quality gate.
- [x] Complete Phase 2 Track B Batch 6 P0 Schema + Commands (`SCH-P2-001/002/003/004/006`, `CMD-P2-001/002/003`) with tests and quality gate.
- [x] Complete Phase 2 Track B Batch 7 P1 All (`ARC-003/004/007/104/105/107/110/203/204/205/206`, `SCH-P2-005`, `CMD-P2-004/005/006`) with tests and quality gate.
- [x] P2-CLOSE-001: Fixed Windows CI blocker in Phase 2 command tests via TOML-safe file URL normalization and regression test coverage (`phase2_commands`); hosted matrix verification completed in `CI` run `22176017545`.
- [x] P2-CLOSE-002: Closed remaining `planned` Phase 2 matrix scenarios with full implementation coverage (`TM-P2-003/004/015/020/024/027/028/029/030`).
- [x] P2-CLOSE-003: Aligned release-closeout status wording across `README.md`, `ROADMAP.md`, `STATUS.yaml`, and this tracker.
- [x] Maintain `spec/phase2/PHASE2_BUILDER_REMAINING.md` as the concise index for remaining Builder-owned Phase 2 closeout work.

### 5.2 Architect-Owned (Claude Opus)

- [x] Phase 2 architecture contracts (Stage A: exploratory design, Stage B: contract freeze).
- [ ] Finalize taxonomy model (L1 categories + L2 tags) for platform phase.
- [ ] Finalize curation rubric dimensions/weights/calibration loop.
- [ ] Finalize crawler strategy RFC constraints and governance policy.

### 5.3 Shared with Boundary Control

- [ ] Any change that mutates command semantics MUST be spec-first (`spec/` update before code).
- [ ] Any Architect decision consumed by Builder MUST be recorded as explicit contract items before implementation.

## 6. Builder State (Phase 1)

1. No unresolved Builder-owned Phase 1 tasks at this checkpoint.

### 6.1 Completed Checklist (B-027)

- [x] Verified command-behavior parity against `spec/phase1/SPEC_COMMANDS.md` (no mismatches found).
- [x] Verified `spec/phase1/SPEC_TRACEABILITY.md` requirement mappings remain complete and status-consistent.
- [x] Verified `spec/phase1/SPEC_TEST_MATRIX.md` scenarios remain fully represented by automated tests.
- [x] Updated `spec/phase1/PHASE1_BUILDER_REMAINING.md` as the concise index of unresolved Builder tasks.

### 6.2 Phase 2 Closeout State (Builder)

1. Builder-owned Phase 2 implementation batches are complete through Batch 7.
2. Builder-owned closeout work items `P2-CLOSE-001` through `P2-CLOSE-003` are completed; hosted matrix verification is confirmed in `CI` run `22176017545`.
3. Previously deferred hardening scenarios `TM-P2-015`, `TM-P2-027`, and `TM-P2-029` are now implemented and covered by deterministic tests (Windows-specific suites are `#[cfg(windows)]` gated).
4. Canonical closeout index: `spec/phase2/PHASE2_BUILDER_REMAINING.md`.

## 7. Phase 2 Architect State

### 7.1 Completed by Claude Opus (Architect)

- [x] Phase 2 Stage A: exploratory architecture design (SPEC_REACTOR, SPEC_ADAPTER, SPEC_REGISTRY, SPEC_SCHEMA_EXT, SPEC_COMMANDS_EXT, SPEC_TEST_MATRIX, SPEC_TRACEABILITY).
- [x] Phase 2 Stage B: contract freeze (2026-02-18).
  - [x] Resolved 20 Freeze Candidates across 5 domains (Reactor, Adapter, Registry, Schema, Commands).
  - [x] Resolved 5 Open Questions (OQ-001 through OQ-005).
  - [x] Added Rollback Trigger to all ADRs missing it (ADR-003, ADR-004, ADR-005, ADR-006, ADR-007, ADR-008, ADR-009).
  - [x] Added 4 new test scenarios (TM-P2-030 through TM-P2-033) from Stage B resolutions.
  - [x] Updated SPEC_TRACEABILITY.md with all 33 test matrix entries.
  - [x] Verified all requirement IDs unique across Phase 2.
  - [x] Verified all P0 requirements have verification entries.
  - [x] Verified no conflict with Phase 1 contracts.
  - [x] Updated STATUS.yaml with Phase 2 frozen status and Builder entry criteria.

### 7.2 Builder Handoff (Phase 2)

Builder implementation can start. Tasks are organized into two independent
tracks that MAY execute in parallel.

#### Track A: Windows Prerequisites (Phase 1 code-only, no Phase 2 dependency)

These tasks fix Phase 1 implementation for Windows compatibility. They have
**zero dependency on Phase 2 architecture** and SHOULD start immediately
(before or in parallel with Track B). ARC-109 (LocalAdapter cross-platform)
depends on WIN-001 being completed first.

1. **WIN-001~004**: Source and test fixes (USERPROFILE fallback, `/tmp` paths, `#[cfg(windows)]` equivalents) — **completed (Batch 1, 2026-02-18)**
2. **WIN-005**: Enable `windows-latest` in CI (gate: WIN-001~004 pass first) — **completed (Batch 2, 2026-02-18; hosted run verified in CI run `22139248260`)**

#### Track B: Phase 2 Architecture Implementation (depends on frozen contracts)

Recommended priority order (P0 first, sequential within track):

1. **P0 Reactor**: ARC-001, ARC-002, ARC-005, ARC-006, ARC-008 (tokio runtime, bounded concurrency, two-phase execution, spawn_blocking, thiserror) — **completed (Batch 3, 2026-02-18)**
2. **P0 Adapter**: ARC-101, ARC-102, ARC-103, ARC-106, ARC-108, ARC-109 (TargetAdapter trait, Local, Docker, deterministic selection, Send+Sync, cross-platform — note: ARC-109 depends on WIN-001) — **completed (Batch 4, 2026-02-18)**
3. **P0 Registry**: ARC-201, ARC-202, ARC-207 (multi-registry config, priority fallback, semver crate) — **completed (Batch 5, 2026-02-18)**
4. **P0 Schema**: SCH-P2-001~004, SCH-P2-006 (registries section, Mode B, environment, backward compat, error codes) — **completed (Batch 6, 2026-02-18)**
5. **P0 Commands**: CMD-P2-001~003 (update, install, apply/repair Mode B support) — **completed (Batch 6, 2026-02-18)**
6. **P1 All**: ARC-003, ARC-004, ARC-007, ARC-104, ARC-105, ARC-107, ARC-110, ARC-203~206, SCH-P2-005, CMD-P2-004~006 — **completed (Batch 7, 2026-02-19)**

#### Dependency Graph

```text
Track A (Windows):  WIN-001~004 ──→ WIN-005 (enable CI)
                        │
                        ▼
Track B (Phase 2):  P0 Reactor ──→ P0 Adapter ──→ P0 Registry ──→ P0 Schema ──→ P0 Commands ──→ P1
                                   (ARC-109 needs WIN-001)
```

Key architectural decisions for Builder reference:

- ADR-002: tokio async runtime
- ADR-003: JoinSet + Semaphore for task coordination
- ADR-004: spawn_blocking for sync-to-async migration
- ADR-005: Docker CLI via tokio::process::Command
- ADR-006: Factory function with match for adapter instantiation
- ADR-007: First-character index bucketing
- ADR-008: Shallow clone for registry sync
- ADR-009: semver crate for version resolution

## 8. Phase 2.5 Builder State

### 8.1 Batch Progress

1. Batch 1 (WS-1 + WS-2) is complete with quality gate pass:
   - Requirements: `SCH-P25-001`, `SCH-P25-002`, `SCH-P25-003`
   - Scenarios: `TM-P25-001` through `TM-P25-005`
   - Gate: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`
2. Batch 2 (WS-3 part 1) is complete with quality gate pass:
   - Requirements: `MVP-001` through `MVP-008`
   - Scenarios: `TM-P25-006` through `TM-P25-015`
   - Additional covered scenarios: `TM-P25-029`, `TM-P25-030`
   - Gate: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`
3. Batch 3 (WS-3 part 2) is complete with quality gate pass:
   - Requirements: `MVP-009` through `MVP-015`
   - Scenarios: `TM-P25-016` through `TM-P25-025`
   - Follow-up hardening: remote URL parity for `--list`/`--all`/`--skill` and interactive summary truncation for >8 discovered skills
   - Gate: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`
4. Batch 4 (WS-4) is complete with quality gate pass:
   - Requirements: `AGT-001` through `AGT-004`
   - Scenarios: `TM-P25-026` through `TM-P25-028` (and regression retention for `TM-P25-029`, `TM-P25-030`)
   - Gate: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`
5. Batch 5 (WS-7) is complete with quality gate pass:
   - Requirements: `UX-001` through `UX-007`
   - Scenarios: `TM-P25-031` through `TM-P25-034`
   - Gate: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`
6. Post-Batch 5 discovery compatibility hardening is complete:
   - Requirements: `MVP-009`, `MVP-012`
   - Scenarios: `TM-P25-023`, `TM-P25-037`, `TM-P25-038`, `TM-P25-039`, `TM-P25-040`
   - Gate: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`
7. Post-Batch 5 default-config bootstrap hardening is complete:
   - Requirement: `MVP-008`
   - Scenarios: `TM-P25-030`, `TM-P25-041`
   - Gate: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`
8. Batch 6 (WS-5) is complete with quality gate pass:
   - Requirements: `DST-001`, `DST-002`, `DST-003`
   - Scenarios: `TM-P25-035`, `TM-P25-036`
   - Gate: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`
9. Post-Batch 6 agent support expansion is complete with quality gate pass:
   - Scope: expanded `--target` alias matrix and project-path-derived global path defaults for newly supported agents, and switched default `storage.root` to `~/.eden-skills/skills`
   - Regression coverage: alias parsing and remove cleanup (`config_lifecycle`), default-path resolution (`paths_tests`), default storage-root fallback (`config_tests` + `init_command`), auto-detection (`agent_detect_tests`/`install_agent_detect_tests`), local-source staging (`install_url_tests`), and Windows hardcopy fallback warning (`install_url_tests`)
   - Gate: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`
10. `spec/phase2.5/SPEC_INSTALL_URL.md`, `SPEC_TEST_MATRIX.md`, `SPEC_TRACEABILITY.md`, and schema defaults (`phase1/phase2/phase2.5`) are synchronized with implemented distribution and agent/discovery behavior.
11. Phase 2.5 closeout readiness + tagged release dry-run is complete with quality gate pass. Added release-smoke contract regression for `eden-skills --help` success semantics (`distribution_tests`); validated local host-target archive packaging + checksum generation + smoke sequence (`--help`, `init`, `install ... --all`); gate: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`.

12. Next recommended execution target: repository public-readiness checklist and first real tag release execution.

## 9. Phase 2.7 Builder State

### 9.1 Batch Progress

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
