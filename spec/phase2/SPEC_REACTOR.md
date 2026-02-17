# SPEC_REACTOR.md

Normative specification for the Phase 2 async concurrency model.

## 1. Purpose

Define the runtime concurrency architecture that replaces Phase 1's serial
`apply` loop with a parallel, bounded-concurrency task reactor based on `tokio`.

## 2. Scope

- Async runtime selection and integration.
- Bounded-concurrency task execution for network I/O.
- Serialization policy for disk I/O operations.
- Sync-to-async migration strategy for existing git operations.

## 3. Non-Goals

- Dependency resolution between skills (DAG scaffolded but not enforced).
- CPU-bound parallelism (use `tokio::spawn_blocking` if needed, not `rayon`).
- Plugin-based task types (Reactor handles download + install only).

## 4. Normative Requirements

| ID | Owner | Priority | Statement | Verification |
| :--- | :--- | :--- | :--- | :--- |
| **ARC-001** | Builder | **P0** | The CLI MUST utilize a `tokio` runtime for all network I/O operations. | `cargo tree` shows `tokio`; network calls use `.await`. |
| **ARC-002** | Builder | **P0** | Skill downloads/updates MUST be executed in parallel with a bounded concurrency limit (default: 10). | Benchmarking 50 skills install time < serial install time / 5. |
| **ARC-003** | Builder | **P1** | Disk I/O (symlinking/copying) SHOULD be serialized per target path to avoid race conditions. | No "file locked" or "race" errors during high-concurrency stress test. |
| **ARC-004** | Builder | **P1** | The concurrency limit SHOULD be configurable via `[reactor]` config section and overridable by `--concurrency` CLI flag. | Setting concurrency to `1` produces serial behavior; setting to `50` allows 50 parallel downloads. |
| **ARC-005** | Builder | **P0** | The Reactor MUST implement two-phase execution: Phase A (parallel source sync) completes before Phase B (serialized install mutations) begins. | No install mutation starts while any download is still in progress. |
| **ARC-006** | Builder | **P0** | Synchronous git operations (clone/fetch/checkout) MUST be executed via `tokio::spawn_blocking` or async process invocation to avoid blocking the async runtime. | No tokio "blocking the runtime" warnings during concurrent operations. |

## 5. Architecture Decisions

### ADR-002: Async Runtime Selection

- **Context:** Need parallelism for Git clone/fetch operations. Phase 3 Crawler
  will be heavily I/O bound.
- **Options:**
    1. **`std::thread` + thread pool:** OS native threads with manual coordination.
       - Pros: No async complexity; familiar model; no colored function problem.
       - Cons: OS threads are heavy (~8 KB stack each); manual join/cancel;
         no ecosystem for structured concurrency; poor fit for Phase 3 crawler.
    2. **`tokio`:** Async runtime with green threads and ecosystem.
       - Pros: Lightweight tasks; built-in `JoinSet`, `Semaphore`, `select!`;
         `tokio::process` for async subprocesses; dominant Rust async ecosystem;
         prepares for Phase 3 high-concurrency crawler.
       - Cons: Async complexity infects the codebase; `main()` becomes
         `#[tokio::main]`; all I/O call sites need conversion.
    3. **`rayon`:** Data parallelism (CPU-bound focus).
       - Pros: Simple `par_iter()` API; great for CPU-bound work.
       - Cons: Designed for CPU parallelism, not I/O; no async support;
         poor fit for network-bound workload.
- **Decision:** **Option 2 (tokio)**.
- **Rationale:** Investing in `tokio` now prepares the foundation for
  high-concurrency crawling in Phase 3. Network I/O is the bottleneck,
  not CPU computation. The `tokio` ecosystem provides `JoinSet`, `Semaphore`,
  and `tokio::process` that align with our task coordination needs.
- **Trade-off:** Adds async complexity to the codebase. All I/O call sites
  must be converted to async. `main()` becomes `#[tokio::main]`.
- **Rollback Trigger:** If async conversion proves too invasive for Phase 1
  backward compatibility, fall back to `std::thread` with a thread pool.

### ADR-003: Task Coordination Strategy

- **Context:** The Reactor must coordinate N parallel downloads with bounded
  concurrency. Need a pattern that handles partial failures gracefully and
  collects per-skill results for deterministic reporting.
- **Options:**
    1. **Stream pipeline (`futures::stream::iter` + `buffer_unordered`):**
       Create a stream of async closures, use `buffer_unordered(N)` for
       bounded concurrency, collect results via `.collect()`.
       - Pros: Concise (3-5 lines); idiomatic; built-in backpressure;
         natural for homogeneous task pipelines.
       - Cons: Homogeneous task types only; cancellation requires external
         signaling; error short-circuiting needs `try_buffer_unordered` or
         manual handling; no per-task lifecycle control.
    2. **Semaphore-bounded spawn (`tokio::JoinSet` + `Arc<Semaphore>`):**
       Spawn each skill as an independent tokio task. Use a shared semaphore
       to cap in-flight tasks. Collect results via `JoinSet::join_next()`.
       - Pros: Heterogeneous tasks possible; each task has independent
         lifetime; `JoinSet` provides structured concurrency and automatic
         cancellation on drop; clean per-task error isolation.
       - Cons: More boilerplate (~15-20 lines); semaphore must be correctly
         sized; developers must acquire/release permits consistently.
    3. **Channel-based worker pool (`mpsc` producer-consumer):**
       Create N worker tasks reading from a bounded channel. Main task sends
       skill descriptors. Workers pull and execute.
       - Pros: Classic pattern; clear separation of dispatch and execution;
         natural load balancing across workers.
       - Cons: Significantly more complex (~40+ lines); overkill for our
         workload; harder to handle per-skill result routing back to the
         coordinator; channel sizing is another tuning parameter.
- **Decision:** **Option 2 (Semaphore-bounded spawn with JoinSet)**.
- **Rationale:** Our workload has two distinct phases (download, then install).
  `JoinSet` provides clean task lifecycle management with automatic cancellation
  on drop. The semaphore pattern naturally supports heterogeneous task types
  (useful when Reactor later handles registry sync alongside skill sync).
  Compared to stream pipeline, this gives us more explicit control over
  per-task error handling, progress reporting, and cancellation.
- **Trade-off:** More boilerplate than `buffer_unordered`. Developers must
  remember to acquire/release semaphore permits correctly. Mitigated by
  encapsulating the pattern in the `SkillReactor` struct.

### ADR-004: Sync-to-Async Migration Strategy

- **Context:** Phase 1 git operations are synchronous (`git2` crate or
  `std::process::Command`). The async Reactor cannot call blocking code
  directly on the tokio runtime without risking thread starvation.
- **Options:**
    1. **`tokio::spawn_blocking` wrappers:** Wrap each sync git call in
       `spawn_blocking`. The tokio runtime delegates to a dedicated blocking
       thread pool.
       - Pros: Minimal code change; works with any sync library (`git2`,
         `std::process::Command`); blocking pool default limit (512) far
         exceeds practical concurrency needs.
       - Cons: Not truly async under the hood; blocking thread pool is a
         shared resource; very high concurrency (500+) could exhaust it.
    2. **`tokio::process::Command` for git CLI:** Replace sync git calls
       with async process invocation of the `git` CLI binary.
       - Pros: Truly async; no blocking threads; leverages OS process
         scheduler; output is well-documented.
       - Cons: Requires `git` binary in PATH; output parsing needed;
         loses `git2` type safety (if currently used).
    3. **Hybrid:** Use `tokio::process::Command` for heavy ops (clone/fetch)
       and `spawn_blocking` + sync library for local-only ops (checkout, status).
       - Pros: Best of both worlds for different operation profiles.
       - Cons: Two code paths to maintain; inconsistent error types.
- **Decision:** **Option 1 (`tokio::spawn_blocking`) as primary strategy
  for Phase 2, with Option 2 as recommended migration path for Phase 3.**
- **Rationale:** `spawn_blocking` preserves Phase 1 code with minimal change.
  The blocking thread pool default (512) far exceeds our practical concurrency
  limits (10-50). For Phase 3's crawler (potentially hundreds of concurrent
  git operations), migration to async process commands is recommended but
  not required for Phase 2 scope.
- **Trade-off:** Not fully async under the hood. Acceptable for Phase 2
  where concurrency is bounded to tens of tasks, not hundreds.

## 6. Data Model

### SkillReactor Component

```text
SkillReactor
├── Phase A: Source Sync (parallel, bounded)
│   ├── acquire semaphore permit
│   ├── spawn_blocking(git clone/fetch/checkout)
│   ├── persist safety metadata
│   ├── release permit
│   └── collect Result<SkillSyncOutcome, SkillSyncError>
├── Phase A completion barrier (all downloads finish)
├── Phase B: Install Mutations (serialized per path)
│   ├── resolve plan (create/update/noop/conflict)
│   ├── execute via TargetAdapter (ARC-101)
│   └── run verification checks
└── Report: aggregate outcomes + diagnostics
```

### Configuration

```toml
# Optional section in skills.toml
[reactor]
concurrency = 10  # Default: 10. Range: 1-100.
```

CLI override: `eden-skills apply --concurrency 20`

Priority: CLI flag > config file > built-in default (10).

## 7. Failure Semantics

- **Network Failure (partial):** Reactor MUST continue processing remaining skills
  after a single skill download fails. Failed downloads are reported at end.
  This preserves Phase 1's "attempt every skill" contract (SPEC_COMMANDS.md 3.2.1).
- **All Downloads Failed:** Exit code `1` with per-skill error diagnostics.
- **Semaphore Exhaustion:** MUST NOT happen; bounded by configuration.
- **Phase Boundary:** If Phase A produces any failures, Phase B MUST still
  execute for successfully synced skills. Failed skills are excluded from
  Phase B and reported in the final summary.
- **Panic in spawn_blocking:** tokio propagates panics via `JoinError`.
  The Reactor MUST catch and convert to a per-skill error diagnostic,
  not crash the process.

## 8. Acceptance Criteria

1. `eden-skills apply` with 20 cached skills completes in < 2 seconds.
2. `eden-skills apply` with uncached skills saturates available network bandwidth.
3. Phase 1 integration tests pass without modification under the new runtime.
4. No race conditions observed in 100 repeated concurrent runs.
5. Setting `--concurrency 1` produces behavior equivalent to Phase 1 serial execution.

## 9. Freeze Candidates

Items requiring Stage B resolution before Builder implementation begins:

| ID | Item | Options Under Consideration | Resolution Needed |
| :--- | :--- | :--- | :--- |
| **FC-R1** | Default concurrency limit value | `10` (current spec) vs `num_cpus * 2` (architecture vision) vs `num_cpus` (conservative) | Pick one default and document rationale. |
| **FC-R2** | Concurrency config surface | `[reactor]` config section + CLI flag (both) vs CLI flag only vs config only | Decide which surfaces are P0 vs P1. |
| **FC-R3** | Phase boundary strictness | Strict barrier (all Phase A done before any Phase B) vs streaming (Phase B starts as each Phase A item completes) | Strict is simpler and safer; streaming is faster for large skill sets. |
| **FC-R4** | Progress reporting UX | Per-skill progress bar (indicatif) vs summary-only vs streaming log lines | Decide minimum viable UX for Phase 2. |
