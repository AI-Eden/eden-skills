# EXECUTION_TRACKER.md

Execution tracker linked to `ROADMAP.md`, `README.md`, `STATUS.yaml`, and `AGENTS.md`.
This file quantifies implementation progress and enforces model responsibility boundaries.

## 1. Snapshot

- Date: 2026-02-13
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
- [x] Draft Config (`skills.toml` with 5 skills)
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

Current automated tests: `73` (workspace unit/integration-style tests).

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

### 5.2 Architect-Owned (Claude Opus)

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
