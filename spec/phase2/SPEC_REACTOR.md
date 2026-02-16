# SPEC_REACTOR.md

Normative specification for the Phase 2 async concurrency model.

## 1. Purpose

Define the runtime concurrency architecture that replaces Phase 1's serial
`apply` loop with a parallel, bounded-concurrency task reactor based on `tokio`.

## 2. Scope

- Async runtime selection and integration.
- Bounded-concurrency task execution for network I/O.
- Serialization policy for disk I/O operations.

## 3. Non-Goals

- Dependency resolution between skills (DAG scaffolded but not enforced).
- CPU-bound parallelism (use `tokio::spawn_blocking` if needed, not `rayon`).

## 4. Normative Requirements

| ID | Owner | Priority | Statement | Verification |
| :--- | :--- | :--- | :--- | :--- |
| **ARC-001** | Builder | **P0** | The CLI MUST utilize a `tokio` runtime for all network I/O operations. | `cargo tree` shows `tokio`; network calls use `.await`. |
| **ARC-002** | Builder | **P0** | Skill downloads/updates MUST be executed in parallel with a bounded concurrency limit (default: 10). | Benchmarking 50 skills install time < serial install time / 5. |
| **ARC-003** | Builder | **P1** | Disk I/O (symlinking/copying) SHOULD be serialized per target path to avoid race conditions. | No "file locked" or "race" errors during high-concurrency stress test. |

## 5. Architecture Decision

### ADR-002: Async Runtime Selection

- **Context:** Need parallelism for Git clone/fetch operations. Phase 3 Crawler
  will be heavily I/O bound.
- **Options:**
    1. `std::thread` -- OS native threads.
    2. `tokio` -- Async runtime with green threads.
    3. `rayon` -- Data parallelism (CPU-bound focus).
- **Decision:** **Option 2 (tokio)**.
- **Rationale:** Investing in `tokio` now prepares the foundation for
  high-concurrency crawling in Phase 3. Network I/O is the bottleneck,
  not CPU computation.
- **Trade-off:** Adds async complexity to the codebase. All I/O call sites
  must be converted to async.
- **Rollback Trigger:** If async conversion proves too invasive for Phase 1
  backward compatibility, fall back to `std::thread` with a thread pool.

## 6. Data Model

### SkillReactor Component

```text
SkillReactor
├── parse skills.toml
├── build task list (future: DAG)
├── spawn N workers (bounded semaphore, default 10)
├── fetch/update sources in parallel (.await)
└── execute install/link steps (serialized per target path)
```

## 7. Failure Semantics

- **Network Failure (partial):** Reactor MUST continue processing remaining skills
  after a single skill download fails. Failed downloads are reported at end.
- **All Downloads Failed:** Exit code `1` with per-skill error diagnostics.
- **Semaphore Exhaustion:** MUST NOT happen; bounded by configuration.

## 8. Acceptance Criteria

1. `eden-skills apply` with 20 cached skills completes in < 2 seconds.
2. `eden-skills apply` with uncached skills saturates available network bandwidth.
3. Phase 1 integration tests pass without modification under the new runtime.
4. No race conditions observed in 100 repeated concurrent runs.
