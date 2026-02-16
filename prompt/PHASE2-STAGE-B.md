# Phase 2 Architecture Spec (Stage B): The "Hyper-Loop" Core Architecture

**Version:** 2.0 (Frozen)
**Parent:** `prompt/PHASE2-STAGE-A.md`
**Status:** CONTRACT_FROZEN
**Owner:** Architect (Claude)
**Enforcement:** Builder MUST comply with all NORMATIVE sections.

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
| **ARC-104** | **P1** | The `DockerAdapter` MUST support `cp` (copy) injection strategy as the primary reliable method. | File appears in container after `eden install` even without shared volumes. |

### 3.3 Registry Resolution (The Resolver)

| ID | Priority | Statement | Verification |
| :--- | :--- | :--- | :--- |
| **ARC-201** | **P0** | Configuration MUST support multiple registry sources with priority weights. | `config.toml` accepts `[registries]` table. |
| **ARC-202** | **P0** | Resolution logic MUST follow `Official -> Forge -> Failure` fallback order by default. | Installing a skill present in both prefers the Official version. |
| **ARC-203** | **P1** | Registry indexes MUST be local Git repositories synchronized via `eden update`. | `~/.eden/registries/` contains cloned index repos. |

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
* **Trade-off:** Updates are not real-time (no symlink magic). `eden update` must re-copy files.

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

### Builder Entry Criteria (Gate)

* [ ] **ARC-001** to **ARC-203** are understood.
* [ ] No breaking changes to Phase 1 `spec/` files (Backward Compatibility).

### Failure Semantics

* **Network Failure:** Partial success allowed. Failed downloads reported at end (Exit Code 0).
* **Injection Failure:** Critical. If `DockerAdapter` cannot connect, fail fast for that specific skill (Exit Code 1).
