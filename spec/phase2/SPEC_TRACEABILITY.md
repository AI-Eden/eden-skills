# SPEC_TRACEABILITY.md

Requirement-to-implementation mapping for Phase 2.
Use this file to recover accurate context after compression.

## 1. Architecture Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
|---|---|---|---|---|---|
| ARC-001 | `SPEC_REACTOR.md` 4 | CLI MUST use `tokio` runtime for all network I/O | -- | -- | planned |
| ARC-002 | `SPEC_REACTOR.md` 4 | Skill downloads MUST be parallel with bounded concurrency (default: 10) | -- | -- | planned |
| ARC-003 | `SPEC_REACTOR.md` 4 | Disk I/O SHOULD be serialized per target path | -- | -- | planned |
| ARC-004 | `SPEC_REACTOR.md` 4 | Concurrency limit SHOULD be configurable via config and CLI flag | -- | -- | planned |
| ARC-005 | `SPEC_REACTOR.md` 4 | Reactor MUST implement two-phase execution (download then install) | -- | -- | planned |
| ARC-006 | `SPEC_REACTOR.md` 4 | Sync git ops MUST use `spawn_blocking` or async process to avoid blocking runtime | -- | -- | planned |
| ARC-007 | `SPEC_REACTOR.md` 4 | Reactor SHOULD support graceful cancellation via `CancellationToken` | -- | -- | planned |
| ARC-008 | `SPEC_REACTOR.md` 4 | Phase 2 domain errors MUST use `thiserror`; `anyhow` only at binary entry point | -- | -- | planned |
| ARC-101 | `SPEC_ADAPTER.md` 4 | System MUST define `TargetAdapter` trait decoupling intent from syscalls | -- | -- | planned |
| ARC-102 | `SPEC_ADAPTER.md` 4 | `LocalAdapter` MUST be provided for backward compatibility | -- | -- | planned |
| ARC-103 | `SPEC_ADAPTER.md` 4 | `DockerAdapter` MUST be provided using `docker` CLI | -- | -- | planned |
| ARC-104 | `SPEC_ADAPTER.md` 4 | `DockerAdapter` MUST support `cp` injection strategy | -- | -- | planned |
| ARC-105 | `SPEC_ADAPTER.md` 4 | `DockerAdapter` MUST use `tokio::process::Command` for async interaction | -- | -- | planned |
| ARC-106 | `SPEC_ADAPTER.md` 4 | Adapter selection MUST be deterministic from config `environment` field | -- | -- | planned |
| ARC-107 | `SPEC_ADAPTER.md` 4 | `TargetAdapter` SHOULD include `uninstall` method | -- | -- | planned |
| ARC-108 | `SPEC_ADAPTER.md` 4 | `TargetAdapter` MUST require `Send + Sync` bounds (for `JoinSet::spawn`) | -- | -- | planned |
| ARC-109 | `SPEC_ADAPTER.md` 4 | `LocalAdapter` MUST work on Linux, macOS, and Windows (platform APIs + tilde expansion) | -- | -- | planned |
| ARC-110 | `SPEC_ADAPTER.md` 4 | Windows symlink privilege error SHOULD include actionable remediation hint | -- | -- | planned |
| ARC-201 | `SPEC_REGISTRY.md` 4 | Configuration MUST support multiple registries with priority weights | -- | -- | planned |
| ARC-202 | `SPEC_REGISTRY.md` 4 | Resolution MUST follow priority-based fallback order | -- | -- | planned |
| ARC-203 | `SPEC_REGISTRY.md` 4 | Registry indexes MUST be local Git repos synced via `eden update` | -- | -- | planned |
| ARC-204 | `SPEC_REGISTRY.md` 4 | Registry index MUST contain `manifest.toml` with `format_version` | -- | -- | planned |
| ARC-205 | `SPEC_REGISTRY.md` 4 | Registry sync SHOULD use shallow clone (`--depth 1`) | -- | -- | planned |
| ARC-206 | `SPEC_REGISTRY.md` 4 | Registry resolution MUST work offline from cached index | -- | -- | planned |
| ARC-207 | `SPEC_REGISTRY.md` 4 | Version constraint matching MUST use `semver` crate | -- | -- | planned |

## 2. Schema Extension Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
|---|---|---|---|---|---|
| SCH-P2-001 | `SPEC_SCHEMA_EXT.md` 2 | `[registries]` section with `url`, `priority`, optional `auto_update` | -- | -- | planned |
| SCH-P2-002 | `SPEC_SCHEMA_EXT.md` 3 | Mode B skill entries (`name` + `version` + optional `registry`) | -- | -- | planned |
| SCH-P2-003 | `SPEC_SCHEMA_EXT.md` 4 | `environment` field in targets (`local`, `docker:<name>`) | -- | -- | planned |
| SCH-P2-004 | `SPEC_SCHEMA_EXT.md` 6 | Backward compatibility: Phase 1 configs remain valid without changes | -- | -- | planned |
| SCH-P2-005 | `SPEC_SCHEMA_EXT.md` 5 | `[reactor]` section with `concurrency` field (optional, default 10) | -- | -- | planned |
| SCH-P2-006 | `SPEC_SCHEMA_EXT.md` 7 | Phase 2 validation errors MUST use stable error codes | -- | -- | planned |

## 3. Command Extension Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
|---|---|---|---|---|---|
| CMD-P2-001 | `SPEC_COMMANDS_EXT.md` 2.1 | `update` command syncs registry indexes concurrently | -- | -- | planned |
| CMD-P2-002 | `SPEC_COMMANDS_EXT.md` 2.2 | `install` command resolves skills from registry by name | -- | -- | planned |
| CMD-P2-003 | `SPEC_COMMANDS_EXT.md` 2.3 | `apply`/`repair` resolve Mode B skills through registry before source sync | -- | -- | planned |
| CMD-P2-004 | `SPEC_COMMANDS_EXT.md` 2.3 | `doctor` emits Phase 2 finding codes (`REGISTRY_STALE`, `ADAPTER_HEALTH_FAIL`, `DOCKER_NOT_FOUND`) | -- | -- | planned |
| CMD-P2-005 | `SPEC_COMMANDS_EXT.md` 4 | `--concurrency` global flag overrides reactor concurrency | -- | -- | planned |
| CMD-P2-006 | `SPEC_COMMANDS_EXT.md` 2.2 | `install --dry-run` displays resolution without side effects | -- | -- | planned |

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
| TM-P2-012 | `SPEC_TEST_MATRIX.md` 2.5 | Two-phase execution | -- | planned |
| TM-P2-013 | `SPEC_TEST_MATRIX.md` 2.6 | Concurrency configuration | -- | planned |
| TM-P2-014 | `SPEC_TEST_MATRIX.md` 2.7 | Spawn blocking safety | -- | planned |
| TM-P2-015 | `SPEC_TEST_MATRIX.md` 3.5 | DockerAdapter symlink fallback | -- | planned |
| TM-P2-016 | `SPEC_TEST_MATRIX.md` 3.6 | Adapter selection determinism | -- | planned |
| TM-P2-017 | `SPEC_TEST_MATRIX.md` 3.7 | Docker binary missing | -- | planned |
| TM-P2-018 | `SPEC_TEST_MATRIX.md` 4.5 | Offline resolution | -- | planned |
| TM-P2-019 | `SPEC_TEST_MATRIX.md` 4.6 | Shallow clone efficiency | -- | planned |
| TM-P2-020 | `SPEC_TEST_MATRIX.md` 4.7 | Yanked version handling | -- | planned |
| TM-P2-021 | `SPEC_TEST_MATRIX.md` 4.8 | Registry manifest validation | -- | planned |
| TM-P2-022 | `SPEC_TEST_MATRIX.md` 4.9 | Reactor config validation | -- | planned |
| TM-P2-023 | `SPEC_TEST_MATRIX.md` 4.10 | Install dry run | -- | planned |
| TM-P2-024 | `SPEC_TEST_MATRIX.md` 3.4 | DockerAdapter permission handling | -- | planned |
| TM-P2-025 | `SPEC_TEST_MATRIX.md` 5.1 | Tilde expansion portability (HOME/USERPROFILE) | -- | planned |
| TM-P2-026 | `SPEC_TEST_MATRIX.md` 5.2 | Cross-platform symlink creation | -- | planned |
| TM-P2-027 | `SPEC_TEST_MATRIX.md` 5.3 | Windows symlink privilege error | -- | planned |
| TM-P2-028 | `SPEC_TEST_MATRIX.md` 5.4 | Cross-platform path normalization | -- | planned |
| TM-P2-029 | `SPEC_TEST_MATRIX.md` 5.5 | Windows safety detection graceful degradation | -- | planned |
| TM-P2-030 | `SPEC_TEST_MATRIX.md` 4.11 | Pre-release version resolution | -- | planned |
| TM-P2-031 | `SPEC_TEST_MATRIX.md` 4.12 | Registry staleness detection (doctor) | -- | planned |
| TM-P2-032 | `SPEC_TEST_MATRIX.md` 4.13 | Mode A/B identifier collision validation | -- | planned |
| TM-P2-033 | `SPEC_TEST_MATRIX.md` 4.14 | Install config persistence | -- | planned |

## 5. Phase 1 Windows Prerequisite Tasks

These tasks fix Phase 1 implementation for Windows compatibility. They are
tracked here because the Phase 2 CI Gate (Section 7) requires Phase 1 tests
to pass on Windows. Phase 1 specs remain frozen; these are code-only changes.

| TASK_ID | Source | Task | Implementation | Status |
|---|---|---|---|---|
| WIN-001 | `SPEC_TEST_MATRIX.md` 6.1 | `user_home_dir()` USERPROFILE fallback | `crates/eden-skills-core/src/paths.rs` | planned |
| WIN-002 | `SPEC_TEST_MATRIX.md` 6.2 | Fix hardcoded `/tmp` paths in tests (Category A: filesystem access) | `apply_repair.rs`, `paths_tests.rs` | planned |
| WIN-003 | `SPEC_TEST_MATRIX.md` 6.2 | Verify `/tmp` string placeholders pass on Windows (Category B) | Multiple test files | planned |
| WIN-004 | `SPEC_TEST_MATRIX.md` 6.3 | Add `#[cfg(windows)]` test equivalents for Unix-only tests | 5 test functions across 3 files | planned |
| WIN-005 | `SPEC_TEST_MATRIX.md` 6.4 | Enable `windows-latest` in CI workflow | `.github/workflows/ci.yml` | planned |
