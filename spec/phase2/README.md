# Phase 2 Specifications: Hyper-Loop Core Architecture

**Status:** Stage A Complete (Exploratory Architecture Design)
**Parent:** `spec/README.md`
**Architecture Vision:** `prompt/PHASE2-STAGE-A.md`
**Next:** Stage B (Freeze Contracts)

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

| File | Domain | Requirement IDs | ADRs | Description |
| :--- | :--- | :--- | :--- | :--- |
| `SPEC_REACTOR.md` | Concurrency | ARC-001 ~ ARC-008 | ADR-002, ADR-003, ADR-004 | tokio async runtime, bounded concurrency, two-phase execution, cancellation, error strategy |
| `SPEC_ADAPTER.md` | Environment | ARC-101 ~ ARC-108 | ADR-001, ADR-005, ADR-006 | TargetAdapter trait, LocalAdapter, DockerAdapter, instantiation, Send+Sync |
| `SPEC_REGISTRY.md` | Registry | ARC-201 ~ ARC-207 | ADR-007, ADR-008, ADR-009 | Double-track registry, index format, resolution logic, version matching |
| `SPEC_SCHEMA_EXT.md` | Config | SCH-P2-001 ~ SCH-P2-006 | -- | `skills.toml` extensions (registries, version, target, reactor config) |
| `SPEC_COMMANDS_EXT.md` | CLI | CMD-P2-001 ~ CMD-P2-006 | -- | New commands: `update`, `install --target`, `--concurrency` flag |
| `SPEC_TEST_MATRIX.md` | Testing | TM-P2-001 ~ TM-P2-024 | -- | Phase 2 acceptance test scenarios |
| `SPEC_TRACEABILITY.md` | Traceability | -- | -- | Requirement-to-implementation mapping for Phase 2 |

## Architecture Decision Record Index

| ADR | Title | Location | Decision |
| :--- | :--- | :--- | :--- |
| ADR-001 | Docker Injection Strategy | `SPEC_ADAPTER.md` 5 | Docker CP (no container restart) |
| ADR-002 | Async Runtime Selection | `SPEC_REACTOR.md` 5 | tokio (green threads, ecosystem) |
| ADR-003 | Task Coordination Strategy | `SPEC_REACTOR.md` 5 | Semaphore-bounded spawn with JoinSet |
| ADR-004 | Sync-to-Async Migration | `SPEC_REACTOR.md` 5 | `spawn_blocking` for Phase 2; async process for Phase 3 |
| ADR-005 | Docker Client Strategy | `SPEC_ADAPTER.md` 5 | CLI via `tokio::process::Command` |
| ADR-006 | Adapter Instantiation Pattern | `SPEC_ADAPTER.md` 5 | Factory function with match |
| ADR-007 | Index Bucketing Strategy | `SPEC_REGISTRY.md` 5 | First-character bucketing with migration path |
| ADR-008 | Registry Sync Strategy | `SPEC_REGISTRY.md` 5 | Shallow clone (`--depth 1`) |
| ADR-009 | Version Resolution Library | `SPEC_REGISTRY.md` 5 | `semver` crate |

## Freeze Candidates Summary

Stage B must resolve these items before Builder implementation begins:

| Domain | Count | IDs |
| :--- | :--- | :--- |
| Reactor | 4 | FC-R1, FC-R2, FC-R3, FC-R4 |
| Adapter | 4 | FC-A1, FC-A2, FC-A3, FC-A5 |
| Registry | 6 | FC-REG1, FC-REG2, FC-REG3, FC-REG4, FC-REG5, FC-REG6 |
| Schema | 3 | FC-S1, FC-S2, FC-S3 |
| Commands | 3 | FC-C1, FC-C2, FC-C3 |
| **Total** | **20** | |

## Normative Language

Same as `spec/README.md`: `MUST`, `SHOULD`, `MAY` per RFC 2119.
