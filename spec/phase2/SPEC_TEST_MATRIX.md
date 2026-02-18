# SPEC_TEST_MATRIX.md

Minimum acceptance test matrix for Phase 2 features.

## 1. Environments

- Linux (latest stable)
- macOS (latest stable)
- Windows (latest stable) — symlink tests require Developer Mode or
  `SeCreateSymbolicLinkPrivilege`; GitHub Actions Windows runners have
  this privilege by default.
- Docker Engine (for adapter tests; Linux/macOS CI only, Docker Desktop
  on Windows is optional/manual)

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

### 2.5 Two-Phase Execution (NEW)

- No install mutation occurs while any download is still in progress.
- Verify by adding a slow-downloading skill and confirming install steps
  only begin after all downloads complete (or fail).

### 2.6 Concurrency Configuration (NEW)

- `--concurrency 1` produces serial download behavior (one skill at a time).
- `[reactor] concurrency = 5` in config limits to 5 concurrent downloads.
- CLI `--concurrency` overrides config value.

### 2.7 Spawn Blocking Safety (NEW)

- No tokio "blocking the runtime" warnings during concurrent operations
  with 20+ skills.
- `spawn_blocking` tasks do not exhaust the blocking thread pool under
  normal concurrency limits (1-100).

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

### 3.5 DockerAdapter Symlink Fallback (NEW)

- When `install.mode = "symlink"` is configured for a Docker target,
  a warning is emitted and copy mode is used instead.
- Files are correctly installed despite mode mismatch.

### 3.6 Adapter Selection Determinism (NEW)

- `environment = "local"` always selects LocalAdapter.
- `environment = "docker:test"` always selects DockerAdapter.
- Unknown environment string fails at config validation time (exit code `2`).

### 3.7 Docker Binary Missing (NEW)

- When `docker` is not in PATH and a Docker target is configured,
  error message clearly states Docker CLI is required.

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

### 4.5 Offline Resolution (NEW)

- With network disabled and registry cache present, `install <name>` resolves
  from cached index without errors.
- With network disabled and no registry cache, `install <name>` fails with
  "Run `eden-skills update` first" message.

### 4.6 Shallow Clone Efficiency (NEW)

- `update` uses shallow clone (`--depth 1`) for initial clone.
- Registry local directory does not contain full git history.

### 4.7 Yanked Version Handling (NEW)

- Yanked versions are excluded from constraint resolution.
- When the only matching version is yanked, error lists available non-yanked versions.

### 4.8 Registry Manifest Validation (NEW)

- Registry with `manifest.toml` containing `format_version = 1` parses correctly.
- Registry without `manifest.toml` logs a warning and assumes format version 1.

### 4.9 Reactor Config Validation (NEW)

- Config with `[reactor] concurrency = 5` validates correctly.
- Config with `[reactor] concurrency = 0` fails validation (exit code `2`).
- Config with `[reactor] concurrency = 101` fails validation (exit code `2`).

### 4.10 Install Dry Run (NEW)

- `install --dry-run <skill-name>` displays resolved source and target info.
- No config file modification occurs.
- No filesystem changes occur.

## 5. Cross-Platform Scenarios

### 5.1 Tilde Expansion Portability

- `~/.claude/skills` resolves correctly using `HOME` on Linux/macOS.
- `~/.claude/skills` resolves correctly using `USERPROFILE` on Windows
  when `HOME` is unset.
- When both `HOME` and `USERPROFILE` are set, `HOME` takes precedence.

### 5.2 Cross-Platform Symlink Creation

- `LocalAdapter` creates symlinks on Linux, macOS, and Windows.
- On Windows, directory sources use `symlink_dir`, file sources use `symlink_file`.

### 5.3 Windows Symlink Privilege Error

- When symlink creation fails due to insufficient Windows privileges,
  error message includes actionable remediation (Developer Mode / admin).
- No panic or opaque OS error code shown to user.

### 5.4 Cross-Platform Path Normalization

- `normalize_lexical` correctly handles `\` path separators on Windows
  and `/` separators on Unix.
- `Component::Prefix` (Windows drive letter, e.g., `C:\`) is preserved.

### 5.5 Windows Safety Detection Graceful Degradation

- Unix executable permission check (`mode() & 0o111`) is skipped on Windows
  without error.
- Other safety checks (ELF header, shebang, file extension) still produce
  correct risk labels on Windows.

## 6. Phase 1 Windows Prerequisite Tasks

The CI Gate (Section 7) requires Phase 1 tests to pass on Windows. The
following Phase 1 implementation fixes MUST be completed by the Builder
before Phase 2 Windows CI can be enabled. These are implementation-only
changes; Phase 1 specs (`spec/phase1/`) remain frozen and do not require
modification.

### 6.1 Source Code Fix: `user_home_dir()` USERPROFILE Fallback

- **File:** `crates/eden-skills-core/src/paths.rs` (function `user_home_dir`)
- **Current:** `env::var("HOME")` only.
- **Required:** Fallback to `env::var("USERPROFILE")` when `HOME` is unset.
- **Scope:** 1 function, ~3 lines changed.

### 6.2 Test Fixes: Hardcoded `/tmp` Paths (12+ instances)

Replace hardcoded `/tmp/` paths in tests with platform-portable alternatives.
Two categories:

**Category A — Filesystem access (MUST fix):**

| File | Line(s) | Current | Fix |
| :--- | :--- | :--- | :--- |
| `cli/tests/apply_repair.rs` | 80 | `Path::new("/tmp/eden-skills-broken")` | Use `tempdir()` |
| `core/tests/paths_tests.rs` | 103-104 | `Path::new("/tmp/a/./b/../c")` | Platform-conditional assertion or `tempdir()` based path |

**Category B — String placeholders (SHOULD verify, likely no change needed):**

| File | Line(s) | Current | Risk |
| :--- | :--- | :--- | :--- |
| `core/tests/plan_json_contract.rs` | 9-10 | `"/tmp/source"`, `"/tmp/target"` | Low — string comparison only |
| `core/tests/config_tests.rs` | 80 | `"file:///tmp/repo.git"` | Low — URL format validation only |
| `core/tests/plan_copy_edge_tests.rs` | 40, 99, 162 | `"file:///tmp/placeholder"` | Low — config construction, not accessed |
| `core/tests/symlink_canonical_tests.rs` | 34, 75 | `"file:///tmp/unused"` | Low — config construction, not accessed |
| `cli/tests/list_json_contract.rs` | 23 | `"file:///tmp/unused"` | Low — config construction, not accessed |
| `cli/tests/list_command.rs` | 23, 71 | `"file:///tmp/unused"` | Low — config construction, not accessed |
| `core/tests/safety_tests.rs` | 106 | `"file:///tmp/origin.git"` | Low — config construction, not accessed |

### 6.3 Test Fixes: `#[cfg(unix)]`-Only Tests Without Windows Equivalents

The following test functions are `#[cfg(unix)]` only. On Windows they are
silently skipped, reducing coverage. The Builder SHOULD add `#[cfg(windows)]`
equivalents where the underlying behavior has a Windows counterpart.

| File | Test Function | Unix Dependency | Windows Equivalent |
| :--- | :--- | :--- | :--- |
| `core/tests/symlink_canonical_tests.rs` | entire `mod unix` | `std::os::unix::fs::symlink` | Use `std::os::windows::fs::symlink_dir` |
| `cli/tests/apply_repair.rs` | `apply_fails_on_permission_denied_target_path` | Unix `chmod 000` | Use `icacls` or Windows ACL denial |
| `cli/tests/exit_code_matrix.rs` | `apply_strict_returns_exit_code_3_on_target_conflict` | Unix symlink for conflict setup | Use Windows symlink APIs |
| `core/tests/plan_copy_edge_tests.rs` | `copy_mode_plan_conflict_on_unreadable_target_file` | Unix `set_mode(0o000)` | Use Windows ACL or read-only attribute |
| `core/tests/plan_copy_edge_tests.rs` | `copy_mode_plan_conflict_on_symlink_in_tree` | Unix symlink API | Use Windows symlink APIs |

### 6.4 CI Workflow Update

- **File:** `.github/workflows/ci.yml`
- **Required:** Uncomment `windows-latest` in the `matrix.os` array.
- **Gate:** All tasks in 6.1-6.3 (Category A and MUST items) must pass first.

## 7. CI Gate (Phase 2)

A release candidate MUST pass:

- All Phase 1 scenario tests (regression gate) on Linux, macOS, and Windows.
- All Phase 2 reactor and registry scenario tests on Linux, macOS, and Windows.
- At least one Docker adapter smoke test on Linux (may require Docker-in-Docker
  or be marked as manual in CI). Docker tests are NOT required on Windows CI.
- Schema extension validation tests for new sections and error codes.
- Cross-platform scenarios (Section 5) on all three OS platforms.
- Phase 1 Windows prerequisite tasks (Section 6) completed and verified.
