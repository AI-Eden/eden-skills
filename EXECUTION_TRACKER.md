# EXECUTION_TRACKER.md

Execution tracker linked to `ROADMAP.md`, `README.md`, `STATUS.yaml`, and `AGENTS.md`.
This file quantifies implementation progress and enforces model responsibility boundaries.

## 1. Snapshot

- Date: 2026-03-17
- Workspace: `eden-skills`
- Most Recent Phase: **Phase 2.98 (completed)** (List Source Display, Doctor UX & Verify Dedup)

## 2. Responsibility Boundaries

- Builder MUST focus on executable implementation, tests, refactors, and non-strategic docs sync.
- Architect (Claude Opus) SHOULD own architecture RFCs, taxonomy design, curation rubric design, and model-calibration policy.
- Builder MUST NOT finalize Architect-owned strategy outputs without explicit user instruction.
- Cross-model edits SHOULD happen by contract-first handoff through `spec/` and this tracker.

## 3. Roadmap Progress (Quantified)

Legend: `[x]` completed, `[~]` in progress, `[ ]` not started

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

### 3.2 Verification and Testing

- [x] CI gate setup for Linux + macOS + Windows (`.github/workflows/ci.yml`)
- [x] Phase 2.95 closeout matrix re-verified (`fmt`, `clippy`, `test --all-targets`, Windows target check, ANSI scan); post-closeout `install.sh` PATH auto-rc follow-up revalidated on the full suite
- [x] Phase 2.97 closeout matrix verified (`fmt`, `clippy`, `test --all-targets`, Windows target check, ANSI scan) after documentation and tracker sync
- [x] Phase 2.98 closeout matrix verified (`fmt`, `clippy`, `test --all-targets`, Windows target check) after README/docs and tracker sync

Current automated tests: `476` (workspace unit/integration-style tests).

### 3.3 Phase 2.97 Work Packages

- [x] WP-1: Update concurrency fix — deduplicate refresh tasks by repo cache key (UFX-001~003)
- [x] WP-2: Table/help/parse-error styling — comfy-table custom_styling + semantic help/footer/parse-error colorization (TST-001~010)
- [x] WP-3: Interactive UX — MultiSelect for remove + install, description-on-hover (IUX-001~010)
- [x] WP-4: Cache cleanup — `clean` command, `--auto-clean`, doctor orphan check (CCL-001~007)
- [x] WP-5: Docker managed — `.eden-managed` manifest, ownership guard (DMG-001~008)
- [x] WP-6: Hint sync — `→` → `~>` spec amendment (HSY-001~002)
- [x] WP-7: Documentation — README.md + docs/ update (DOC-001~002)

### 3.4 Phase 2.98 Work Packages

- [x] WP-1: List source column — replace `Path` with `Source` in `list` table (LSR-001~003)
- [x] WP-2: Doctor UX — `--no-warning` flag, `Level` column rename, severity coloring (DUX-001~006)
- [x] WP-3: Verify dedup — short-circuit dependent checks when target missing (VDD-001~003)
- [x] WP-4: Documentation — README.md + docs/ update (DOC-001)

## 4. Pending Tasks with Planned LLM Ownership

### 4.1 Architect-Owned (Claude Opus)

- [ ] Finalize taxonomy model (L1 categories + L2 tags) for platform phase.
- [ ] Finalize curation rubric dimensions/weights/calibration loop.
- [ ] Finalize crawler strategy RFC constraints and governance policy.

### 4.2 Shared with Boundary Control

- [ ] Any change that mutates command semantics MUST be spec-first (`spec/` update before code).
- [ ] Any Architect decision consumed by Builder MUST be recorded as explicit contract items before implementation.

## 5. Phase Records

All phase execution records (both active and frozen) live in `trace/<phase>/`.
Each directory contains `status.yaml` (machine-readable) and `tracker.md`
(human-readable batch progress).

| Phase | Status | Started | Completed | Archive |
| ----- | ------ | ------- | --------- | ------- |
| Phase 1 | Frozen | — | 2026-02-13 | [trace/phase1/](trace/phase1/) |
| Phase 2 | Frozen | — | 2026-02-19 | [trace/phase2/](trace/phase2/) |
| Phase 2.5 | Frozen | 2026-02-20 | 2026-02-20 | [trace/phase2.5/](trace/phase2.5/) |
| Phase 2.7 | Frozen | 2026-02-21 | 2026-03-02 | [trace/phase2.7/](trace/phase2.7/) |
| Phase 2.8 | Frozen | 2026-03-04 | 2026-03-05 | [trace/phase2.8/](trace/phase2.8/) |
| Phase 2.9 | Frozen | 2026-03-05 | 2026-03-06 | [trace/phase2.9/](trace/phase2.9/) |
| Phase 2.95 | Frozen | 2026-03-06 | 2026-03-07 | [trace/phase2.95/](trace/phase2.95/) |
| Phase 2.97 | Frozen | 2026-03-07 | 2026-03-08 | [trace/phase2.97/](trace/phase2.97/) |
| **Phase 2.98** | **Frozen** | 2026-03-17 | 2026-03-17 | [trace/phase2.98/](trace/phase2.98/) |
