# Phase 2 Architecture Spec (Stage A): The "Hyper-Loop" Upgrade

**Version:** 2.0 (Draft)
**Parent:** `ROADMAP.md`
**Objective:** Evolve `eden-skills` from a local file-linker to a high-performance, environment-agnostic package manager.

---

## 0. Guardrails (Non-Negotiable)

These rules apply to all Phase 2 design and implementation work. Sourced from `AGENTS.md` and project-wide conventions.

1. **AGENTS.md Compliance:** Read and follow `AGENTS.md` first, especially Read Order, Authority Order, Role Boundaries, and Guardrails.
2. **Authority Order:** When files conflict, resolution MUST follow: `spec/**/*.md` > `STATUS.yaml` > `EXECUTION_TRACKER.md` > `ROADMAP.md` > `README.md`.
3. **Responsibility Boundary:** Architect owns taxonomy, curation rubric, and crawler strategy. Builder owns implementation, tests, and refactors. Do not perform Builder-owned implementation work in Architect-owned deliverables, and vice versa.
4. **Language Policy:** Talk to user in Chinese. All repository file content MUST be English-only.
5. **Phase Isolation:** Do not alter Phase 1 CLI behavior contracts (`spec/phase1/SPEC_COMMANDS.md`, `spec/phase1/SPEC_SCHEMA.md`, etc.). Phase 2 contracts must be isolated in `spec/phase2/` and MUST NOT inject semantics into existing Phase 1 normative sections.

---

## 1. Architectural Pivot

### 1.1 The "Async Reactor" Pattern (Concurrency)

**Current State (Phase 1):** The `apply` command processes skills serially (Clone A -> Link A -> Clone B -> Link B).
**Phase 2 Requirement:**
The CLI must utilize a `TaskQueue` based on `tokio`. Network IO (Cloning/Pulling) must be concurrent. Disk IO (Linking) should remain serial or locked per-path to avoid race conditions.

* **Component:** `SkillReactor`
* **Logic:**
    1. Parse `skills.toml`.
    2. Build a Dependency Graph (DAG) - *Prepared for future dependency resolution*.
    3. Spawn `N` worker threads (default to CPU cores * 2).
    4. Fetch/Update sources in parallel.
    5. Execute Install/Link steps as resources become available.

### 1.2 The "Target Adapter" Pattern (Environment Agnosticism)

**Current State (Phase 1):** The CLI assumes `std::fs::symlink` works on the local filesystem.
**Phase 2 Requirement:**
We must decouple the *intent* ("Install Skill X to Location Y") from the *execution* ("Syscall").

* **Trait Definition (Rust):**

    ```rust
    #[async_trait]
    pub trait TargetAdapter {
        // Check if the target environment is accessible (e.g., Docker container running?)
        async fn health_check(&self) -> Result<()>;
        
        // Check if a path exists inside the target
        async fn path_exists(&self, path: &Path) -> bool;
        
        // Execute the installation (Symlink or Copy)
        async fn install(&self, source: &Path, target: &Path, mode: InstallMode) -> Result<()>;
        
        // Run a command inside the target (for post-install hooks)
        async fn exec(&self, cmd: &str) -> Result<String>;
    }
    ```

* **Implementations:**
  * `LocalAdapter`: Wraps `std::fs` (The Phase 1 logic).
  * `DockerAdapter`: Wraps `docker exec` and `docker cp`. allowing skills to be injected into running Agent containers without mounting volumes.
  * `SshAdapter` (Future): Wraps `ssh` for remote dev servers.

---

## 2. The Registry System (Double-Track)

### 2.1 Configuration Schema Upgrade

We are moving from "Hardcoded Git URLs" to "Registry Resolution".

**Updated `skills.toml` Schema:**

```toml
# New Section: Registry Definitions
[registries]

[registries.official]
url = "[https://github.com/eden-skills/registry-official.git](https://github.com/eden-skills/registry-official.git)"
priority = 100
auto_update = true

[registries.forge]
url = "[https://github.com/eden-skills/registry-forge.git](https://github.com/eden-skills/registry-forge.git)"
priority = 50

# Modified Skills Section
[[skills]]
# Option A: Direct Git (Legacy Phase 1 support, keeps working)
id = "my-private-tool"
source = { repo = "...", ref = "main" }

# Option B: Registry Resolution (Phase 2 New Feature)
# This looks up "google-search" in 'official' then 'forge'
name = "google-search"
version = "^2.0" 
registry = "official" # Optional constraint

[[skills.targets]]
# Target specific environments
environment = "local" # or "docker:my-agent-container"
path = "~/.claude/skills"

```

### 2.2 The Registry Index Format

The registries (`official` and `forge`) are essentially git repositories containing a strictly structured index.

* **Structure:**

```text
registry-repo/
├── index/
│   ├── g/
│   │   └── google-search.toml  <-- Metadata & pointers to actual repo
│   ├── p/
│   │   └── python-interpreter.toml
├── README.md
└── POLICY.md

```

* **Resolution Logic:**

1. `eden update`: Pulls the latest commits from configured registry repos (concurrently).
2. `eden install google-search`:

* Checks `official/index/g/google-search.toml`.
* If not found, checks `forge/index/g/google-search.toml`.
* Reads the Git URL and Commit Hash from the TOML.
* Passes it to the `Downloader`.

---

## 3. Implementation Plan (The "Vibe Coding" Prompts)

### Step 1: The Refactor

* **Goal:** non-breaking refactor to introduce `AsyncReactor`.
* **Prompt Strategy:** "Refactor the current `apply` loop. Instead of iterating serially, create a `stream::iter` of tasks and use `buffer_unordered` to execute the git fetch operations in parallel. Ensure the `repair` logic remains synchronous or thread-safe."

### Step 2: The Adapter

* **Goal:** Abstract FS operations.
* **Prompt Strategy:** "Create a `TargetAdapter` trait. Move all `std::fs` calls currently in `main.rs` into a struct `LocalAdapter` that implements this trait. The main logic should strictly call the trait methods, not `std::fs` directly."

### Step 3: The Docker Driver

* **Goal:** Support containers.
* **Prompt Strategy:** "Implement `DockerAdapter` for `TargetAdapter`. Use the `bollard` crate (Rust Docker client) or simple `std::process::Command` calls to `docker` CLI. It should support copying files into a container path if symlinking is not possible (or warn the user)."

---

## 4. Success Criteria for Phase 2

1. **Performance:** `eden update` with 20 skills takes < 2 seconds (assuming cached) or saturates network bandwidth (uncached).
2. **Versatility:** Can install a skill into a running Docker container from the host machine via `eden install --target docker:container_name`.
3. **Ecosystem:** The `eden-official` registry repo exists (even if empty) and the CLI can read from it.
