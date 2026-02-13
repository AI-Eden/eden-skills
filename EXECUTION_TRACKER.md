# EXECUTION_TRACKER.md

Execution tracker linked to `ROADMAP.md` and `README.md`.
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
- [x] Freeze Specs (`spec/README.md`, `SPEC_*` baseline established)
- [x] Draft Config (`skills.toml` with 5 skills)
- [~] Rust CLI Build (`plan/apply/doctor/repair` implemented; source clone/update wired; deep edge-case hardening pending)
- [~] Test Matrix completion (fresh/repeated/broken/moved/copy/permission automated; invalid config + CI hosted pass pending)
- [ ] Crawler RFC (Claude-owned)
- [ ] Curation RFC (Claude-owned)
- [ ] Safety Gate MVP (license gate, risk labels, no-exec mode)
- [~] CLI UX RFC (`init/add/remove/set/list/config export/import` contract captured in spec; code not started)
- [ ] CLI framework refactor to `clap` (planned)

Progress score (roadmap action items): `6.5 / 10 = 65%`

### 3.2 Phase 1 Mandatory Command Status (Spec)

- [x] `plan` baseline implemented
- [x] `apply` baseline implemented
- [x] `doctor` baseline implemented
- [x] `repair` baseline implemented

Progress score (mandatory command availability): `4 / 4 = 100%`

Quality note: baseline availability is complete; production hardening and test-matrix coverage remain incomplete.
Runtime note: in restricted sandboxes, default `storage.root` (`~/.local/share/...`) may be non-writable and cause `apply` failure unless overridden.

### 3.3 Verification and Testing

- [x] TOML parsing, defaults, and validation tests present
- [x] CLI global arg parsing tests present
- [~] `SPEC_TEST_MATRIX.md` scenario automation (6/7 scenarios covered by tests)
- [~] CI gate setup for Linux + macOS smoke (`.github/workflows/ci.yml`), first hosted run pending

Current automated tests: `11` (workspace unit/integration-style tests).

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
- [x] Added CI smoke workflow for Linux + macOS (`cargo fmt/check/test`).
- [x] Refactored test layout to Rust mixed strategy: small unit tests in source + scenario/integration tests in per-crate `tests/`.
- [x] Introduced command-model spec for lifecycle commands (`init/add/remove/set/list/config export/import`).

## 5. Pending Tasks with Planned LLM Ownership

### 5.1 Builder-Owned (GPT-5 Codex)

- [~] Harden copy-mode delta detection for edge cases (symlink-in-tree, large-file strategy, permission anomalies).
- [ ] Add integration tests covering every scenario in `SPEC_TEST_MATRIX.md`.
- [ ] Implement Safety Gate MVP mechanics (license check wiring, risk flag scan, no-exec mode plumbing).
- [ ] Migrate CLI argument parsing to `clap` subcommands/flags.
- [ ] Implement lifecycle commands incrementally: `init` -> `list` -> `config export` -> `add/remove/set` -> `config import`.

### 5.2 Architect-Owned (Claude Opus)

- [ ] Finalize taxonomy model (L1 categories + L2 tags) for platform phase.
- [ ] Finalize curation rubric dimensions/weights/calibration loop.
- [ ] Finalize crawler strategy RFC constraints and governance policy.

### 5.3 Shared with Boundary Control

- [ ] Any change that mutates command semantics MUST be spec-first (`spec/` update before code).
- [ ] Any Architect decision consumed by Builder MUST be recorded as explicit contract items before implementation.

## 6. Next Execution Target (Builder)

1. Complete remaining invalid-config/exit-code scenario coverage from `SPEC_TEST_MATRIX.md`.
2. Upgrade `doctor` output to include issue code + severity + remediation hint (spec-aligned).
3. Migrate argument parsing to `clap` and start lifecycle commands from `init`.
