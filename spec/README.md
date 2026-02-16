# spec/

Implementation contracts for `eden-skills` across all phases.

## Purpose

This directory defines executable specifications for CLI behavior and architecture.
`ROADMAP.md` explains strategy. `spec/` defines what must be built.

## Directory Structure

```
spec/
├── README.md              (this file - master index)
├── phase1/                (Phase 1: CLI Foundation - FROZEN)
│   ├── SPEC_COMMANDS.md
│   ├── SPEC_SCHEMA.md
│   ├── SPEC_AGENT_PATHS.md
│   ├── SPEC_TEST_MATRIX.md
│   ├── SPEC_TRACEABILITY.md
│   └── PHASE1_BUILDER_REMAINING.md
└── phase2/                (Phase 2: Hyper-Loop Core Architecture)
    ├── README.md
    ├── SPEC_REACTOR.md
    ├── SPEC_ADAPTER.md
    ├── SPEC_REGISTRY.md
    ├── SPEC_SCHEMA_EXT.md
    ├── SPEC_COMMANDS_EXT.md
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

- `phase2/SPEC_REACTOR.md`: tokio concurrency model and task queue (ARC-001~003)
- `phase2/SPEC_ADAPTER.md`: TargetAdapter trait, LocalAdapter, DockerAdapter (ARC-101~104)
- `phase2/SPEC_REGISTRY.md`: double-track registry system, index format, resolution logic (ARC-201~203)
- `phase2/SPEC_SCHEMA_EXT.md`: `skills.toml` Phase 2 extensions (registries, version, target environments)
- `phase2/SPEC_COMMANDS_EXT.md`: Phase 2 new commands (update, install --target)
- `phase2/SPEC_TEST_MATRIX.md`: Phase 2 acceptance test scenarios
- `phase2/SPEC_TRACEABILITY.md`: Phase 2 requirement-to-implementation mapping

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

1. Identify which phase the change belongs to (`phase1/` or `phase2/`).
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

When reading an `_EXT` file, always read the corresponding Phase 1 base file first.
The base file defines the foundation; the `_EXT` file defines additive changes only.
`_EXT` files MUST NOT contradict or override Phase 1 base semantics.
