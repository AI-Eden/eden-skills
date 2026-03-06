# SPEC_TEST_MATRIX.md

Phase 2.95 acceptance test scenarios.

## 1. Convention

- Scenario IDs: `TM-P295-001` to `TM-P295-048`.
- Tests marked `auto` are implemented as Rust integration tests.
- Tests marked `script` are shell/PowerShell script tests.
- Tests marked `manual` require manual verification.

## 2. Install Scripts (WP-5)

| ID | Scenario | Spec | Type | Status |
| :--- | :--- | :--- | :--- | :--- |
| TM-P295-001 | `install.sh` detects Linux x86_64 and downloads correct archive | ISC-001 | script | pending |
| TM-P295-002 | `install.sh` detects macOS arm64 and downloads correct archive | ISC-001 | script | pending |
| TM-P295-003 | `install.sh` aborts on unsupported platform with clear error | ISC-003 | script | pending |
| TM-P295-004 | `install.sh` aborts on SHA-256 mismatch | ISC-004 | script | pending |
| TM-P295-005 | `install.sh` updates the selected shell rc file when dir not in PATH (without duplicate PATH entries) | ISC-005 | script | pending |
| TM-P295-006 | `install.ps1` detects Windows x86_64 and installs binary | ISC-002 | script | pending |
| TM-P295-007 | `install.ps1` aborts on SHA-256 mismatch | ISC-004 | script | pending |
| TM-P295-008 | `EDEN_SKILLS_VERSION` env var pins install to specific version | ISC-007 | script | pending |
| TM-P295-009 | `cargo binstall eden-skills` resolves correct download URL | ISC-006 | manual | pending |

## 3. Remove All (WP-2)

| ID | Scenario | Spec | Type | Status |
| :--- | :--- | :--- | :--- | :--- |
| TM-P295-010 | Interactive input `*` returns all skill IDs | RMA-001 | auto | pending |
| TM-P295-011 | Input `* 2` produces mixed-token error | RMA-002 | auto | pending |
| TM-P295-012 | Wildcard triggers strengthened confirmation with `[y/N]` default | RMA-003 | auto | pending |
| TM-P295-013 | Prompt text includes `* for all` hint | RMA-004 | auto | pending |
| TM-P295-014 | Wildcard + `-y` flag skips confirmation and removes all | RMA-003 | auto | pending |
| TM-P295-015 | Wildcard confirmation declined (`N`) cancels removal | RMA-003 | auto | pending |

## 4. Windows Junction (WP-3)

| ID | Scenario | Spec | Type | Status |
| :--- | :--- | :--- | :--- | :--- |
| TM-P295-016 | Windows symlink available: uses symlink (no junction) | WJN-001 | auto | pending |
| TM-P295-017 | Windows symlink unavailable: falls back to junction | WJN-001 | auto | pending |
| TM-P295-018 | Windows symlink and junction unavailable: falls back to copy | WJN-001 | auto | pending |
| TM-P295-019 | Junction install recorded as `mode = "symlink"` in lock | WJN-003 | auto | pending |
| TM-P295-020 | `plan` detects junction as valid symlink-mode target (not conflict) | WJN-004 | auto | pending |
| TM-P295-021 | Junction removal succeeds before reinstall | WJN-005 | auto | pending |
| TM-P295-022 | Junction probe creates and cleans up temp junction | WJN-006 | auto | pending |
| TM-P295-023 | `junction` crate compiles on all CI platforms (no-op on non-Windows) | WJN-002 | auto | pending |

## 5. Performance Sync (WP-1)

| ID | Scenario | Spec | Type | Status |
| :--- | :--- | :--- | :--- | :--- |
| TM-P295-024 | Repo cache directory `.repos/` created on first sync | PSY-001 | auto | pending |
| TM-P295-025 | Two skills from same repo+ref produce one clone, one cache dir | PSY-001 | auto | pending |
| TM-P295-026 | Two skills from same repo but different refs produce two cache dirs | PSY-001 | auto | pending |
| TM-P295-027 | URL normalization: HTTPS, SSH, trailing `.git` produce same key | PSY-002 | auto | pending |
| TM-P295-028 | Ref sanitization: `main`, `v2.0`, `refs/heads/main` produce valid keys | PSY-002 | auto | pending |
| TM-P295-029 | Discovery clone moved to cache location (no second clone) | PSY-003 | auto | pending |
| TM-P295-030 | Cross-filesystem rename fallback: fresh clone in cache on rename failure | PSY-003 | auto | pending |
| TM-P295-031 | Install batches all selected skills into one `sync_sources_async` call | PSY-004 | auto | pending |
| TM-P295-032 | `apply` with unchanged lock entries skips fetch for those repos | PSY-005 | auto | pending |
| TM-P295-033 | `repair` always fetches all repos (no skip optimization) | PSY-005 | auto | pending |
| TM-P295-034 | `update` Mode A refresh uses repo cache paths | PSY-006 | auto | pending |
| TM-P295-035 | Old per-skill directories do not prevent new cache-based installs | PSY-007 | auto | pending |
| TM-P295-036 | `plan` and `doctor` resolve source from repo cache | PSY-006 | auto | pending |
| TM-P295-037 | Copy-mode mtime+size fast path avoids byte comparison when matching | PSY-008 | auto | pending |
| TM-P295-038 | Local source installs unchanged (no repo cache) | PSY-001 | auto | pending |

## 6. Docker Bind Mount (WP-4)

| ID | Scenario | Spec | Type | Status |
| :--- | :--- | :--- | :--- | :--- |
| TM-P295-039 | Bind mount detected: install creates host-side symlink, no docker cp | DBM-002 | auto | pending |
| TM-P295-040 | No bind mount: install falls back to docker cp | DBM-001 | auto | pending |
| TM-P295-041 | `docker mount-hint` outputs recommended `-v` flags | DBM-003 | auto | pending |
| TM-P295-042 | `docker mount-hint` reports "already mounted" when all paths covered | DBM-003 | auto | pending |
| TM-P295-043 | `doctor` reports `DOCKER_NO_BIND_MOUNT` for unmounted targets | DBM-004 | auto | pending |
| TM-P295-044 | Install completion shows bind-mount hint after docker cp | DBM-005 | auto | pending |
| TM-P295-045 | Bind mount uninstall removes on host, not via docker exec | DBM-002 | auto | pending |
| TM-P295-046 | `--target docker:my-container` detects multiple agents inside container | DBM-007 | auto | pending |
| TM-P295-047 | `--target docker:my-container` with no agents in container falls back to ClaudeCode | DBM-007 | auto | pending |
| TM-P295-048 | Existing manual docker targets in skills.toml are not affected by auto-detection | DBM-007 | auto | pending |

## 7. Regression

Regression tests are not individually numbered. The following MUST
pass after all Phase 2.95 changes:

- `cargo fmt --all -- --check` MUST pass.
- `cargo clippy --workspace -- -D warnings` MUST pass.
- `cargo test --workspace` — all existing Phase 1/2/2.5/2.7/2.8/2.9
  tests MUST continue to pass.
- For any batch that touches `cfg(windows)` code or Windows-only
  dependencies, `cargo check --workspace --all-targets --target
  x86_64-pc-windows-msvc` MUST pass when that target is installed.
- All `--json` output contracts MUST remain unchanged.
- Exit codes 0/1/2/3 MUST retain their semantics.
