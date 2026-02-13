# AGENTS.md

Agent coordination guide for `eden-skills`.
This file is designed for fast recovery after context compression.

## 1. Read Order (Compression-Safe)

1. `spec/README.md`
2. `spec/SPEC_*.md`
3. `spec/SPEC_TRACEABILITY.md`
4. `STATUS.yaml`
5. `EXECUTION_TRACKER.md`
6. `ROADMAP.md`
7. `README.md`

## 2. Authority Order

When files disagree, follow:

1. `spec/*.md`
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
3. Update tests, especially `spec/SPEC_TEST_MATRIX.md` scenarios.
4. Update `spec/SPEC_TRACEABILITY.md` links for changed requirements.
5. Update `STATUS.yaml` and `EXECUTION_TRACKER.md`.

## 5. Quick Start Task Routing

- If task is CLI behavior or validation: start from `spec/SPEC_COMMANDS.md` and `spec/SPEC_SCHEMA.md`.
- If task is target path logic: start from `spec/SPEC_AGENT_PATHS.md`.
- If task is verification scope: start from `spec/SPEC_TEST_MATRIX.md`.
- If task is progress planning: use `STATUS.yaml` first, then `EXECUTION_TRACKER.md`.

## 6. Guardrails

- Preserve `skills.toml` as source-of-truth config.
- Keep command semantics deterministic and idempotent.
- Do not introduce Phase 2 crawler/taxonomy implementation into Phase 1 specs.
