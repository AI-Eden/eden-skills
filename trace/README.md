# trace/

Execution records for all phases (both active and frozen). Each phase
directory contains `status.yaml` (machine-readable batch progress) and
`tracker.md` (human-readable narrative).

## Purpose

Root `STATUS.yaml` and `EXECUTION_TRACKER.md` are routing files that
contain only project metadata and pointers to `trace/<phase>/`.
All phase-specific execution data lives here so that:

1. Root files stay small and stable — they never grow during a phase.
2. Builders only edit `trace/<current-phase>/` — no root file churn.
3. Phase completion requires zero file migration (data is already here).
4. Historical records remain accessible for audits and retrospectives.

## Structure

```text
trace/
├── README.md
├── phase1/        (frozen)
│   ├── status.yaml
│   └── tracker.md
├── phase2/        (frozen)
│   ├── status.yaml
│   └── tracker.md
├── phase2.5/      (frozen)
│   ├── status.yaml
│   └── tracker.md
├── phase2.7/      (frozen)
│   ├── status.yaml
│   └── tracker.md
└── phase2.8/      (active)
    ├── status.yaml
    └── tracker.md
```

## Convention

When a new phase starts:

1. Create `trace/<phase>/` with `status.yaml` and `tracker.md`.
2. Add a pointer entry in root `STATUS.yaml` under `phases:`.
3. Add a row in root `EXECUTION_TRACKER.md` phase records table.
4. Builder updates `trace/<phase>/` files after every batch.

When a phase completes:

1. Update `status` in `trace/<phase>/status.yaml` to `closeout_completed`.
2. Update the pointer in root `STATUS.yaml` with `completed_at`.
3. No file migration needed — data is already in `trace/<phase>/`.
