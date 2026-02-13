# spec/

Implementation contract for `eden-skills` Phase 1 (Rust CLI).

## Purpose

This directory defines executable specifications for the CLI behavior.
`ROADMAP.md` explains strategy. `spec/` defines what must be built.

## Scope

Phase 1 commands and behavior only:

- `plan`
- `apply`
- `doctor`
- `repair`

## Rule of Authority

When documents disagree, follow this order:

1. `spec/*.md` (normative behavior)
2. `ROADMAP.md` (product strategy and milestones)
3. `README.md` (project summary)

## Normative Language

Keywords are interpreted as:

- `MUST`: mandatory behavior
- `SHOULD`: recommended behavior
- `MAY`: optional behavior

## Spec Files

- `SPEC_SCHEMA.md`: `skills.toml` schema, defaults, and validation
- `SPEC_AGENT_PATHS.md`: agent detection and path resolution policy
- `SPEC_COMMANDS.md`: CLI contract for `plan/apply/doctor/repair`
- `SPEC_TEST_MATRIX.md`: minimum acceptance test matrix

## Contributor Workflow

1. Update the relevant spec file first.
2. Implement code to match the spec.
3. Add or update tests from `SPEC_TEST_MATRIX.md`.
4. If behavior changed, update `README.md` and `ROADMAP.md` references.

## Non-goal

Do not add Phase 2 crawler/taxonomy behavior in these specs.
