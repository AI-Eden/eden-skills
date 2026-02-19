# eden-skills Tutorials

This directory contains task-oriented guides for features that are already implemented and stabilized in Phase 1 and Phase 2.

If you are new, read in order.

## Learning Path

1. [Quickstart: First Successful Run](01-quickstart.md)  
   First run with `init`, `plan`, `apply`, `doctor`, `repair`.

2. [Config Lifecycle Management](02-config-lifecycle.md)  
   Manage `skills.toml` from CLI (`add`, `remove`, `set`, `list`, `config export/import`).

3. [Registry and Install Workflow](03-registry-and-install.md)  
   Use Phase 2 registry workflow (`update`, `install`) and Mode A/Mode B config.

4. [Docker Targets Guide](04-docker-targets.md)  
   Configure Docker targets and understand Docker adapter behavior.

5. [Safety, Strict Mode, and Exit Codes](05-safety-strict-and-exit-codes.md)  
   Safety metadata, strict mode semantics, and automation-friendly exit codes.

6. [Troubleshooting Playbook](06-troubleshooting.md)  
   Diagnose common failures using finding codes and command output patterns.

## Command Contract References

For normative behavior, refer to specs:

- [Phase 1 Command Contract](../spec/phase1/SPEC_COMMANDS.md)
- [Phase 1 Schema Contract](../spec/phase1/SPEC_SCHEMA.md)
- [Phase 2 Command Extensions](../spec/phase2/SPEC_COMMANDS_EXT.md)
- [Phase 2 Schema Extensions](../spec/phase2/SPEC_SCHEMA_EXT.md)
- [Phase 2 Adapter Contract](../spec/phase2/SPEC_ADAPTER.md)
- [Phase 2 Registry Contract](../spec/phase2/SPEC_REGISTRY.md)

For current implementation state:

- [Status Snapshot](../STATUS.yaml)
- [Execution Tracker](../EXECUTION_TRACKER.md)
