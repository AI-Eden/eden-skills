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

## 3. Non-Goals

- `SshAdapter` or any remote-server adapter beyond Docker (deferred to Phase 3+).
- Real-time file synchronization for Docker targets (`docker cp` is point-in-time).
- Automatic agent environment discovery (users must specify targets explicitly).

## 4. Normative Requirements

| ID | Owner | Priority | Statement | Verification |
| :--- | :--- | :--- | :--- | :--- |
| **ARC-101** | Builder | **P0** | The system MUST define a `TargetAdapter` trait decoupling intent from syscalls. | Code contains `trait TargetAdapter` with async methods for health check, path existence, install, and exec. |
| **ARC-102** | Builder | **P0** | A `LocalAdapter` implementation MUST be provided for backward compatibility. | Phase 1 integration tests pass without modification when using `LocalAdapter`. |
| **ARC-103** | Builder | **P0** | A `DockerAdapter` implementation MUST be provided using `docker` CLI or API. | Can install a skill to a running container without volume mount. |
| **ARC-104** | Builder | **P1** | The `DockerAdapter` MUST support `cp` (copy) injection strategy as the primary reliable method. | File appears in container after `eden-skills install` even without shared volumes. |

## 5. Architecture Decision

### ADR-001: Docker Injection Strategy

- **Context:** Agents in Docker containers need access to skills. Symlinks on
  host do not work inside containers without volume mapping.
- **Options:**
    1. *Dynamic Volume Mount:* Restart container with `-v`.
    2. *Docker CP:* Copy files via API/CLI.
    3. *SSH/SCP:* Treat container as remote server.
- **Decision:** **Option 2 (Docker CP)**.
- **Rationale:** Least intrusive. Does not require restarting the container
  (which kills Agent state). Matches "Injection" paradigm.
- **Trade-off:** Updates are not real-time (no symlink magic). `eden-skills update`
  must re-copy files after source changes.
- **Rollback Trigger:** If `docker cp` proves unreliable for large skill trees
  or permission-sensitive paths, consider SSH-based injection.

## 6. Data Model

### TargetAdapter Trait (Rust)

```rust
#[async_trait]
pub trait TargetAdapter {
    /// Check if the target environment is accessible.
    async fn health_check(&self) -> Result<()>;

    /// Check if a path exists inside the target.
    async fn path_exists(&self, path: &Path) -> bool;

    /// Execute the installation (symlink or copy).
    async fn install(&self, source: &Path, target: &Path, mode: InstallMode) -> Result<()>;

    /// Run a command inside the target (for post-install hooks).
    async fn exec(&self, cmd: &str) -> Result<String>;
}
```

### Adapter Selection Logic

```text
target = "local"                  → LocalAdapter (default)
target = "docker:<container>"     → DockerAdapter { container_name }
target = "ssh:<host>"             → future SshAdapter (not Phase 2)
```

## 7. Failure Semantics

- **LocalAdapter Failure:** Same as Phase 1 (exit code `1` for I/O errors).
- **DockerAdapter Connection Failure:** Fail fast for that specific skill with
  exit code `1`. Include container name and connection error in diagnostics.
- **DockerAdapter Copy Failure:** Report per-file failure. Partial injection
  is allowed but MUST be reported as a warning.
- **Health Check Failure:** MUST prevent install attempt. Report actionable
  error (e.g., "Container 'my-agent' is not running").

## 8. Acceptance Criteria

1. `eden-skills install --target local` produces identical behavior to Phase 1 `apply`.
2. `eden-skills install --target docker:test-container` copies skill files into a
   running container's filesystem.
3. `DockerAdapter` health check fails gracefully when container is not running.
4. No Phase 1 test regressions when `LocalAdapter` is the default path.
