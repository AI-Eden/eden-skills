# SPEC_TEST_MATRIX.md

Minimum acceptance test matrix for Phase 2 features.

## 1. Environments

- Linux (latest stable)
- macOS (latest stable)
- Docker Engine (for adapter tests)

## 2. Reactor Scenarios

### 2.1 Concurrent Download

- `apply` with 10+ skills downloads sources concurrently.
- Total time is significantly less than serial execution.
- No race conditions or file corruption.

### 2.2 Bounded Concurrency

- With concurrency limit set to 2, no more than 2 downloads run simultaneously.
- All skills still install successfully.

### 2.3 Partial Download Failure

- When 1 of 5 skills has an unreachable source, the other 4 succeed.
- Failed skill is reported with actionable diagnostics.
- Exit code is `1`.

### 2.4 Phase 1 Backward Compatibility

- Phase 1 integration tests pass without modification under the new async runtime.
- Serial behavior is preserved for disk I/O operations.

## 3. Adapter Scenarios

### 3.1 LocalAdapter Parity

- `apply` via `LocalAdapter` produces identical results to Phase 1 `apply`.
- All Phase 1 test scenarios pass when `LocalAdapter` is explicitly selected.

### 3.2 DockerAdapter Install

- `install --target docker:<container>` copies skill files into a running container.
- Files are present and readable inside the container after install.

### 3.3 DockerAdapter Health Check

- When target container is not running, health check fails with actionable error.
- Install attempt is prevented (fail fast).

### 3.4 DockerAdapter Permission Handling

- When container filesystem is read-only at target path, copy fails with
  clear error message including container name and path.

## 4. Registry Scenarios

### 4.1 Registry Update

- `update` clones registry repos on first run.
- `update` pulls latest on subsequent runs.
- Partial failure (one registry down) does not block others.

### 4.2 Registry Resolution

- `install <skill-name>` finds the skill in the configured registry index.
- When skill exists in both `official` and `forge`, higher-priority wins.
- When skill is not found in any registry, exits with `SKILL_NOT_FOUND` error.

### 4.3 Version Constraint Matching

- Exact version (`1.2.0`) matches only that version.
- Caret constraint (`^1.2`) matches compatible versions.
- When no version matches, error lists available versions.

### 4.4 Schema Extension Validation

- Config with `[registries]` section validates correctly.
- Config with Mode B skill entry (`name` + `version`) validates correctly.
- Mode A and Mode B entries can coexist in the same config.
- Mixing `id`+`source` with `name` in one entry fails validation (exit code `2`).

## 5. CI Gate (Phase 2)

A release candidate MUST pass:

- All Phase 1 scenario tests (regression gate).
- All Phase 2 reactor and registry scenario tests.
- At least one Docker adapter smoke test (may require Docker-in-Docker or
  be marked as manual in CI).
