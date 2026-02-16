# Phase 2 Architecture Spec (Stage B): The "Hyper-Loop" Core Architecture

**Version:** 2.0 (Frozen)
**Parent:** `prompt/PHASE2-STAGE-A.md`
**Status:** CONTRACT_FROZEN
**Owner:** Architect (Claude)
**Enforcement:** Builder MUST comply with all NORMATIVE sections.

---

## 0. Guardrails (Non-Negotiable)

These rules are binding on all roles (Architect and Builder) throughout Phase 2 execution. Violation of any rule below invalidates the affected deliverable.

1. **AGENTS.md Compliance:** Read and follow `AGENTS.md` first, especially Read Order, Authority Order, Role Boundaries, and Guardrails.
2. **Authority Order:** When files conflict, resolution MUST follow: `spec/**/*.md` > `STATUS.yaml` > `EXECUTION_TRACKER.md` > `ROADMAP.md` > `README.md`.
3. **Responsibility Boundary:** Architect owns taxonomy, curation rubric, and crawler strategy. Builder owns implementation, tests, and refactors. Neither role may finalize the other's deliverables without explicit user instruction.
4. **Language Policy:** Talk to user in Chinese. All repository file content MUST be English-only.
5. **Phase Isolation:** Do not alter Phase 1 CLI behavior contracts (`spec/phase1/SPEC_COMMANDS.md`, `spec/phase1/SPEC_SCHEMA.md`, etc.). Phase 2 contracts must be isolated in `spec/phase2/` and MUST NOT inject semantics into existing Phase 1 normative sections.
6. **No-Stop Constraint:** Do not stop at analysis or recommendation-only output. Deliverables must be directly created or updated in the repository in the same turn.

---

## 1. Purpose

To evolve `eden-skills` from a local file-linker (Phase 1) to a high-performance, environment-agnostic package manager capable of supporting the "Double-Track" registry system.

## 2. Scope

* **In Scope:**
  * Refactoring core runtime to Async/Await (`tokio`).
  * Implementation of `TargetAdapter` abstraction (Local + Docker).
  * Implementation of `Registry` resolution logic (Official + Forge).
* **Out of Scope:**
  * The Crawler Implementation (Moved to Phase 3).
  * Web UI / Search Interface.

* **Non-Goals:**
  * Real-time file synchronization for Docker targets. `docker cp` is a point-in-time snapshot; live symlink behavior inside containers is explicitly not pursued (see ADR-001).
  * Dependency resolution between skills. The DAG structure is scaffolded for future use but MUST NOT block Phase 2 delivery.
  * `SshAdapter` or any remote-server adapter beyond Docker. Deferred to Phase 3+.
  * Backward-incompatible `skills.toml` schema migration. Phase 1 `source = { repo, ref }` syntax MUST continue to work.
  * Automatic agent environment discovery. Users must explicitly specify target environments in configuration.

## 3. Normative Requirements (Norms)

### 3.1 Concurrency Model (The Reactor)

| ID | Priority | Statement | Verification |
| :--- | :--- | :--- | :--- |
| **ARC-001** | **P0** | The CLI MUST utilize a `tokio` runtime for all network I/O operations. | `cargo tree` shows `tokio`; Network calls are `.await`. |
| **ARC-002** | **P0** | Skill downloads/updates MUST be executed in parallel with a bounded concurrency limit (default: 10). | Benchmarking 50 skills install time < Serial install time / 5. |
| **ARC-003** | **P1** | Disk I/O (Symlinking/Copying) SHOULD be serialized per target path to avoid race conditions. | No "file locked" or "race" errors during high-concurrency stress test. |

### 3.2 Environment Abstraction (The Adapter)

| ID | Priority | Statement | Verification |
| :--- | :--- | :--- | :--- |
| **ARC-101** | **P0** | The system MUST define a `TargetAdapter` trait decoupling intent from syscalls. | Code contains `trait TargetAdapter { async fn inject(...) }`. |
| **ARC-102** | **P0** | A `LocalAdapter` implementation MUST be provided for backward compatibility. | Phase 1 integration tests pass without modification. |
| **ARC-103** | **P0** | A `DockerAdapter` implementation MUST be provided using `docker cli` or API. | Can install skill to a running container without volume mount. |
| **ARC-104** | **P1** | The `DockerAdapter` MUST support `cp` (copy) injection strategy as the primary reliable method. | File appears in container after `eden-skills install` even without shared volumes. |

### 3.3 Registry Resolution (The Resolver)

| ID | Priority | Statement | Verification |
| :--- | :--- | :--- | :--- |
| **ARC-201** | **P0** | Configuration MUST support multiple registry sources with priority weights. | `config.toml` accepts `[registries]` table. |
| **ARC-202** | **P0** | Resolution logic MUST follow `Official -> Forge -> Failure` fallback order by default. | Installing a skill present in both prefers the Official version. |
| **ARC-203** | **P1** | Registry indexes MUST be local Git repositories synchronized via `eden-skills update`. | `~/.eden-skills/registries/` contains cloned index repos. |

---

## 4. Architecture Decision Records (ADR)

### ADR-001: Docker Injection Strategy

* **Context:** Agents in Docker containers need access to skills. Symlinks on host do not work inside containers without volume mapping.
* **Options:**
    1. *Dynamic Volume Mount:* Restart container with `-v`.
    2. *Docker CP:* Copy files via API/CLI.
    3. *SSH/SCP:* Treat container as remote server.
* **Decision:** **Option 2 (Docker CP)**.
* **Rationale:** Least intrusive. Does not require restarting the container (which kills Agent state). Matches "Injection" paradigm.
* **Trade-off:** Updates are not real-time (no symlink magic). `eden-skills update` must re-copy files.

### ADR-002: Async Runtime Selection

* **Context:** Need parallelism for Git operations.
* **Options:**
    1. *std::thread:* OS native threads.
    2. *tokio:* Async runtime.
    3. *rayon:* Data parallelism.
* **Decision:** **Option 2 (tokio)**.
* **Rationale:** Phase 3 (Crawler) will be heavily I/O bound. Investing in `tokio` now prepares the foundation for high-concurrency crawling later.

---

## 5. Data Model (Frozen)

### 5.1 Registry Index Structure

The "Database" is a file tree in a git repo:

```text
/index
  /a
    /agent-search.toml
  /b
    /browser-use.toml

```

### 5.2 Updated `skills.toml` Fragment

```toml
[registries]
official = { url = "...", priority = 100 }
forge    = { url = "...", priority = 10 }

[[skills]]
name = "browser-use"
version = "1.2.0"   # SemVer matching against Registry TOML
target = "docker:devin-container" # Triggers DockerAdapter

```

---

## 6. Traceability & Handoff Checklist

### Builder Entry Criteria (Freeze Gate)

Builder implementation can start only if **all** checks pass:

* [ ] **ARC-001** to **ARC-203** are understood and accepted.
* [ ] No breaking changes to Phase 1 `spec/` files (Backward Compatibility).
* [ ] All requirement IDs are unique across all Phase 2 spec files.
* [ ] Every **MUST** (P0) requirement has a verification entry.
* [ ] All new or modified file content is English-only.
* [ ] All open design questions are resolved or explicitly deferred with owner and due phase.

### Failure Semantics

* **Network Failure:** Partial success allowed. Failed downloads reported at end (Exit Code 0).
* **Injection Failure:** Critical. If `DockerAdapter` cannot connect, fail fast for that specific skill (Exit Code 1).

### Normative Requirement Format

Each requirement in Phase 2 spec files must include:

| Field | Description |
| :--- | :--- |
| **ID** | Unique identifier (`ARC-xxx` for architecture, `REG-xxx` for registry, `ADT-xxx` for adapter). |
| **Owner** | `Architect` \| `Builder` \| `Shared` |
| **Priority** | `P0` (MUST) \| `P1` (SHOULD) \| `P2` (MAY) |
| **Statement** | Normative requirement using RFC 2119 keywords. |
| **Verification** | One testable condition proving compliance. |

### Architecture Decision Discipline

Any architecture decision in Phase 2 spec files must follow ADR format:

| Field | Description |
| :--- | :--- |
| **Decision ID** | Unique identifier (e.g., `ADR-001`). |
| **Context** | Problem or constraint motivating the decision. |
| **Options** | At least 2 evaluated alternatives. |
| **Chosen Option** | The selected approach with rationale. |
| **Trade-offs** | Known downsides of the chosen option. |
| **Rollback Trigger** | Condition under which the decision should be revisited. |

### Acceptance Criteria

Phase 2 spec work is considered complete when all of the following hold:

1. Builder can understand implementation direction for every ARC requirement without guessing core intent.
2. No conflict exists between Phase 2 contracts and current Phase 1 specs (`spec/phase1/SPEC_COMMANDS.md`, `spec/phase1/SPEC_SCHEMA.md`, `spec/phase1/SPEC_AGENT_PATHS.md`).
3. Every P0 requirement is traceable to a concrete verification condition.
4. All written file content is English-only.
5. Phase 2 success criteria (from Stage A: Performance, Versatility, Ecosystem) are achievable from this spec without additional architectural decisions.

---

## 7. Open Questions

Items below are unresolved or deferred. Each must be closed (with decision or explicit deferral) before the relevant implementation milestone begins.

| ID | Question | Owner | Due Phase | Status |
| :--- | :--- | :--- | :--- | :--- |
| **OQ-001** | What SemVer matching strategy should registry resolution use (exact, range `^`, prefix `~`)? | Architect | Phase 2 | Open |
| **OQ-002** | What fields are required vs. optional in each registry index TOML entry (e.g., `description`, `license`, `min_eden_skills_version`)? | Architect | Phase 2 | Open |
| **OQ-003** | Should the concurrency limit (default: 10) be configurable via `skills.toml`, CLI flag, or environment variable? | Shared | Phase 2 | Open |
| **OQ-004** | How should `DockerAdapter` handle permission errors inside the container (retry as root, fail, warn)? | Builder | Phase 2 | Open |
| **OQ-005** | What is the rollback strategy if `docker cp` partially fails mid-injection (some files copied, others not)? | Shared | Phase 2 | Open |
