# spec/

Implementation contracts for `eden-skills` across all phases.

## Purpose

This directory defines executable specifications for CLI behavior and architecture.
`ROADMAP.md` explains strategy. `spec/` defines what must be built.

## Directory Structure

```txt
spec/
├── README.md              (this file - master index)
├── phase1/                (Phase 1: CLI Foundation - FROZEN)
│   ├── SPEC_COMMANDS.md
│   ├── SPEC_SCHEMA.md
│   ├── SPEC_AGENT_PATHS.md
│   ├── SPEC_TEST_MATRIX.md
│   ├── SPEC_TRACEABILITY.md
│   └── PHASE1_BUILDER_REMAINING.md
├── phase2/                (Phase 2: Hyper-Loop Core Architecture - FROZEN)
│   ├── README.md
│   ├── SPEC_REACTOR.md
│   ├── SPEC_ADAPTER.md
│   ├── SPEC_REGISTRY.md
│   ├── SPEC_SCHEMA_EXT.md
│   ├── SPEC_COMMANDS_EXT.md
│   ├── SPEC_TEST_MATRIX.md
│   ├── SPEC_TRACEABILITY.md
│   └── PHASE2_BUILDER_REMAINING.md
├── phase2.5/              (Phase 2.5: MVP Launch)
│   ├── README.md
│   ├── SPEC_INSTALL_URL.md
│   ├── SPEC_SCHEMA_P25.md
│   ├── SPEC_AGENT_DETECT.md
│   ├── SPEC_CLI_UX.md
│   ├── SPEC_DISTRIBUTION.md
│   ├── SPEC_TEST_MATRIX.md
│   └── SPEC_TRACEABILITY.md
└── phase2.7/              (Phase 2.7: UX Polish & Lock File)
    ├── README.md
    ├── SPEC_LOCK.md
    ├── SPEC_HELP_SYSTEM.md
    ├── SPEC_OUTPUT_POLISH.md
    ├── SPEC_REMOVE_ENH.md
    ├── SPEC_TEST_MATRIX.md
    └── SPEC_TRACEABILITY.md
```

## Phase 1: CLI Foundation (FROZEN)

Phase 1 specs are frozen. Content changes require explicit user approval.

- `phase1/SPEC_COMMANDS.md`: CLI command contract and lifecycle command model
- `phase1/SPEC_SCHEMA.md`: `skills.toml` schema, defaults, and validation
- `phase1/SPEC_AGENT_PATHS.md`: agent detection and path resolution policy
- `phase1/SPEC_TEST_MATRIX.md`: minimum acceptance test matrix
- `phase1/SPEC_TRACEABILITY.md`: requirement IDs mapped to code and tests
- `phase1/PHASE1_BUILDER_REMAINING.md`: indexed list of remaining Builder-owned Phase 1 tasks

## Phase 2: Hyper-Loop Core Architecture

Phase 2 specs define the async runtime, environment adapter, and registry system.
Files with `_EXT` suffix extend Phase 1 base contracts (read the base file first).

- `phase2/SPEC_REACTOR.md`: tokio concurrency model, two-phase execution, cancellation, error strategy (ARC-001~008)
- `phase2/SPEC_ADAPTER.md`: TargetAdapter trait, LocalAdapter, DockerAdapter, instantiation, Send+Sync, Windows portability (ARC-101~110)
- `phase2/SPEC_REGISTRY.md`: double-track registry, index format, resolution logic, version matching (ARC-201~207)
- `phase2/SPEC_SCHEMA_EXT.md`: `skills.toml` Phase 2 extensions (registries, version, target, reactor config)
- `phase2/SPEC_COMMANDS_EXT.md`: Phase 2 new commands (update, install --target, --concurrency flag)
- `phase2/SPEC_TEST_MATRIX.md`: Phase 2 acceptance test scenarios
- `phase2/SPEC_TRACEABILITY.md`: Phase 2 requirement-to-implementation mapping
- `phase2/PHASE2_BUILDER_REMAINING.md`: indexed list of remaining Builder-owned Phase 2 closeout tasks

## Phase 2.5: MVP Launch

Phase 2.5 bridges the implemented Phase 2 architecture to a usable MVP product.
It adds URL-based install, agent auto-detection, CLI beautification, and binary
distribution — without introducing any Phase 3 features (crawler, taxonomy, curation).

- `phase2.5/SPEC_INSTALL_URL.md`: install from URL with source format parsing, SKILL.md discovery, interactive flow (MVP-001~015)
- `phase2.5/SPEC_SCHEMA_P25.md`: schema amendment for empty skills array and minimal init template (SCH-P25-001~003)
- `phase2.5/SPEC_AGENT_DETECT.md`: agent auto-detection for install targets (AGT-001~004)
- `phase2.5/SPEC_CLI_UX.md`: CLI output beautification with colors, spinners, symbols (UX-001~007)
- `phase2.5/SPEC_DISTRIBUTION.md`: binary distribution via GitHub Releases and cargo install (DST-001~003)
- `phase2.5/SPEC_TEST_MATRIX.md`: Phase 2.5 acceptance test scenarios (TM-P25-001~036)
- `phase2.5/SPEC_TRACEABILITY.md`: Phase 2.5 requirement-to-implementation mapping

## Rule of Authority

When documents disagree, follow this order:

1. `spec/**/*.md` (normative behavior)
2. `STATUS.yaml` (machine-readable execution status)
3. `EXECUTION_TRACKER.md` (quantified progress and ownership)
4. `ROADMAP.md` (product strategy and milestones)
5. `README.md` (project summary)

## Normative Language

Keywords are interpreted as:

- `MUST`: mandatory behavior
- `SHOULD`: recommended behavior
- `MAY`: optional behavior

## Contributor Workflow

1. Identify which phase the change belongs to (`phase1/`, `phase2/`, or `phase2.5/`).
2. Update the relevant spec file first.
3. Implement code to match the spec.
4. Add or update tests from the corresponding `SPEC_TEST_MATRIX.md`.
5. Run `cargo fmt --all`, `cargo clippy --workspace`, and `cargo test --workspace`.
6. Fix clippy findings when possible; for unavoidable lints, use the smallest-scope `#[allow(...)]` with a brief justification.
7. Update the corresponding `SPEC_TRACEABILITY.md` mappings.
8. If behavior changed, update `STATUS.yaml`, `EXECUTION_TRACKER.md`, `README.md`, and `ROADMAP.md`.

## Cross-Phase Extension Convention

Phase 2 `_EXT` spec files extend Phase 1 base contracts:

- `SPEC_SCHEMA_EXT.md` extends `phase1/SPEC_SCHEMA.md`
- `SPEC_COMMANDS_EXT.md` extends `phase1/SPEC_COMMANDS.md`

Phase 2.5 `_P25` spec files amend Phase 1 or extend Phase 2 contracts:

- `SPEC_SCHEMA_P25.md` amends `phase1/SPEC_SCHEMA.md` (relaxes one validation rule)
- `SPEC_INSTALL_URL.md` extends `phase2/SPEC_COMMANDS_EXT.md` (adds URL-mode install)

When reading an extension file, always read the corresponding base file first.
The base file defines the foundation; extension files define additive changes only.
Extension files MUST NOT contradict base semantics except where explicitly noted
as an amendment (Phase 2.5 `SPEC_SCHEMA_P25.md` Section 2 and Phase 2.7
`SPEC_OUTPUT_POLISH.md` Section 2 are the documented exceptions).

## Phase 2.7: UX Polish & Lock File

Phase 2.7 polishes the CLI user experience and introduces a lock file for
diff-driven reconciliation.

- `phase2.7/SPEC_LOCK.md`: `skills.lock` format, lifecycle, and diff-driven apply with `Remove` action (LCK-001~010)
- `phase2.7/SPEC_HELP_SYSTEM.md`: help text, version info, command grouping, short flags (HLP-001~007)
- `phase2.7/SPEC_OUTPUT_POLISH.md`: `owo-colors` migration, error refinement, `--color` flag (OUT-001~008)
- `phase2.7/SPEC_REMOVE_ENH.md`: batch remove, interactive selection, `-y`/`--yes` flag (RMV-001~005)
- `phase2.7/SPEC_TEST_MATRIX.md`: Phase 2.7 acceptance test scenarios (TM-P27-001~040)
- `phase2.7/SPEC_TRACEABILITY.md`: Phase 2.7 requirement-to-implementation mapping
