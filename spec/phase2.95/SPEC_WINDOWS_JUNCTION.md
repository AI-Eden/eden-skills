# SPEC_WINDOWS_JUNCTION.md

Windows NTFS junction point fallback for symlink-mode installs.

**Related contracts:**

- `spec/phase2/SPEC_ADAPTER.md` (LocalAdapter, symlink creation, Windows portability)
- `spec/phase2.5/SPEC_CLI_UX.md` (UX-008: Windows hardcopy fallback)

## 1. Problem Statement

On Windows, creating symbolic links requires either Developer Mode
enabled or administrator privileges (`SeCreateSymbolicLinkPrivilege`).
When symlinks are unavailable, the current fallback is hard copy
(`InstallMode::Copy`), which duplicates all files and defeats the
benefits of linking (instant updates, no disk waste).

NTFS junction points (directory junctions) are an alternative that:

- Do NOT require administrator privileges.
- Support directories (skills are always directories).
- Have negligible access-time overhead (same NTFS reparse mechanism
  as symlinks).
- Are transparent to virtually all applications.

## 2. Three-Level Fallback Chain

On Windows, the install mode decision MUST follow this chain:

```text
1. Symlink (std::os::windows::fs::symlink_dir)
   ↓ fails (PermissionDenied)
2. Junction (junction::create)
   ↓ fails (e.g., cross-volume)
3. Hard Copy (recursive file copy)
```

On non-Windows platforms, the chain is unchanged: symlink or copy.

### 2.1 Decision Logic

The `resolve_default_install_mode_decision()` function in `install.rs`
MUST be extended:

```rust
#[cfg(windows)]
fn resolve_default_install_mode_decision() -> DefaultInstallModeDecision {
    if windows_supports_symlink_creation() {
        return DefaultInstallModeDecision { mode: Symlink, ..default };
    }
    if windows_supports_junction_creation() {
        return DefaultInstallModeDecision { mode: Symlink, use_junction: true, ..default };
    }
    DefaultInstallModeDecision { mode: Copy, warn_windows_hardcopy_fallback: true }
}
```

The junction probe follows the same pattern as the symlink probe:
create a junction in a temporary directory, verify it exists, then
clean up.

### 2.2 Warning Message

When junction is used as fallback, a warning MUST be emitted:

```text
  ⚠ Windows symlink permission unavailable; using NTFS junction
    (functionally equivalent, no admin required).
```

When junction also fails and copy is used:

```text
  ⚠ Windows symlink and junction unavailable; falling back to
    hardcopy mode (this may slow down installs).
```

## 3. Transparent Implementation

### 3.1 No New InstallMode Variant

Junction MUST NOT be exposed as a separate `InstallMode` enum variant.
To users and to `skills.toml` / `skills.lock`, junction installs are
recorded as `mode = "symlink"`. The junction is an internal
implementation detail of the symlink mode on Windows.

### 3.2 Adapter Layer

In `adapter.rs`, `create_symlink()` MUST be extended on Windows:

```rust
#[cfg(windows)]
fn create_symlink(source: &Path, target: &Path, _is_dir: bool) -> Result<(), AdapterError> {
    match std::os::windows::fs::symlink_dir(source, target) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => {
            junction::create(source, target)
                .map_err(|e| AdapterError::Runtime {
                    detail: format!("junction fallback failed: {e}"),
                })
        }
        Err(err) => Err(map_windows_symlink_error(err, source, target)),
    }
}
```

Similarly, `apply_symlink()` in `common.rs` MUST follow the same
fallback pattern.

### 3.3 Removal

`remove_existing_path()` in `adapter.rs` and `remove_symlink_path()`
in `common.rs` MUST handle junction removal on Windows:

```rust
#[cfg(windows)]
async fn remove_symlink_or_junction(path: &Path) -> Result<(), AdapterError> {
    if junction::exists(path).unwrap_or(false) {
        junction::delete(path).map_err(|e| ...)?;
        return Ok(());
    }
    remove_symlink(path).await
}
```

## 4. Plan Detection

### 4.1 Problem

In `plan.rs`, `determine_action()` checks
`metadata.file_type().is_symlink()` to determine if a target is a
valid symlink. On Windows, `is_symlink()` returns `false` for
junction points (they use `IO_REPARSE_TAG_MOUNT_POINT`, not
`IO_REPARSE_TAG_SYMLINK`).

### 4.2 Solution

On Windows, when `install_mode == Symlink` and `is_symlink()` returns
false, the plan MUST additionally check `junction::exists(target_path)`.
If the target is a junction, it MUST be treated equivalently to a
symlink for all plan decisions:

- Read the junction target via `junction::get_target()`.
- Compare against the expected source path.
- Return `Noop` if targets match, `Update` if they differ.

```rust
#[cfg(windows)]
fn is_symlink_or_junction(metadata: &Metadata, path: &Path) -> bool {
    metadata.file_type().is_symlink() || junction::exists(path).unwrap_or(false)
}

#[cfg(not(windows))]
fn is_symlink_or_junction(metadata: &Metadata, _path: &Path) -> bool {
    metadata.file_type().is_symlink()
}
```

### 4.3 Target Reading

`read_symlink_target()` in `plan.rs` uses `fs::read_link()`. On
Windows, `fs::read_link()` works for both symlinks and junctions
(it reads the reparse point target). No change is needed for target
reading.

## 5. Dependency

The `junction` crate MUST be added as a Windows-only dependency:

```toml
[target.'cfg(windows)'.dependencies]
junction = "1"
```

The crate is pure Rust, uses `windows-sys` internally, and has no
transitive C dependencies.

## 6. Normative Requirements

| ID | Owner | Priority | Statement | Verification |
| :--- | :--- | :--- | :--- | :--- |
| **WJN-001** | Builder | **P0** | Windows install MUST follow the three-level fallback: symlink → junction → copy. | Probe logic tests all three levels. |
| **WJN-002** | Builder | **P0** | `junction` crate MUST be added as `cfg(windows)` dependency. | `Cargo.toml` updated; compiles on all platforms. |
| **WJN-003** | Builder | **P0** | Junction MUST NOT be exposed as a new `InstallMode` variant; recorded as `mode = "symlink"`. | `skills.toml` and `skills.lock` show `symlink` for junction installs. |
| **WJN-004** | Builder | **P0** | `plan.rs` MUST detect junction reparse points as valid symlink-mode targets on Windows. | `determine_action` returns `Noop`/`Update` (not `Conflict`) for junction targets. |
| **WJN-005** | Builder | **P0** | `adapter.rs` and `common.rs` MUST handle junction creation and removal on Windows. | Junction created when symlink fails; junction removed before reinstall. |
| **WJN-006** | Builder | **P1** | Junction probe MUST be added to `resolve_default_install_mode_decision`. | Probe creates and verifies a junction in temp directory. |

## 7. Backward Compatibility

| Existing Feature | Phase 2.95 Behavior |
| :--- | :--- |
| `--copy` flag | Unchanged. Explicit copy bypasses the fallback chain. |
| Existing symlink installs on Windows | Unchanged. Symlinks continue to work when privileges are available. |
| `skills.lock` format | Unchanged. Junction installs recorded as `mode = "symlink"`. |
| Linux / macOS | Unchanged. Junction code is `cfg(windows)` only. |
| `doctor` / `verify` | Junctions pass the same checks as symlinks (existence + target match). |
