# spec/

Implementation contract for `eden-skills` Phase 1 (Rust CLI).

## Purpose

This directory defines executable specifications for the CLI behavior.
`ROADMAP.md` explains strategy. `spec/` defines what must be built.

## Scope

- Phase 1 mandatory behavior for reconciliation commands (`plan`, `apply`, `doctor`, `repair`).
- Post-Phase-1 command UX expansion MAY be specified here when it does not conflict with Phase 1 delivery.

## Rule of Authority

When documents disagree, follow this order:

1. `spec/*.md` (normative behavior)
2. `STATUS.yaml` (machine-readable execution status)
3. `EXECUTION_TRACKER.md` (quantified progress and ownership)
4. `ROADMAP.md` (product strategy and milestones)
5. `README.md` (project summary)

## Normative Language

Keywords are interpreted as:

- `MUST`: mandatory behavior
- `SHOULD`: recommended behavior
- `MAY`: optional behavior

## Spec Files

- `SPEC_SCHEMA.md`: `skills.toml` schema, defaults, and validation
- `SPEC_AGENT_PATHS.md`: agent detection and path resolution policy
- `SPEC_COMMANDS.md`: CLI command contract and lifecycle command model
- `SPEC_TEST_MATRIX.md`: minimum acceptance test matrix
- `SPEC_TRACEABILITY.md`: requirement IDs mapped to code and tests
- `PHASE1_BUILDER_REMAINING.md`: indexed list of remaining Builder-owned Phase 1 tasks

## Contributor Workflow

1. Update the relevant spec file first.
2. Implement code to match the spec.
3. Add or update tests from `SPEC_TEST_MATRIX.md`.
4. Run `cargo fmt --all`, `cargo clippy --workspace`, and `cargo test --workspace`.
5. Fix clippy findings when possible; for unavoidable lints, use the smallest-scope `#[allow(...)]` with a brief justification.
6. Update `SPEC_TRACEABILITY.md` mappings.
7. If behavior changed, update `STATUS.yaml`, `EXECUTION_TRACKER.md`, `README.md`, and `ROADMAP.md`.

## Non-goal

Do not add Phase 2 crawler/taxonomy behavior in these specs.
