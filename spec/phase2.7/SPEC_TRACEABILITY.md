# SPEC_TRACEABILITY.md

Requirement-to-implementation mapping for Phase 2.7.
Use this file to recover accurate context after compression.

**Status:** Skeleton — Builder fills `Implementation`, `Tests`, and
`Status` columns during TDD.

## 1. Lock File Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
|---|---|---|---|---|---|
| LCK-001 | `SPEC_LOCK.md` 5.2 | `apply` MUST generate `Remove` actions for skills in lock but absent from TOML | `eden-skills-core/src/lock.rs` (`compute_lock_diff`, `LockTarget.environment`), `eden-skills-cli/src/commands.rs` (`uninstall_orphaned_lock_entries`) | `lock_diff_tests.rs` (TM-P27-004, TM-P27-014) | done |
| LCK-002 | `SPEC_LOCK.md` 4.1 | Lock file MUST be written after every mutating command | `eden-skills-cli/src/commands.rs` (`write_lock_for_config`, init/apply/repair/install/remove), `eden-skills-core/src/lock.rs` (`LockTarget.environment` persisted in lock) | `lock_lifecycle_tests.rs` (TM-P27-001~003, TM-P27-012, TM-P27-013), `lock_diff_tests.rs` (TM-P27-014) | done |
| LCK-003 | `SPEC_LOCK.md` 3.1 | Lock file MUST use TOML format with required fields | `eden-skills-core/src/lock.rs` (`LockFile`, `LockSkillEntry`, `LockTarget`, `write_lock_file`) | `lock_tests.rs` (round_trip, contains_all_required_fields) | done |
| LCK-004 | `SPEC_LOCK.md` 2.2 | Lock file MUST be co-located with config file | `eden-skills-core/src/lock.rs` (`lock_path_for_config`) | `lock_tests.rs` (replaces_toml, appends_lock), `lock_lifecycle_tests.rs` (co_located) | done |
| LCK-005 | `SPEC_LOCK.md` 4.3 | Missing lock file MUST NOT cause errors | `eden-skills-core/src/lock.rs` (`read_lock_file`) | `lock_tests.rs` (missing_returns_none), `lock_lifecycle_tests.rs` (TM-P27-006) | done |
| LCK-006 | `SPEC_LOCK.md` 4.4 | Corrupted lock file MUST emit warning and proceed | `eden-skills-core/src/lock.rs` (`read_lock_file`) | `lock_tests.rs` (corrupted_returns_none, unsupported_version), `lock_lifecycle_tests.rs` (TM-P27-007) | done |
| LCK-007 | `SPEC_LOCK.md` 5.5 | `plan` MUST show `Remove` actions from lock diff | `eden-skills-cli/src/commands.rs` (`plan` reads lock, `build_remove_plan_items`) | `lock_diff_tests.rs` (TM-P27-005) | done |
| LCK-008 | `SPEC_LOCK.md` 5.4 | Unchanged skills MAY skip source sync | `eden-skills-cli/src/commands.rs` (`filter_config_for_sync`) | `lock_diff_tests.rs` (TM-P27-011) | done |
| LCK-009 | `SPEC_LOCK.md` 3.3 | Lock entries MUST be sorted alphabetically by id | `eden-skills-core/src/lock.rs` (`write_lock_file` sort) | `lock_tests.rs` (sorted_by_id, sorted_by_agent), `lock_lifecycle_tests.rs` (TM-P27-009) | done |
| LCK-010 | `SPEC_LOCK.md` 3.2 | `resolved_commit` SHOULD record full SHA-1 | `eden-skills-cli/src/commands.rs` (`collect_resolved_commits`, `write_lock_for_config_with_commits`) | `lock_diff_tests.rs` (TM-P27-010) | done |

## 2. Help System Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
|---|---|---|---|---|---|
| HLP-001 | `SPEC_HELP_SYSTEM.md` 2.1 | Root CLI MUST support `--version` / `-V` | `eden-skills-cli/src/lib.rs` (`#[command(version)]`) | `help_system_tests::version_flag_and_short_alias_print_package_version` | done |
| HLP-002 | `SPEC_HELP_SYSTEM.md` 3 | Root `--help` MUST show version, about, groups, examples | `eden-skills-cli/src/lib.rs` (`about`, `long_about`, `after_help`, `before_help`) | `help_system_tests::root_help_contains_version_about_groups_and_examples` | done |
| HLP-003 | `SPEC_HELP_SYSTEM.md` 4 | Every subcommand MUST have an `about` description | `eden-skills-cli/src/lib.rs` (`#[command(about = "...")]` on Commands and ConfigSubcommand) | `help_system_tests::subcommands_include_normative_about_descriptions` | done |
| HLP-004 | `SPEC_HELP_SYSTEM.md` 5 | Every argument MUST have a `help` annotation | `eden-skills-cli/src/lib.rs` (`#[arg(help = "...")]` on all args) | `help_system_tests::install_help_shows_argument_descriptions` | done |
| HLP-005 | `SPEC_HELP_SYSTEM.md` 3.2 | Commands MUST be grouped with headings | `eden-skills-cli/src/lib.rs` (`next_help_heading` on subcommands, grouped text in `after_help`) | `help_system_tests::root_help_contains_version_about_groups_and_examples` | done |
| HLP-006 | `SPEC_HELP_SYSTEM.md` 6 | Short flags `-s`, `-t`, `-y`, `-V` MUST be available | `eden-skills-cli/src/lib.rs` (`short = 's'`, `short = 't'`, `short = 'y'`, `-V` via version) | `help_system_tests::short_flags_are_accepted_for_install_and_root_version` | done |
| HLP-007 | `SPEC_HELP_SYSTEM.md` 5.2 | `install` MUST accept `--copy` flag | `eden-skills-cli/src/lib.rs` (InstallArgs.copy), `commands.rs` (InstallRequest.copy, requested_install_mode, upsert_mode_a/b) | `help_system_tests::install_copy_flag_persists_copy_mode_and_copy_verify_defaults` | done |

## 3. Output Polish Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
|---|---|---|---|---|---|
| OUT-001 | `SPEC_OUTPUT_POLISH.md` 4.1 | All hardcoded ANSI MUST be replaced with `owo-colors` | | | pending |
| OUT-002 | `SPEC_OUTPUT_POLISH.md` 4.3 | `console` crate MUST be removed as direct dependency | | | pending |
| OUT-003 | `SPEC_OUTPUT_POLISH.md` 3 | Root CLI MUST accept `--color auto\|always\|never` | | | pending |
| OUT-004 | `SPEC_OUTPUT_POLISH.md` 5.1 | Error output MUST use formatted `error:` prefix with hint | | | pending |
| OUT-005 | `SPEC_OUTPUT_POLISH.md` 5.2 | IO errors MUST include contextual path and hint | | | pending |
| OUT-006 | `SPEC_OUTPUT_POLISH.md` 3.4 | Windows MUST call `enable_ansi_support` | | | pending |
| OUT-007 | `SPEC_OUTPUT_POLISH.md` 2.3 | Color palette MUST be limited to 12 standard ANSI colors | | | pending |
| OUT-008 | `SPEC_OUTPUT_POLISH.md` 5.4 | Pre-flight checks SHOULD detect missing git/docker | | | pending |

## 4. Remove Enhancement Requirements

| REQ_ID | Source | Requirement | Implementation | Tests | Status |
|---|---|---|---|---|---|
| RMV-001 | `SPEC_REMOVE_ENH.md` 2.1 | `remove` MUST accept multiple positional skill IDs | | | pending |
| RMV-002 | `SPEC_REMOVE_ENH.md` 2.1 | Unknown IDs in batch remove MUST fail atomically | | | pending |
| RMV-003 | `SPEC_REMOVE_ENH.md` 3.1 | `remove` with no args on TTY MUST enter interactive mode | | | pending |
| RMV-004 | `SPEC_REMOVE_ENH.md` 3.2 | `remove` with no args on non-TTY MUST fail | | | pending |
| RMV-005 | `SPEC_REMOVE_ENH.md` 4 | `-y`/`--yes` MUST skip confirmation on `remove` and `install` | | | pending |

## 5. Test Matrix Coverage

| SCENARIO_ID | Source | Scenario | Automated Test | Status |
|---|---|---|---|---|
| TM-P27-001 | `SPEC_TEST_MATRIX.md` 2 | Lock file creation on first apply | `lock_lifecycle_tests::apply_creates_lock_on_first_run` | done |
| TM-P27-002 | `SPEC_TEST_MATRIX.md` 2 | Lock file updated after install | `lock_lifecycle_tests::install_creates_lock_file` | done |
| TM-P27-003 | `SPEC_TEST_MATRIX.md` 2 | Lock file updated after remove | `lock_lifecycle_tests::remove_updates_lock_file` | done |
| TM-P27-004 | `SPEC_TEST_MATRIX.md` 2 | Orphan removal via apply | `lock_diff_tests::apply_removes_orphaned_skill_from_lock` | done |
| TM-P27-005 | `SPEC_TEST_MATRIX.md` 2 | Plan shows remove actions | `lock_diff_tests::plan_shows_remove_actions_for_orphans`, `lock_diff_tests::plan_json_includes_remove_action` | done |
| TM-P27-006 | `SPEC_TEST_MATRIX.md` 2 | Missing lock file fallback | `lock_lifecycle_tests::apply_succeeds_without_existing_lock_file` | done |
| TM-P27-007 | `SPEC_TEST_MATRIX.md` 2 | Corrupted lock file recovery | `lock_lifecycle_tests::apply_recovers_from_corrupted_lock` | done |
| TM-P27-008 | `SPEC_TEST_MATRIX.md` 2 | Lock co-location with custom config | `lock_lifecycle_tests::lock_co_located_with_custom_config_path` | done |
| TM-P27-009 | `SPEC_TEST_MATRIX.md` 2 | Lock entries sorted alphabetically | `lock_lifecycle_tests::lock_entries_sorted_after_apply` | done |
| TM-P27-010 | `SPEC_TEST_MATRIX.md` 2 | Lock preserves resolved commit | `lock_diff_tests::lock_records_resolved_commit_after_apply` | done |
| TM-P27-011 | `SPEC_TEST_MATRIX.md` 2 | Apply noop optimization | `lock_diff_tests::apply_noop_with_unchanged_config` | done |
| TM-P27-012 | `SPEC_TEST_MATRIX.md` 2 | Lock init creates empty lock | `lock_lifecycle_tests::init_creates_empty_lock_file` | done |
| TM-P27-013 | `SPEC_TEST_MATRIX.md` 2 | Repair updates lock | `lock_lifecycle_tests::repair_updates_lock_file_after_fixing_broken_symlink` | done |
| TM-P27-014 | `SPEC_TEST_MATRIX.md` 2 | Apply remove with Docker target | `lock_diff_tests::apply_removes_orphaned_docker_target_from_lock` | done |
| TM-P27-015 | `SPEC_TEST_MATRIX.md` 2 | Strict mode does not block removals | `lock_diff_tests::strict_mode_does_not_block_removals` | done |
| TM-P27-016 | `SPEC_TEST_MATRIX.md` 3 | Version flag | `help_system_tests::version_flag_and_short_alias_print_package_version` | done |
| TM-P27-017 | `SPEC_TEST_MATRIX.md` 3 | Root help contains version and groups | `help_system_tests::root_help_contains_version_about_groups_and_examples` | done |
| TM-P27-018 | `SPEC_TEST_MATRIX.md` 3 | Subcommand help has description | `help_system_tests::subcommands_include_normative_about_descriptions` | done |
| TM-P27-019 | `SPEC_TEST_MATRIX.md` 3 | Argument help has description | `help_system_tests::install_help_shows_argument_descriptions` | done |
| TM-P27-020 | `SPEC_TEST_MATRIX.md` 3 | Short flags work | `help_system_tests::short_flags_are_accepted_for_install_and_root_version` | done |
| TM-P27-021 | `SPEC_TEST_MATRIX.md` 3 | Install copy flag | `help_system_tests::install_copy_flag_persists_copy_mode_and_copy_verify_defaults` | done |
| TM-P27-022 | `SPEC_TEST_MATRIX.md` 4 | No hardcoded ANSI in source | | pending |
| TM-P27-023 | `SPEC_TEST_MATRIX.md` 4 | Console crate removed | | pending |
| TM-P27-024 | `SPEC_TEST_MATRIX.md` 4 | Color flag auto | | pending |
| TM-P27-025 | `SPEC_TEST_MATRIX.md` 4 | Color flag never | | pending |
| TM-P27-026 | `SPEC_TEST_MATRIX.md` 4 | Color flag always | | pending |
| TM-P27-027 | `SPEC_TEST_MATRIX.md` 4 | Error format with hint | | pending |
| TM-P27-028 | `SPEC_TEST_MATRIX.md` 4 | Error context for missing config | | pending |
| TM-P27-029 | `SPEC_TEST_MATRIX.md` 4 | Error context for unknown skill | | pending |
| TM-P27-030 | `SPEC_TEST_MATRIX.md` 4 | Windows ANSI support | | pending |
| TM-P27-031 | `SPEC_TEST_MATRIX.md` 4 | JSON mode unaffected | | pending |
| TM-P27-032 | `SPEC_TEST_MATRIX.md` 5 | Batch remove multiple skills | | pending |
| TM-P27-033 | `SPEC_TEST_MATRIX.md` 5 | Batch remove atomic validation | | pending |
| TM-P27-034 | `SPEC_TEST_MATRIX.md` 5 | Interactive remove on TTY | | pending |
| TM-P27-035 | `SPEC_TEST_MATRIX.md` 5 | Non-TTY remove without args fails | | pending |
| TM-P27-036 | `SPEC_TEST_MATRIX.md` 5 | Remove yes flag skips prompt | | pending |
| TM-P27-037 | `SPEC_TEST_MATRIX.md` 5 | Install yes flag skips prompt | | pending |
| TM-P27-038 | `SPEC_TEST_MATRIX.md` 5 | Remove empty config | | pending |
| TM-P27-039 | `SPEC_TEST_MATRIX.md` 5 | Batch remove JSON output | | pending |
| TM-P27-040 | `SPEC_TEST_MATRIX.md` 6 | Full regression | | pending |
