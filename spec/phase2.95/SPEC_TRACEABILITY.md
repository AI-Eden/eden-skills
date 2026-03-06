# SPEC_TRACEABILITY.md

Requirement-to-implementation mapping for Phase 2.95.
Use this file to recover accurate context after compression.

**Status:** PENDING — Populated by Builder during implementation.

## 1. Performance Sync Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
| --- | --- | --- | --- | --- | --- |
| PSY-001 | `SPEC_PERF_SYNC.md` 2.1 | Source sync MUST use repo-level cache at `.repos/` | | | pending |
| PSY-002 | `SPEC_PERF_SYNC.md` 2.2 | Cache key from normalized URL + sanitized ref | | | pending |
| PSY-003 | `SPEC_PERF_SYNC.md` 3.2 | Discovery clone MUST be reused via move | | | pending |
| PSY-004 | `SPEC_PERF_SYNC.md` 4.2 | Install sync MUST batch into one reactor call | | | pending |
| PSY-005 | `SPEC_PERF_SYNC.md` 5.2 | Apply SHOULD skip sync for unchanged repos | | | pending |
| PSY-006 | `SPEC_PERF_SYNC.md` 7 | update/apply/repair MUST use repo cache | | | pending |
| PSY-007 | `SPEC_PERF_SYNC.md` 6.2 | Migration MUST be gradual and non-destructive | | | pending |
| PSY-008 | `SPEC_PERF_SYNC.md` 8 | Copy-mode mtime+size fast path | | | pending |

## 2. Remove All Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
| --- | --- | --- | --- | --- | --- |
| RMA-001 | `SPEC_REMOVE_ALL.md` 2.1 | `*` wildcard returns all skill IDs | | | pending |
| RMA-002 | `SPEC_REMOVE_ALL.md` 2.2 | `*` combined with other tokens MUST error | | | pending |
| RMA-003 | `SPEC_REMOVE_ALL.md` 3 | Wildcard triggers strengthened confirmation | | | pending |
| RMA-004 | `SPEC_REMOVE_ALL.md` 2.3 | Prompt includes `* for all` hint | | | pending |

## 3. Windows Junction Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
| --- | --- | --- | --- | --- | --- |
| WJN-001 | `SPEC_WINDOWS_JUNCTION.md` 2 | Three-level fallback: symlink → junction → copy | | | pending |
| WJN-002 | `SPEC_WINDOWS_JUNCTION.md` 5 | `junction` crate as `cfg(windows)` dependency | | | pending |
| WJN-003 | `SPEC_WINDOWS_JUNCTION.md` 3.1 | Junction NOT exposed as new InstallMode | | | pending |
| WJN-004 | `SPEC_WINDOWS_JUNCTION.md` 4 | plan.rs detects junction reparse points | | | pending |
| WJN-005 | `SPEC_WINDOWS_JUNCTION.md` 3.2–3.3 | Adapter handles junction create/remove | | | pending |
| WJN-006 | `SPEC_WINDOWS_JUNCTION.md` 2.1 | Junction probe in install mode decision | | | pending |

## 4. Docker Bind Mount Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
| --- | --- | --- | --- | --- | --- |
| DBM-001 | `SPEC_DOCKER_BIND.md` 2 | Bind mount detection via docker inspect | | | pending |
| DBM-002 | `SPEC_DOCKER_BIND.md` 2.3 | Bind mount → host-side symlink | | | pending |
| DBM-003 | `SPEC_DOCKER_BIND.md` 3 | `docker mount-hint` subcommand | | | pending |
| DBM-004 | `SPEC_DOCKER_BIND.md` 4 | Doctor reports DOCKER_NO_BIND_MOUNT | | | pending |
| DBM-005 | `SPEC_DOCKER_BIND.md` 5 | Install completion bind-mount hint | | | pending |
| DBM-006 | `SPEC_DOCKER_BIND.md` 4 | docs/04-docker-targets.md updated | | | pending |
| DBM-007 | `SPEC_DOCKER_BIND.md` 2 | `--target docker:` auto-detects agents in container | | | pending |

## 5. Install Script Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
| --- | --- | --- | --- | --- | --- |
| ISC-001 | `SPEC_INSTALL_SCRIPT.md` 2.1 | install.sh for Linux/macOS | `install.sh`, `README.md`, `docs/01-quickstart.md` | `crates/eden-skills-cli/tests/install_script_tests.rs` (TM-P295-001, TM-P295-002) | completed |
| ISC-002 | `SPEC_INSTALL_SCRIPT.md` 2.2 | install.ps1 for Windows | `install.ps1`, `README.md`, `docs/01-quickstart.md` | `crates/eden-skills-cli/tests/install_script_tests.rs` (TM-P295-006) | completed |
| ISC-003 | `SPEC_INSTALL_SCRIPT.md` 2.1 | Platform detection and triple mapping | `install.sh`, `install.ps1` | `crates/eden-skills-cli/tests/install_script_tests.rs` (TM-P295-001, TM-P295-002, TM-P295-003, TM-P295-006) | completed |
| ISC-004 | `SPEC_INSTALL_SCRIPT.md` 2.1 | SHA-256 integrity verification | `install.sh`, `install.ps1` | `crates/eden-skills-cli/tests/install_script_tests.rs` (TM-P295-004, TM-P295-007) | completed |
| ISC-005 | `SPEC_INSTALL_SCRIPT.md` 2.1 | PATH check and shell-specific hint | `install.sh`, `install.ps1` | `crates/eden-skills-cli/tests/install_script_tests.rs` (TM-P295-005, TM-P295-006) | completed |
| ISC-006 | `SPEC_INSTALL_SCRIPT.md` 3 | cargo-binstall metadata | `crates/eden-skills-cli/Cargo.toml` | `crates/eden-skills-cli/tests/install_script_tests.rs` (TM-P295-009) | completed |
| ISC-007 | `SPEC_INSTALL_SCRIPT.md` 2.1 | EDEN_SKILLS_VERSION version pinning | `install.sh`, `install.ps1` | `crates/eden-skills-cli/tests/install_script_tests.rs` (TM-P295-006, TM-P295-008) | completed |
