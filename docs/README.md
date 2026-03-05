# eden-skills Tutorials

Task-oriented guides for all implemented features. If you are new, read in order.

## Learning Path

1. [Quickstart: First Successful Run](01-quickstart.md)  
   Install skills from a URL or set up a config-driven workflow (`init` → `apply` → `doctor`).

2. [Config Lifecycle Management](02-config-lifecycle.md)  
   Manage `skills.toml` from CLI (`add`, `remove`, `set`, `list`, `config export/import`), including batch and interactive remove.

3. [Registry and Install Workflow](03-registry-and-install.md)  
   Use the registry workflow (`update` + `install by name`) with Mode A / Mode B config and version constraints.

4. [Docker Targets Guide](04-docker-targets.md)  
   Configure Docker targets and understand adapter behavior and lock-diff cleanup.

5. [Safety, Strict Mode, and Exit Codes](05-safety-strict-and-exit-codes.md)  
   Safety metadata, strict mode semantics, and automation-friendly exit codes.

6. [Troubleshooting Playbook](06-troubleshooting.md)  
   Diagnose common failures using finding codes and recovery steps.

## Command Contract References

For normative behavior, refer to the spec files:

- [Phase 1 Command Contract](../spec/phase1/SPEC_COMMANDS.md)
- [Phase 1 Schema Contract](../spec/phase1/SPEC_SCHEMA.md)
- [Phase 2 Command Extensions](../spec/phase2/SPEC_COMMANDS_EXT.md)
- [Phase 2 Schema Extensions](../spec/phase2/SPEC_SCHEMA_EXT.md)
- [Phase 2 Adapter Contract](../spec/phase2/SPEC_ADAPTER.md)
- [Phase 2 Registry Contract](../spec/phase2/SPEC_REGISTRY.md)
- [Phase 2.5 Install URL Contract](../spec/phase2.5/SPEC_INSTALL_URL.md)
- [Phase 2.7 Lock Contract](../spec/phase2.7/SPEC_LOCK.md)
- [Phase 2.7 Help Contract](../spec/phase2.7/SPEC_HELP_SYSTEM.md)
- [Phase 2.7 Output Contract](../spec/phase2.7/SPEC_OUTPUT_POLISH.md)
- [Phase 2.7 Remove Contract](../spec/phase2.7/SPEC_REMOVE_ENH.md)
- [Phase 2.8 Table Rendering Contract](../spec/phase2.8/SPEC_TABLE_RENDERING.md)
- [Phase 2.8 Output Upgrade Contract](../spec/phase2.8/SPEC_OUTPUT_UPGRADE.md)
- [Phase 2.8 Code Structure Contract](../spec/phase2.8/SPEC_CODE_STRUCTURE.md)

For current implementation state:

- [Status Snapshot](../STATUS.yaml)
- [Execution Tracker](../EXECUTION_TRACKER.md)
