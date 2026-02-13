# eden-skills

Deterministic skill installation and reconciliation for agent environments.

`eden-skills` is a local CLI that makes skill setup predictable across tools like Claude Code and Cursor.  
Instead of only fetching skills, it manages the full state lifecycle: `plan`, `apply`, `doctor`, `repair`.
Phase 1 implementation stack is Rust.

## Why This Exists

In real-world setups, skill install paths and runtime discovery paths can drift (for example `~/.agents/skills` vs `~/.claude/skills`).  
`eden-skills` focuses on reliability:

- Deterministic path resolution
- Idempotent installs
- Post-install verification
- Auto-repair for broken or stale mappings

## Core Workflow

1. `plan`: show what will change (dry-run diff)
2. `apply`: perform symlink/copy operations idempotently
3. `doctor`: detect broken mappings and explain why
4. `repair`: reconcile drifted state automatically

## Design Principles

- Config-driven (`skills.toml`)
- Platform-agnostic
- License-aware indexing behavior
- Safety-first execution (risk labels + metadata traceability + metadata-only mode)

## Status

Active implementation (Phase 1 focus: CLI reliability baseline).
Authoritative machine-readable status: `STATUS.yaml`.

## Docs

- Agent handoff and recovery guide: `AGENTS.md`
- Machine-readable status snapshot: `STATUS.yaml`
- Full roadmap: `ROADMAP.md`
- Execution tracker and model-boundary ownership: `EXECUTION_TRACKER.md`
- Spec index and writing rules: `spec/README.md`
- CLI spec set (source of truth for implementation): `spec/`
- Requirement-to-code/test mapping: `spec/SPEC_TRACEABILITY.md`
- Sample config for local dev: `skills.toml`

## Workspace Layout

- `crates/eden-skills-core`: shared domain models and core logic
- `crates/eden-skills-cli`: Phase 1 binary (`eden-skills`)
- `crates/eden-skills-indexer`: Phase 2 binary placeholder (crawler/data engine)

## Local Commands

- `cargo check --workspace`
- `cargo run -p eden-skills-cli -- plan --config ./skills.toml`

---

AI Eden Organization Project  
Maintained as a whitepaper-first, build-second workflow.
