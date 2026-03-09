# Agentic Engineering Workflow

eden-skills was built using a structured multi-model AI collaboration. This
document explains the workflow so that contributors can use the same approach —
or adapt it to their own.

## The Three Roles

| Role | Model | Owns | Cannot |
| --- | --- | --- | --- |
| **Scout** | Gemini | Market research, competitive analysis, roadmap drafting | Make architecture decisions or write code |
| **Architect** | Claude | Behavior specs, architecture decisions, large-scale refactoring direction | Write implementation code or modify `crates/` |
| **Builder** | GPT | Implementation code, tests, CI, doc sync | Modify spec files or make architecture decisions |

A human orchestrator assigns tasks, reviews outputs, and resolves conflicts
between roles. The human owns final approval on all deliverables.

## How It Works

### Phase Lifecycle

Each development phase follows this cycle:

```txt
Scout (research) → Architect (spec) → Builder (implement) → Human (review)
```

1. **Scout** analyzes the problem space, evaluates options, and produces a
   strategic brief (captured in `ROADMAP.md`).
2. **Architect** reads the brief and writes formal behavior specifications
   under `spec/<phase>/`. This happens in two stages:
   - **Stage A** (exploratory): propose 2-3 design options per domain with
     trade-off analysis; mark unresolved items as "Freeze Candidates".
   - **Stage B** (freeze): close all candidates, produce `CONTRACT_FROZEN`
     specifications with MUST/SHOULD/MAY requirements.
3. **Builder** reads the frozen specs and implements code + tests. Work is
   organized into batches following the dependency graph in the phase README.
4. **Human** reviews the output, runs CI, and signs off.

### Kick Files

The `prompt/` directory contains one kick file per phase per role. Each kick
file is a role-scoped prompt with the following structure:

```txt
[Your Identity]   — role name, what you own, what you must not do
[Context]         — which specs to read, which phases are frozen
[Pre-Flight Check]— files to verify before starting work
[Your Mission]    — concrete deliverables for this phase
[Hard Constraints]— language policy, authority order, phase isolation
```

This structure enforces **AI RBAC** (role-based access control): the Architect
prompt explicitly forbids writing code; the Builder prompt explicitly forbids
modifying spec files. The separation prevents a single model from making both
the design decision and the implementation, which improves auditability.

### Batch and Handoff

When a phase is too large for a single context window, the Builder kick file
defines a batch execution rhythm:

1. Read the phase README for the work package dependency graph.
2. Execute one batch (a set of independent work packages).
3. Report completion status and any blocking issues.
4. The human triggers the next batch with a handoff message.

This pattern was used in every phase from 2.0 through 2.97, producing 38,500
lines of Rust across multiple sessions without context loss.

### Skill Injection

Builder kick files require the agent to proactively read and follow workspace
skills (e.g., Rust Best Practices, Rust Async Patterns) before writing the
corresponding code. This means eden-skills — a skills manager — was itself
built using agent skills.

## The Spec Directory

The `spec/` directory contains 62 behavior specification files totaling over
11,000 lines. These are organized by phase:

```txt
spec/
├── phase1/          — CLI foundation (commands, schema, agent paths)
├── phase2/          — async reactor, adapter, registry
├── phase2.5/        — URL install, agent detection, distribution
├── phase2.7/        — lock file, help system, output polish
├── phase2.8/        — table rendering, output upgrade, code structure
├── phase2.95/       — performance, Windows junction, Docker bind
├── phase2.97/       — update fix, table style, interactive UX, cache clean
└── README.md        — spec index
```

Each spec file follows a consistent format:

- **Requirement ID** (e.g., `UFX-001`, `IUX-003`)
- **Priority** (MUST / SHOULD / MAY)
- **Statement** (what the system must do)
- **Verification** (how to test it)
- **Traceability** links to implementation

The specs are the Architect's output, not a prerequisite for human contributors.
If you use AI agents to contribute, point your Architect at the existing spec
structure and it will produce specs in the same format.

## Contributing with AI Agents

You do not need to write specs by hand. The intended workflow for contributors:

1. **Identify the change** you want to make (bug fix, feature, refactor).
2. **For small changes** (< 100 lines): skip specs, submit code directly with
   tests. Reference existing spec IDs in your PR description if applicable.
3. **For larger changes**: use an AI agent as your Architect. Feed it the
   relevant `spec/<phase>/` files as context and ask it to draft a spec
   amendment or new spec. Then use a Builder agent to implement from the spec.
4. **Review** the output yourself before submitting.

The kick files in `prompt/` are MIT-licensed examples. Feel free to adapt them
for your own projects or fork the workflow entirely.

## Key Files

| File | Purpose |
| --- | --- |
| [`AGENTS.md`](../AGENTS.md) | Agent coordination guide; read order, authority order, role boundaries |
| [`ROADMAP.md`](../ROADMAP.md) | Strategic vision and phase definitions |
| [`STATUS.yaml`](../STATUS.yaml) | Machine-readable execution state |
| [`EXECUTION_TRACKER.md`](../EXECUTION_TRACKER.md) | Detailed work package tracking |
| [`prompt/`](../prompt/) | Complete kick file archive (11 files) |
| [`spec/`](../spec/) | Behavior specifications (62 files) |
