# Phase 2 Specifications: Hyper-Loop Core Architecture

**Status:** Draft
**Parent:** `spec/README.md`
**Architecture Spec:** `prompt/PHASE2-STAGE-B.md` (Frozen Contract)

## Purpose

Phase 2 evolves `eden-skills` from a local file-linker (Phase 1) to a high-performance,
environment-agnostic package manager with async concurrency, Docker injection, and
double-track registry resolution.

## Relationship to Phase 1

Phase 1 specs (`spec/phase1/`) are frozen and define the CLI foundation.
Phase 2 specs in this directory either:

1. **Introduce new domains** (Reactor, Adapter, Registry) with no Phase 1 counterpart.
2. **Extend Phase 1 contracts** via `_EXT` files that add capabilities without modifying base semantics.

When reading an `_EXT` file, always read the corresponding Phase 1 base file first:

- `SPEC_SCHEMA_EXT.md` extends `spec/phase1/SPEC_SCHEMA.md`
- `SPEC_COMMANDS_EXT.md` extends `spec/phase1/SPEC_COMMANDS.md`

## Spec Files

| File | Domain | Requirement IDs | Description |
| :--- | :--- | :--- | :--- |
| `SPEC_REACTOR.md` | Concurrency | ARC-001 ~ ARC-003 | tokio async runtime, bounded concurrency, task queue |
| `SPEC_ADAPTER.md` | Environment | ARC-101 ~ ARC-104 | TargetAdapter trait, LocalAdapter, DockerAdapter |
| `SPEC_REGISTRY.md` | Registry | ARC-201 ~ ARC-203 | Double-track registry, index format, resolution logic |
| `SPEC_SCHEMA_EXT.md` | Config | SCH-P2-xxx | `skills.toml` Phase 2 extensions (registries, version, target) |
| `SPEC_COMMANDS_EXT.md` | CLI | CMD-P2-xxx | New commands: `update`, `install --target` |
| `SPEC_TEST_MATRIX.md` | Testing | TM-P2-xxx | Phase 2 acceptance test scenarios |
| `SPEC_TRACEABILITY.md` | Traceability | -- | Requirement-to-implementation mapping for Phase 2 |

## Normative Language

Same as `spec/README.md`: `MUST`, `SHOULD`, `MAY` per RFC 2119.
