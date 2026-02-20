# Phase 2.5 Specifications: MVP Launch

**Status:** DRAFT (Pending Builder Implementation)
**Parent:** `spec/README.md`
**Planned by:** Architect (Claude Opus), 2026-02-20

## Purpose

Phase 2.5 bridges the gap between the fully implemented Phase 2 architecture
and a usable MVP product. It adds no Phase 3 features (crawler, taxonomy,
curation) but instead focuses on the user-facing installation experience:

> Given a repository URL, install the skill in one command.

## Relationship to Phase 1 and Phase 2

- Phase 1 specs (`spec/phase1/`) are frozen and define the CLI foundation.
- Phase 2 specs (`spec/phase2/`) are frozen and define async reactor, adapter,
  and registry architecture.
- Phase 2.5 specs in this directory:
  1. **Amend** one Phase 1 validation rule (empty skills array) via `_P25` extension.
  2. **Introduce new domains** (URL install, agent detection, CLI UX, distribution).
  3. **Do NOT modify** any Phase 1 or Phase 2 command semantics.

## Work Streams

| WS | Priority | Spec File | Domain | Description |
| :--- | :--- | :--- | :--- | :--- |
| WS-1 + WS-2 | **P0** | `SPEC_SCHEMA_P25.md` | Config | Relax empty skills constraint; simplify init template |
| WS-3 | **P0** | `SPEC_INSTALL_URL.md` | CLI | Install from URL with source format parsing, SKILL.md discovery, interactive flow |
| WS-4 | **P1** | `SPEC_AGENT_DETECT.md` | CLI | Agent auto-detection for install targets |
| WS-5 | **P1** | `SPEC_DISTRIBUTION.md` | Infra | Binary distribution via GitHub Releases and `cargo install` |
| WS-7 | **P1** | `SPEC_CLI_UX.md` | CLI | CLI output beautification with colors, spinners, and symbols |
| -- | -- | `SPEC_TEST_MATRIX.md` | Testing | Phase 2.5 acceptance test scenarios |
| -- | -- | `SPEC_TRACEABILITY.md` | Traceability | Requirement-to-implementation mapping |

WS-6 (seed registry) is optional and does not require a normative spec.

## Requirement ID Ranges

| Domain | ID Range |
| :--- | :--- |
| Install URL | MVP-001 ~ MVP-015 |
| Schema Amendment | SCH-P25-001 ~ SCH-P25-003 |
| Agent Detection | AGT-001 ~ AGT-004 |
| CLI UX | UX-001 ~ UX-007 |
| Distribution | DST-001 ~ DST-003 |
| Test Scenarios | TM-P25-001 ~ TM-P25-028 |

## Dependency Graph

```text
WS-1/WS-2 (Schema + Init) ──→ WS-3 (Install from URL) ──→ WS-4 (Agent Detection)
                                        │                         │
                                        └──→ WS-7 (CLI UX) ←─────┘

WS-5 (Distribution) ←── independent, may run in parallel
```

## Normative Language

Same as `spec/README.md`: `MUST`, `SHOULD`, `MAY` per RFC 2119.
