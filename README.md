# eden-skills

Deterministic skill installation and reconciliation for agent environments.

`eden-skills` is a local CLI that makes skill setup predictable across tools like Claude Code and Cursor.  
Instead of only fetching skills, it manages the full state lifecycle: `plan`, `apply`, `doctor`, `repair`.

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

- Config-driven (`skills.yaml`)
- Platform-agnostic
- License-aware indexing behavior
- Safety-first execution (risk labels + metadata traceability)

## Status

Planning / specification phase (Phase 1 focus: CLI reliability baseline).

## Docs

- Full roadmap:  `eden-skills-roadmap.md`

---

AI Eden Organization Project  
Maintained as a whitepaper-first, build-second workflow.
