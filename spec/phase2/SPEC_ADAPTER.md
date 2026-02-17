# SPEC_ADAPTER.md

Normative specification for the Phase 2 environment abstraction layer.

## 1. Purpose

Decouple the *intent* ("Install Skill X to Location Y") from the *execution*
("Syscall") by defining a `TargetAdapter` trait. This enables skill installation
into local filesystems, Docker containers, and future remote environments
without changing core logic.

## 2. Scope

- `TargetAdapter` trait definition.
- `LocalAdapter` implementation (Phase 1 backward compatibility).
- `DockerAdapter` implementation (Docker CP injection).
- Adapter selection and instantiation from config.

## 3. Non-Goals

- `SshAdapter` or any remote-server adapter beyond Docker (deferred to Phase 3+).
- Real-time file synchronization for Docker targets (`docker cp` is point-in-time).
- Automatic agent environment discovery (users must specify targets explicitly).
- Multi-container orchestration (single container per target entry).

## 4. Normative Requirements

| ID | Owner | Priority | Statement | Verification |
| :--- | :--- | :--- | :--- | :--- |
| **ARC-101** | Builder | **P0** | The system MUST define a `TargetAdapter` trait decoupling intent from syscalls. | Code contains `trait TargetAdapter` with async methods for health check, path existence, install, and exec. |
| **ARC-102** | Builder | **P0** | A `LocalAdapter` implementation MUST be provided for backward compatibility. | Phase 1 integration tests pass without modification when using `LocalAdapter`. |
| **ARC-103** | Builder | **P0** | A `DockerAdapter` implementation MUST be provided using `docker` CLI. | Can install a skill to a running container without volume mount. |
| **ARC-104** | Builder | **P1** | The `DockerAdapter` MUST support `cp` (copy) injection strategy as the primary reliable method. | File appears in container after `eden-skills install` even without shared volumes. |
| **ARC-105** | Builder | **P1** | The `DockerAdapter` MUST use `tokio::process::Command` for async Docker CLI interaction. | Docker operations do not block the tokio runtime. |
| **ARC-106** | Builder | **P0** | Adapter selection MUST be deterministic: the adapter type is derived solely from the `environment` field in config. | Same config always produces the same adapter instance. No runtime environment sniffing. |
| **ARC-107** | Builder | **P1** | The `TargetAdapter` trait SHOULD include an `uninstall` method for removing previously installed skill targets. | `eden-skills remove` can clean up installed targets across adapter types. |
| **ARC-108** | Builder | **P0** | The `TargetAdapter` trait MUST require `Send + Sync` bounds. This is mandatory for compatibility with `JoinSet::spawn` which requires `Send` on spawned futures. | `Box<dyn TargetAdapter>` compiles with `JoinSet::spawn`; no `Send` bound errors. |

## 5. Architecture Decisions

### ADR-001: Docker Injection Strategy

- **Context:** Agents in Docker containers need access to skills. Symlinks on
  host do not work inside containers without volume mapping.
- **Options:**
    1. **Dynamic Volume Mount:** Restart container with `-v host:container`.
       - Pros: Real-time sync; native filesystem access.
       - Cons: Requires container restart (kills Agent state); user must have
         Docker Compose or orchestrator access; security implications of
         host mounts.
    2. **Docker CP:** Copy files via `docker cp` CLI or API.
       - Pros: Least intrusive; no restart; works with any running container;
         matches "injection" paradigm.
       - Cons: Not real-time (point-in-time copy); updates require re-copy;
         large skill trees may be slow.
    3. **SSH/SCP:** Treat container as remote server with SSH daemon.
       - Pros: Standard remote file transfer; supports incremental sync (rsync).
       - Cons: Requires SSH daemon in container (security risk, extra config);
         not standard for Docker workflows.
- **Decision:** **Option 2 (Docker CP)**.
- **Rationale:** Least intrusive. Does not require restarting the container
  (which kills Agent state). Matches "Injection" paradigm. Simple to implement
  and test.
- **Trade-off:** Updates are not real-time (no symlink magic). `eden-skills update`
  must re-copy files after source changes.
- **Rollback Trigger:** If `docker cp` proves unreliable for large skill trees
  or permission-sensitive paths, consider SSH-based injection or volume mount
  with graceful restart.

### ADR-005: Docker Client Strategy

- **Context:** The DockerAdapter needs to interact with the Docker daemon for
  container inspection, file copying, and command execution. Two main approaches
  exist in the Rust ecosystem.
- **Options:**
    1. **`docker` CLI via `tokio::process::Command`:**
       Shell out to the `docker` binary for all operations (`docker cp`,
       `docker exec`, `docker inspect`).
       - Pros: Zero additional dependencies; works with any Docker installation
         (Docker Desktop, Docker CE, Podman with Docker compat); trivially
         async via `tokio::process`; well-documented output formats.
       - Cons: Requires `docker` binary in PATH; output parsing may be fragile
         for edge cases; spawning processes has per-invocation overhead.
    2. **`bollard` crate (Rust Docker API client):**
       Use the bollard library to communicate with Docker daemon via Unix
       socket or HTTP.
       - Pros: Type-safe API; no shell parsing; direct socket communication;
         supports streaming operations (logs, exec output).
       - Cons: Large dependency tree (~50 transitive deps); API surface is
         complex; may not support all Docker configurations (custom socket
         paths, remote hosts, Podman socket); tighter coupling to Docker
         API version.
    3. **Hybrid:** Use `bollard` for state queries (inspect, health check),
       CLI for file operations (cp).
       - Pros: Type-safe where it matters most (state inspection).
       - Cons: Two codepaths; inconsistent error handling; both dependency costs.
- **Decision:** **Option 1 (CLI via `tokio::process::Command`)**.
- **Rationale:** Our Docker interaction surface is small and well-defined
  (3 operations: inspect, cp, exec). The CLI approach has zero additional
  dependencies, is trivially testable (mock the command), and works with
  Docker, Podman, and other OCI-compatible runtimes. The type-safety benefits
  of bollard do not justify its dependency cost for our use case.
- **Trade-off:** Must parse CLI output. Mitigated by using `--format` flags
  for structured output (e.g., `docker inspect --format '{{.State.Running}}'`).

### ADR-006: Adapter Instantiation Pattern

- **Context:** The system must create the correct adapter instance based on
  config. Need a pattern that is extensible for future adapter types without
  over-engineering.
- **Options:**
    1. **Factory function with match:**

       ```rust
       fn create_adapter(env: &str) -> Result<Box<dyn TargetAdapter>>
       ```

       Parse the `environment` string and return the appropriate adapter.
       - Pros: Simple; Rust-idiomatic; easy to understand.
       - Cons: Adding new adapter types requires modifying the factory.
    2. **Registry pattern (`HashMap<String, AdapterFactory>`):**
       Register adapter factories by name, look up at runtime.
       - Pros: Extensible without modifying core code; plugin-friendly.
       - Cons: Overkill for 2 adapter types; runtime indirection; more
         complex error reporting.
- **Decision:** **Option 1 (Factory function with match)**.
- **Rationale:** Phase 2 has exactly 2 adapter types. YAGNI for a plugin-style
  registry. The factory function is trivially extensible when Phase 3 adds
  `SshAdapter` (add one match arm). Migration to a registry pattern is
  straightforward if Phase 4+ needs it.

## 6. Data Model

### TargetAdapter Trait (Rust)

```rust
#[async_trait]
pub trait TargetAdapter: Send + Sync {
    /// Adapter type identifier (e.g., "local", "docker").
    fn adapter_type(&self) -> &str;

    /// Check if the target environment is accessible.
    async fn health_check(&self) -> Result<()>;

    /// Check if a path exists inside the target.
    async fn path_exists(&self, path: &Path) -> Result<bool>;

    /// Execute the installation (symlink or copy).
    async fn install(&self, source: &Path, target: &Path, mode: InstallMode) -> Result<()>;

    /// Remove a previously installed skill from the target.
    async fn uninstall(&self, target: &Path) -> Result<()>;

    /// Run a command inside the target (for post-install hooks).
    async fn exec(&self, cmd: &str) -> Result<String>;
}
```

### Adapter Selection Logic

```text
environment = "local"                  → LocalAdapter (default)
environment = "docker:<container>"     → DockerAdapter { container_name }
environment = "ssh:<host>"             → future SshAdapter (not Phase 2)
environment = <unknown>                → validation error at config parse time
```

### LocalAdapter Behavior

- `health_check()`: Always returns `Ok(())` (local filesystem is always available).
- `path_exists()`: Wraps `tokio::fs::metadata`.
- `install()`: Wraps `tokio::fs::symlink` (symlink mode) or recursive copy (copy mode).
  Equivalent to Phase 1 behavior.
- `uninstall()`: Removes symlink or directory at target path.
- `exec()`: Wraps `tokio::process::Command` for local shell execution.

### DockerAdapter Internal Flow

```text
DockerAdapter::health_check()
├── docker inspect --format '{{.State.Running}}' <container>
├── if "true" → Ok(())
└── if error or "false" → Err(ContainerNotRunning)

DockerAdapter::install(source, target, mode)
├── health_check()
├── mode = copy always (symlinks cannot cross container boundary)
├── docker cp <host_source>/. <container>:<target>
└── verify: docker exec <container> test -e <target>

DockerAdapter::uninstall(target)
├── docker exec <container> rm -rf <target>
└── report success/failure

DockerAdapter::exec(cmd)
├── docker exec <container> sh -c <cmd>
└── capture stdout + stderr
```

**Note:** `DockerAdapter::install` always uses copy mode regardless of the
`install.mode` config value. Symlinks cannot cross container boundaries.
The adapter SHOULD emit a warning if `install.mode = "symlink"` is configured
for a Docker target.

## 7. Failure Semantics

- **LocalAdapter Failure:** Same as Phase 1 (exit code `1` for I/O errors).
- **DockerAdapter Connection Failure:** Fail fast for that specific skill with
  exit code `1`. Include container name and connection error in diagnostics.
- **DockerAdapter Copy Failure:** Report per-file failure. Partial injection
  is allowed but MUST be reported as a warning.
- **Health Check Failure:** MUST prevent install attempt. Report actionable
  error (e.g., "Container 'my-agent' is not running. Start it with
  `docker start my-agent`").
- **Docker Binary Missing:** When `docker` is not in PATH, DockerAdapter
  construction MUST fail with a clear error message: "Docker CLI not found.
  Install Docker or ensure `docker` is in your PATH."

## 8. Acceptance Criteria

1. `eden-skills install --target local` produces identical behavior to Phase 1 `apply`.
2. `eden-skills install --target docker:test-container` copies skill files into a
   running container's filesystem.
3. `DockerAdapter` health check fails gracefully when container is not running.
4. No Phase 1 test regressions when `LocalAdapter` is the default path.
5. `DockerAdapter` works with containers that do not have volume mounts.
6. Symlink mode configured for Docker target emits a warning and falls back to copy.

## 9. Freeze Candidates

Items requiring Stage B resolution before Builder implementation begins:

| ID | Item | Options Under Consideration | Resolution Needed |
| :--- | :--- | :--- | :--- |
| **FC-A1** | `uninstall` method scope | Full cleanup (`rm -rf` target) vs symlink-only removal vs marker-based cleanup | Define what "uninstall" means for each adapter type. |
| **FC-A2** | DockerAdapter retry policy | No retry (fail fast) vs 1 retry with backoff vs configurable retry count | Decide error recovery strategy for transient Docker failures. |
| **FC-A3** | Docker Compose service name support | `docker:<service>` via `docker compose exec` vs container-name-only | Decide if we support Compose service names or require resolved container names. |
| **FC-A5** | DockerAdapter symlink-to-copy fallback | Silent fallback vs warning + fallback vs error | Current recommendation: warning + fallback (Section 6 note). Needs confirmation. |

**Note:** FC-A4 (Send + Sync bounds) was resolved and promoted to normative
requirement ARC-108. `Send + Sync` is mandatory because `JoinSet::spawn`
requires `Send` on spawned futures, and `Sync` is needed for `Arc<dyn TargetAdapter>`
shared across tasks. This aligns with the `rust-async-patterns` best practice:
"Don't forget Send bounds — For spawned futures."
