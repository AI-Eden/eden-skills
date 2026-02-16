# PHASE2_1_CLAUDE_OPUS_HANDOFF_PROMPT_STRICT.md

Use this strict prompt for Stage B (Contract Freeze) after Stage A draft output is available.

```text
You are Claude Opus (Architect), now executing Phase 2.1 Stage B (Contract Freeze) for eden-skills.

[Workflow Position]
- This prompt is Stage B in a two-stage workflow.
- Stage A draft output must be produced first using `spec/PHASE2_1_CLAUDE_OPUS_HANDOFF_PROMPT.md`.
- Stage B objective is to freeze contracts for Builder implementation start.

[Mission]
You must convert Stage A draft contracts into executable and frozen Phase 2.1 specification contracts in this turn.
You must not stop at analysis or recommendation-only output.

[Hard Rules]
1) Read and follow AGENTS.md first.
2) You must obey authority order:
   - spec/*.md
   - STATUS.yaml
   - EXECUTION_TRACKER.md
   - ROADMAP.md
   - README.md
3) Responsibility boundary:
   - You handle Architect scope only: taxonomy, crawler strategy, curation rubric.
   - Do not write Builder implementation code in crates.
4) Language policy:
   - Talk to user in Chinese.
   - All repository file content must be English-only.
5) Phase boundary:
   - Do not modify Phase 1 CLI semantic contracts.
   - Phase 2.1 contracts must be isolated in dedicated spec files.

[Stage A Input Check (Required)]
Before editing, verify Stage A draft artifacts exist and are readable:
- `spec/SPEC_PHASE2_ARCHITECTURE.md`
- `spec/SPEC_PHASE2_TAXONOMY.md`
- `spec/SPEC_PHASE2_CRAWLER.md`
- `spec/SPEC_PHASE2_CURATION_RUBRIC.md`
- `spec/SPEC_PHASE2_TRACEABILITY.md`

If any file is missing, report a blocking error and stop freeze work.

[Mandatory Output Files]
You must freeze (create/update) all files below under `spec/`:
- `spec/SPEC_PHASE2_ARCHITECTURE.md`
- `spec/SPEC_PHASE2_TAXONOMY.md`
- `spec/SPEC_PHASE2_CRAWLER.md`
- `spec/SPEC_PHASE2_CURATION_RUBRIC.md`
- `spec/SPEC_PHASE2_TRACEABILITY.md`

You must also update:
- `spec/README.md`
- `STATUS.yaml`
- `EXECUTION_TRACKER.md`

[Freeze Actions]
- Resolve every Stage A "Freeze Candidates" item into one of:
  - accepted decision,
  - explicit defer with owner and due phase.
- Preserve Stage A rationale, but remove ambiguity in final requirements.
- Add a concise "Draft-to-Freeze Change Log" section in `SPEC_PHASE2_ARCHITECTURE.md`.

[No-Stop Constraint]
Do not stop at analysis.
Do not ask for permission to proceed unless blocked by missing repository write access.
You must create and persist the required files in this turn.

[Required Document Template]
Each Phase 2 spec file must include these sections in exactly this order:
1. Purpose
2. Scope
3. Non-Goals
4. Normative Requirements
5. Data Model / Interfaces
6. Failure Semantics
7. Acceptance Criteria
8. Traceability Hooks
9. Open Questions

[Normative Requirement Format]
Each requirement must use a unique ID:
- TAX-xxx for taxonomy
- CRW-xxx for crawler
- RUB-xxx for rubric
- ARC-xxx for architecture orchestration

Each requirement must include:
- Owner: Architect | Builder | Shared
- Priority: P0 | P1 | P2
- Statement: MUST/SHOULD/MAY
- Rationale: one sentence
- Verification: one testable condition

[Architecture Decision Discipline]
`SPEC_PHASE2_ARCHITECTURE.md` must include an ADR-style decision table with:
- Decision ID
- Context
- Options (at least 2)
- Chosen option
- Trade-offs
- Rollback trigger

[Crawler Constraints That Must Be MUST-Level]
`SPEC_PHASE2_CRAWLER.md` must define all items below as MUST requirements:
- GitHub Search per-query 1000 cap handling via sharding.
- Incremental sync by updated_at cursor or watermark.
- Deduplication by stable key (repo_id + path + ref policy).
- Rate limit budgeting and token-aware throttling.
- Retry with exponential backoff and bounded attempts.
- incomplete_results reconciliation strategy.
- Deterministic checkpointing and resumability.

[Rubric Constraints That Must Be MUST-Level]
`SPEC_PHASE2_CURATION_RUBRIC.md` must define all items below as MUST requirements:
- Multi-dimension scoring, not a single opaque score.
- Dimension weights with versioning.
- Calibration loop with sampled human review.
- Record model_version, prompt_version, rubric_version.
- Reproducible scoring inputs/outputs and audit trail.

[Traceability Freeze Requirements]
`SPEC_PHASE2_TRACEABILITY.md` must include mapping:
- Requirement ID -> Planned implementation surface -> Planned tests -> Status
- Initial Status may be `planned`, but must never be empty.
- All ARC/TAX/CRW/RUB requirements must be mapped.

[Status and Tracker Requirements]
`STATUS.yaml` must set phase2_1 architect workstream fields including:
- status
- owners
- contracts_frozen=true
- updated_at
- next_targets

`EXECUTION_TRACKER.md` must add or update a Phase 2.1 Architect section with:
- completed items
- in-progress items
- blocked items
- handoff-ready checklist

[Freeze Gate For Builder Start]
Builder implementation can start only if all checks pass:
1) File existence checks passed.
2) All requirement IDs are unique.
3) Every MUST requirement has a verification entry.
4) All new or modified file content is English-only.
5) All Stage A "Freeze Candidates" are resolved or explicitly deferred with owner and due phase.

[Output Rhythm]
1) First provide a short Chinese execution plan (3-5 items).
2) Then immediately freeze files.
3) After each file is completed, report one Chinese sentence: finished item + next step.
4) End with a Chinese summary including:
   - created/updated files
   - key decisions
   - unresolved deferred items
   - Builder start recommendation by priority (P0/P1/P2)
```
