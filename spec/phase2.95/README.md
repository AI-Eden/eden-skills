# Phase 2.95 Specifications: Performance, Platform Reach & UX Completeness

**Status:** DRAFT
**Parent:** `spec/README.md`
**Planned by:** Architect (Claude Opus), 2026-03-06

## Purpose

Phase 2.95 addresses five quality and reach gaps identified after Phase 2.9:

1. **Install sync performance** — eliminate redundant Git clones by
   introducing a repo-level cache, reusing discovery clones, and
   running batch-parallel source sync.
2. **Remove "all" shortcut** — add a `*` wildcard to the interactive
   remove selection prompt, with strengthened double confirmation.
3. **Windows junction fallback** — on Windows systems without symlink
   privileges, fall back to NTFS junction points before resorting to
   hard copy, providing near-zero-overhead linking without admin rights.
4. **Docker bind-mount strategy** — detect existing bind mounts in
   Docker targets and install via host-side symlink instead of
   `docker cp`, with a `docker mount-hint` subcommand and `doctor`
   diagnostics for unconfigured targets.
5. **Cross-platform install scripts** — provide one-liner `curl | bash`
   (Linux/macOS) and `irm | iex` (Windows) installers that download
   pre-built binaries from GitHub Releases, with `cargo-binstall`
   metadata support.

## Relationship to Earlier Phases

- Phase 1/2/2.5/2.7/2.8/2.9 specs are frozen.
- Phase 2.95 specs in this directory:
  1. **Refactor** the Phase 2 source sync model (`source.rs`) from
     per-skill cloning to repo-level caching with deduplication.
  2. **Extend** the Phase 2.7 `remove` interactive mode with a `*`
     wildcard token.
  3. **Extend** the Phase 2 `LocalAdapter` and install mode decision
     with NTFS junction point support on Windows.
  4. **Extend** the Phase 2 `DockerAdapter` with bind-mount detection
     and a new `docker mount-hint` subcommand.
  5. **Add** cross-platform install scripts and `cargo-binstall` metadata
     (new files, no existing spec dependency).

## Scope Exclusions

- No Phase 3 features (crawler, taxonomy, curation).
- No changes to `--json` output schemas for existing commands.
- No changes to exit code semantics (0/1/2/3).
- No changes to `skills.toml` format.
- `skills.lock` format is unchanged; field values may reference new
  repo-cache paths.
- No changes to existing CLI command names or flag semantics
  (except additive: new `docker mount-hint` subcommand).

## Work Packages

| WP | Priority | Spec File | Domain | Description |
| :--- | :--- | :--- | :--- | :--- |
| WP-1 | **P0** | `SPEC_PERF_SYNC.md` | Core + CLI | Repo-level cache, discovery reuse, batch sync, update/apply/repair migration |
| WP-2 | **P2** | `SPEC_REMOVE_ALL.md` | CLI | `*` wildcard in interactive remove, strengthened confirmation |
| WP-3 | **P1** | `SPEC_WINDOWS_JUNCTION.md` | Core | NTFS junction fallback chain, `junction` crate integration |
| WP-4 | **P1** | `SPEC_DOCKER_BIND.md` | Core + CLI | Container agent auto-detection, bind-mount detection, `docker mount-hint`, doctor check |
| WP-5 | **P0** | `SPEC_INSTALL_SCRIPT.md` | Distribution | `install.sh`, `install.ps1`, `cargo-binstall` metadata |
| -- | -- | `SPEC_TEST_MATRIX.md` | Testing | Phase 2.95 acceptance test scenarios |
| -- | -- | `SPEC_TRACEABILITY.md` | Traceability | Requirement-to-implementation mapping |

## Requirement ID Ranges

| Domain | ID Range |
| :--- | :--- |
| Performance Sync | PSY-001 ~ PSY-008 |
| Remove All | RMA-001 ~ RMA-004 |
| Windows Junction | WJN-001 ~ WJN-006 |
| Docker Bind Mount | DBM-001 ~ DBM-007 |
| Install Script | ISC-001 ~ ISC-007 |
| Test Scenarios | TM-P295-001 ~ TM-P295-048 |

## Execution Order

```text
B1 (Install Scripts / WP-5) ──────────────────────────┐
B2 (Remove All / WP-2) ───────────────────────────────┤
B3 (Windows Junction / WP-3) ─────────────────────────┤
B4 (Perf Part 1 / WP-1 core) ──→ B5 (Perf Part 2) ──┤──→ B7 (Regression)
B6 (Docker Bind Mount / WP-4) ────────────────────────┘
```

B1, B2, B3, B4, and B6 are independent of each other.
B5 depends on B4 (repo-level cache infrastructure).
B7 depends on all preceding batches.

## New CLI Elements

| Element | Type | Description |
| :--- | :--- | :--- |
| `docker mount-hint` | Subcommand | Output recommended `-v` flags for a Docker container |

## New Dependencies

| Crate | Version | Platform | Purpose |
| :--- | :--- | :--- | :--- |
| `junction` | `1` | `cfg(windows)` | Create/detect/remove NTFS junction points |

## Normative Language

Same as `spec/README.md`: `MUST`, `SHOULD`, `MAY` per RFC 2119.
