# Eden-Skills: The Next-Gen Agent Skills Ecosystem

**Version:** 1.0 | **Date:** Feb 2026 | **Status:** Execution In Progress (Phase 2 Closeout)

---

Execution tracking and model-boundary handoff log: `EXECUTION_TRACKER.md`
Machine-readable execution status: `STATUS.yaml`
Agent recovery/navigation entrypoint: `AGENTS.md`

---

## 1. Executive Summary & Problem Statement

### 1.1 The Context

With the proliferation of Agentic AI, standardization of "Skills" (executable capabilities for agents) has become a hot topic. Vercel's `skills.sh` (based on `vercel-labs/skills`) is the current pioneer, offering a CLI to install skills.

### 1.2 The "Two Critical Flaws" of Current Solutions

Our analysis of the existing `skills.sh` ecosystem reveals two fundamental bottlenecks that hinder professional adoption:

1. **The "Deterministic Install" Failure (Path Resolution + Link Verification):**
    * **Observation:** Even though the CLI exposes agent selection and symlink modes, real-world runs can still end with skills only present in `~/.agents/skills`, while `~/.claude/skills` linkage/discovery is not reliably established.
    * **Consequence:** The installation pipeline is non-deterministic across environments. Users must manually inspect, relink, and debug path mismatches after each install/update.
2. **The Discovery & Taxonomy Vacuum:**
    * **Observation:** Discovery relies on primitive full-text search (`grep` style) or simple popularity metrics.
    * **Consequence:** As the ecosystem grows, finding high-quality, domain-specific skills becomes impossible without a robust taxonomy (Categories + Tags) similar to `crates.io` or `npm`.

---

## 2. Strategic Vision: From "Extension" to "Platform"

### 2.1 The Pivot

Initially conceived as a "supplement" or "proxy" to `skills.sh`, the strategy has evolved based on legal and technical analysis.
> **Core Decision:** We will not build a dependency; we will build a **Competitor**.

### 2.2 Compliance & Legal Feasibility

* **License Check:** The core reference repos (e.g., `vercel-labs/skills`, `vercel-labs/agent-skills`) are under **MIT**.
  * *Verdict:* It is legal to fork, modify, redistribute, and index those repositories.
  * *Constraint:* For ecosystem-wide indexing, we must enforce per-repo license policy. Repositories without explicit open-source licenses are index-only (metadata), not mirror-and-redistribute.
* **Scraping Restrictions:** Vercel's Acceptable Use Policy and Terms restrict unauthorized automated extraction and abusive crawling behavior on hosted services.
  * *Verdict:* **Do not scrape the website.**
* **The Solution:** The source of truth is **GitHub**, not Vercel. We will index public GitHub repositories directly via the GitHub API, ensuring full compliance and independence.

### 2.3 Long-Term Goal (The "System B")

To build the "Google" or "NPM" for Agent Skills:

* **Platform Agnostic:** Not tied to Vercel/Next.js ecosystem.
* **AI-Curated:** Using LLMs to automatically categorize, tag, and rate skills based on code quality, not just popularity.
* **Double-Layer Taxonomy:** Strict Categories (L1) + Flexible Tags (L2).

---

## 3. Short-Term Execution: "Follow-Edens-Skills" (Tool C)

Before building the platform, we must solve the immediate pain point: installation reliability and repeatability across agent environments.

### 3.1 The Product: `eden-skills` (Provisional Name)

A local CLI tool acting as a "Skill State Reconciler" for Agent Skills.
Phase 1 implementation language is **Rust** (deterministic, typed, single-binary delivery).

### 3.2 Core Feature: The "Soul" of the Tool (Plan + Apply + Verify + Repair)

The tool must support a configuration-driven approach (Infrastructure as Code):

```toml
# ~/.config/eden-skills/skills.toml (Draft Schema)
version = 1

[[skills]]
id = "browser-tool"

[skills.source]
repo = "https://github.com/vercel-labs/skills.git"
subpath = "packages/browser"
ref = "main"

[skills.install]
mode = "symlink"

[[skills.targets]]
agent = "claude-code"
expected_path = "~/.claude/skills"

[[skills.targets]]
agent = "cursor"
expected_path = "~/.cursor/skills"

[skills.verify]
enabled = true
checks = ["path-exists", "is-symlink", "target-resolves"]
```

### 3.3 Implementation Logic

1. **Parse** the `skills.toml`.
2. **Clone/Update** the repo to a central storage.
3. **Plan** the target graph (source path, target path, install mode) and output a dry-run diff.
4. **Detect** local Agents and resolve path strategy (`~/.claude/skills`, `~/.cursor/skills`, etc.).
5. **Apply** (`ln -sf` or copy) idempotently.
6. **Verify** with post-install checks (existence, link validity, path resolution).
7. **Repair** drifted state automatically (broken symlink, moved source, stale target).

### 3.4 Success Criteria (Phase 1)

* Re-running `eden-skills apply` is idempotent and produces no unintended changes.
* `eden-skills doctor` can detect and explain all broken mappings.
* `eden-skills repair` can self-heal common failure states without manual relinking.

### 3.5 Security & Supply Chain Baseline (Phase 1)

* **License Gate:** Block mirror-and-redistribute for repositories without explicit permissive licenses.
* **Integrity Metadata:** Persist source repo, commit SHA, and retrieval timestamp for each installed skill.
* **Risk Labels:** Mark skills that include executable scripts (`.sh`, `.py`, binaries) as "review-required" before enablement.
* **Execution Policy:** Recommend sandboxed execution path for high-risk skills and provide a "no-exec metadata-only" mode.

### 3.6 Specification-First Delivery Rule (Phase 1)

To keep implementation deterministic across human and AI contributors:

* Phase 1 normative specs live under `spec/`.
* `spec/**/*.md` is the implementation source of truth for CLI behavior.
* Any behavior change must update `spec/` first, then code, then tests.
* `ROADMAP.md` remains strategic; `README.md` remains summary-level.

---

## 4. AI Collaboration Roadmap

Leveraging **GPT-5.3-Codex** (High Speed/Code Gen) and **Claude Opus 4.6** (High Reasoning/Architecture).

### Phase 1: The CLI Foundation (Immediate)

* **Role:** Architect (Claude) defines the `skills.toml` schema, verification model, and "Agent Detection Strategy".
* **Role:** Builder (GPT-5.3) writes the **Rust CLI** implementing `plan/apply/doctor/repair` from `spec/`.
* **Deliverable:** A deterministic installer that solves the "Last Mile Reliability" problem.

### Phase 2: The "Hyper-Loop" Core Upgrade (New Priority)

* **Context:** Before scaling the data (crawler), we must scale the tool's capability to handle complexity and performance.
* **Role:** Architect (Claude) defines the `TargetAdapter` interface and `AsyncReactor` pattern.
* **Role:** Builder (GPT-5.3) refactors the CLI from serial-sync to parallel-async.
* **Deliverables:**
    1. **Concurrency:** `tokio`-based parallel download/update for registries.
    2. **Environment Agnosticism:** `DockerAdapter` to inject skills into running containers.
    3. **Registry Resolution:** Support for `official` (curated) and `forge` (community) double-track sources.

### Phase 3: The Data Engine (Mid-Term)

* **Role:** Architect (Claude) designs the **Double-Layer Taxonomy** (Categories vs. Tags).
* **Role:** Builder (GPT-5.3) writes a **GitHub Crawler** (using official API) to find repositories with `SKILL.md`.
* **Role:** Curator (Claude) processes the raw `SKILL.md` files:
* "Read this skill description."
* "Assign it to Category: [Data Analysis | Web | DevOps]."
* "Generate 3 optimized search tags."
* "Score quality with rubric-backed dimensions, not a single opaque number."

#### Phase 3 Engineering Constraints (GitHub API Reality)

* **Search Ceiling:** GitHub Search endpoints cap query results at 1,000 items per search expression.
* **Rate Limits:** Search and core APIs have distinct rate limits; crawler must implement token-aware throttling.
* **Partial Results:** Some responses can be `incomplete_results=true`; pipeline must support retry and reconciliation.
* **Design Requirement:** Use query sharding (time/range partition), incremental sync by `updated_at`, deduplication by repo ID + path, and exponential backoff.

#### Phase 3 Curation Quality Controls

* **Scoring Rubric:** Break quality into weighted dimensions (documentation clarity, maintenance signals, safety posture, usability).
* **Human Calibration:** Sample and manually review scored outputs each cycle; track inter-rater drift and adjust prompts/weights.
* **Versioned Outputs:** Store `model_version`, `prompt_version`, and `rubric_version` with each record for reproducibility.

### Phase 4: The Platform Launch (Long-Term)

* Merge the CLI with the new Dataset.
* Launch the search interface.

---

## 5. Next Steps (Action Items)

Operational progress for these items is tracked in `STATUS.yaml` and `EXECUTION_TRACKER.md`.
This section remains a strategic checklist.

### Phase 1: Foundation (Completed)

1. [x] **Initialize Repo:** Create `eden-skills`.
2. [x] **Freeze Specs:** Define Phase 1 contracts (`spec/phase1/SPEC_SCHEMA.md`, etc.).
3. [x] **Draft Config:** Create `skills.toml` with manual Git sources.
4. [x] **Rust CLI Build:** Implement `plan/apply/doctor/repair` (Serial & Local).
5. [x] **Safety Gate MVP:** Implement risk metadata persistence and checks.
6. [x] **CLI UX Refactor:** Adopt `clap` for robust subcommand parsing.

### Phase 2: Core Upgrade (Implemented; closeout hardening tracked)

1. [x] **Phase 2 Spec Freeze:** Stage A/B contract workflow completed for Phase 2 specs.
2. [x] **Registry RFC + Implementation:** TOML-backed multi-registry index structure (`official`/`forge`) implemented and tested.
3. [x] **Async Refactor:** Serial source sync path refactored to `tokio` + bounded-concurrency reactor execution.
4. [x] **Docker Adapter:** `TargetAdapter` Docker implementation (`docker exec/cp`) implemented with health/error handling.
5. [x] **Registry Resolution:** Mode B skill resolution from registry index implemented (`install` + `apply`/`repair`).
6. [x] **Closeout Audit:** Builder closeout items `P2-CLOSE-001` to `P2-CLOSE-003` dispositioned and synchronized in tracking docs.
7. [ ] **Post-Release Hardening:** Deferred scenarios `TM-P2-015`, `TM-P2-027`, `TM-P2-029`.

### Phase 3: Data Engine (Future)

1. [ ] **Crawler RFC:** Define sharding and rate-limit handling for GitHub API.
2. [ ] **Curation RFC:** Define quality scoring rubric and calibration process.

> *"The goal is not just to download code; it is to configure the agent's environment seamlessly."*
