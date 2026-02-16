# SPEC_TRACEABILITY.md

Requirement-to-implementation mapping for Phase 2.
Use this file to recover accurate context after compression.

## 1. Architecture Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
|---|---|---|---|---|---|
| ARC-001 | `SPEC_REACTOR.md` 4 | CLI MUST use `tokio` runtime for all network I/O | -- | -- | planned |
| ARC-002 | `SPEC_REACTOR.md` 4 | Skill downloads MUST be parallel with bounded concurrency (default: 10) | -- | -- | planned |
| ARC-003 | `SPEC_REACTOR.md` 4 | Disk I/O SHOULD be serialized per target path | -- | -- | planned |
| ARC-101 | `SPEC_ADAPTER.md` 4 | System MUST define `TargetAdapter` trait decoupling intent from syscalls | -- | -- | planned |
| ARC-102 | `SPEC_ADAPTER.md` 4 | `LocalAdapter` MUST be provided for backward compatibility | -- | -- | planned |
| ARC-103 | `SPEC_ADAPTER.md` 4 | `DockerAdapter` MUST be provided using `docker` CLI or API | -- | -- | planned |
| ARC-104 | `SPEC_ADAPTER.md` 4 | `DockerAdapter` MUST support `cp` injection strategy | -- | -- | planned |
| ARC-201 | `SPEC_REGISTRY.md` 4 | Configuration MUST support multiple registries with priority weights | -- | -- | planned |
| ARC-202 | `SPEC_REGISTRY.md` 4 | Resolution MUST follow priority-based fallback order | -- | -- | planned |
| ARC-203 | `SPEC_REGISTRY.md` 4 | Registry indexes MUST be local Git repos synced via `eden update` | -- | -- | planned |

## 2. Schema Extension Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
|---|---|---|---|---|---|
| SCH-P2-001 | `SPEC_SCHEMA_EXT.md` 2 | `[registries]` section with `url`, `priority`, optional `auto_update` | -- | -- | planned |
| SCH-P2-002 | `SPEC_SCHEMA_EXT.md` 3 | Mode B skill entries (`name` + `version` + optional `registry`) | -- | -- | planned |
| SCH-P2-003 | `SPEC_SCHEMA_EXT.md` 4 | `environment` field in targets (`local`, `docker:<name>`) | -- | -- | planned |
| SCH-P2-004 | `SPEC_SCHEMA_EXT.md` 5 | Backward compatibility: Phase 1 configs remain valid without changes | -- | -- | planned |

## 3. Command Extension Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
|---|---|---|---|---|---|
| CMD-P2-001 | `SPEC_COMMANDS_EXT.md` 2.1 | `update` command syncs registry indexes concurrently | -- | -- | planned |
| CMD-P2-002 | `SPEC_COMMANDS_EXT.md` 2.2 | `install` command resolves skills from registry by name | -- | -- | planned |
| CMD-P2-003 | `SPEC_COMMANDS_EXT.md` 2.3 | `apply`/`repair` resolve Mode B skills through registry before source sync | -- | -- | planned |
| CMD-P2-004 | `SPEC_COMMANDS_EXT.md` 2.3 | `doctor` emits Phase 2 finding codes (REGISTRY_STALE, ADAPTER_HEALTH_FAIL) | -- | -- | planned |

## 4. Test Matrix Coverage

| SCENARIO_ID | Source | Scenario | Automated Test | Status |
|---|---|---|---|---|
| TM-P2-001 | `SPEC_TEST_MATRIX.md` 2.1 | Concurrent download | -- | planned |
| TM-P2-002 | `SPEC_TEST_MATRIX.md` 2.2 | Bounded concurrency | -- | planned |
| TM-P2-003 | `SPEC_TEST_MATRIX.md` 2.3 | Partial download failure | -- | planned |
| TM-P2-004 | `SPEC_TEST_MATRIX.md` 2.4 | Phase 1 backward compatibility | -- | planned |
| TM-P2-005 | `SPEC_TEST_MATRIX.md` 3.1 | LocalAdapter parity | -- | planned |
| TM-P2-006 | `SPEC_TEST_MATRIX.md` 3.2 | DockerAdapter install | -- | planned |
| TM-P2-007 | `SPEC_TEST_MATRIX.md` 3.3 | DockerAdapter health check | -- | planned |
| TM-P2-008 | `SPEC_TEST_MATRIX.md` 4.1 | Registry update | -- | planned |
| TM-P2-009 | `SPEC_TEST_MATRIX.md` 4.2 | Registry resolution | -- | planned |
| TM-P2-010 | `SPEC_TEST_MATRIX.md` 4.3 | Version constraint matching | -- | planned |
| TM-P2-011 | `SPEC_TEST_MATRIX.md` 4.4 | Schema extension validation | -- | planned |
