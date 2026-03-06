# SPEC_INSTALL_SCRIPT.md

Cross-platform install scripts and `cargo-binstall` metadata.

**Related contracts:**

- `spec/phase2.5/SPEC_DISTRIBUTION.md` (binary distribution via GitHub Releases)

## 1. Problem Statement

Installing eden-skills currently requires either the Rust toolchain
(`cargo install eden-skills`) or manually downloading a binary from
GitHub Releases. Neither path is beginner-friendly.

## 2. Install Scripts

### 2.1 `install.sh` — Linux and macOS

**Invocation:**

```bash
curl -fsSL https://raw.githubusercontent.com/AI-Eden/eden-skills/main/install.sh | bash
```

**Script behavior:**

1. **Platform detection:** `uname -s` (OS) + `uname -m` (arch).
   Map to Rust target triple:

   | OS | Arch | Target |
   | :--- | :--- | :--- |
   | Linux | x86_64 | `x86_64-unknown-linux-gnu` |
   | Linux | aarch64 / arm64 | `aarch64-unknown-linux-gnu` |
   | Darwin | x86_64 | `x86_64-apple-darwin` |
   | Darwin | arm64 | `aarch64-apple-darwin` |

   Unsupported combinations MUST produce a clear error and exit 1.

2. **Version resolution:** If `EDEN_SKILLS_VERSION` is set, use it.
   Otherwise, query the GitHub API for the latest release tag:
   `GET https://api.github.com/repos/AI-Eden/eden-skills/releases/latest`.

3. **Download:** Fetch the `.tar.gz` archive and checksums file from
   the release assets URL.

4. **Integrity verification:** Compute SHA-256 of the downloaded
   archive and compare against the checksums file. Abort on mismatch.

5. **Extraction:** Extract the `eden-skills` binary from the archive.

6. **Installation:** Place the binary in `EDEN_SKILLS_INSTALL_DIR`
   (default: `~/.eden-skills/bin/`). Create the directory if needed.

7. **PATH check:** If the install directory is not in `$PATH`, `install.sh`
   MUST select the appropriate shell rc file (`.zshrc`, `.bashrc`, or
   `.profile`) based on `$SHELL` and check whether the PATH export is
   already configured there.
   - If the PATH export is already present, do not append a duplicate.
   - If it is missing, append the export line automatically:

     ```sh
     export PATH="$HOME/.eden-skills/bin:$PATH"
     ```

   - After detection or update, print a note explaining how to reload the
     shell config so the new PATH takes effect immediately.
   - If the rc file cannot be updated automatically, print the same export
     command as a manual fallback.

8. **Prerequisite check:** Verify `git` is available. If not, print
   a warning (eden-skills requires git for source operations).

9. **Verification:** Run `eden-skills --version` and print success.

**Error handling:** Every step MUST check its exit code. Failures
MUST produce a human-readable error message and exit 1.

**Idempotency:** Re-running the script MUST succeed (overwrite the
existing binary).

### 2.2 `install.ps1` — Windows

**Invocation:**

```powershell
irm https://raw.githubusercontent.com/AI-Eden/eden-skills/main/install.ps1 | iex
```

**Script behavior:**

1. **Architecture detection:** `[System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture`.
   Currently only `x86_64-pc-windows-msvc` is built.

2. **Version resolution:** Same as `install.sh` — `EDEN_SKILLS_VERSION`
   env var or GitHub API latest release.

3. **Download:** Fetch the `.zip` archive and checksums file using
   `Invoke-WebRequest`.

4. **Integrity verification:** `Get-FileHash -Algorithm SHA256` on
   the downloaded archive; compare against checksums. Abort on
   mismatch.

5. **Extraction:** `Expand-Archive` the `.zip` file.

6. **Installation:** Place `eden-skills.exe` in
   `$env:EDEN_SKILLS_INSTALL_DIR` (default:
   `$env:USERPROFILE\.eden-skills\bin\`).

7. **PATH update:** Add the install directory to the user-level `Path`
   environment variable via `[Environment]::SetEnvironmentVariable`.
   Print a note that new terminal sessions will pick up the change.

8. **Prerequisite check:** Verify `git` is available.

9. **Verification:** Run `eden-skills --version`.

### 2.3 Script Location

Both scripts MUST be placed at the repository root:

- `install.sh` — POSIX shell (no bashisms; compatible with
  `/bin/sh`, `bash`, `zsh`).
- `install.ps1` — PowerShell 5.1+ compatible.

## 3. `cargo-binstall` Metadata

### 3.1 Purpose

`cargo binstall eden-skills` downloads a pre-built binary from GitHub
Releases instead of compiling from source.

### 3.2 Configuration

Add to `crates/eden-skills-cli/Cargo.toml`:

```toml
[package.metadata.binstall]
pkg-url = "{ repo }/releases/download/v{ version }/eden-skills-{ version }-{ target }.{ archive-suffix }"
bin-dir = "{ bin }{ binary-ext }"
pkg-fmt = "tgz"

[package.metadata.binstall.overrides.x86_64-pc-windows-msvc]
pkg-fmt = "zip"
```

### 3.3 Template Variables

| Variable | Example Value |
| :--- | :--- |
| `{ repo }` | `https://github.com/AI-Eden/eden-skills` |
| `{ version }` | `0.5.0` |
| `{ target }` | `x86_64-unknown-linux-gnu` |
| `{ archive-suffix }` | `tar.gz` (or `zip` for Windows) |
| `{ bin }` | `eden-skills` |
| `{ binary-ext }` | `.exe` on Windows, empty otherwise |

## 4. Documentation Updates

### 4.1 README.md

The Install section MUST be updated to show the one-liner as the
primary install method, with `cargo install` as an alternative:

```markdown
## Install

**Linux / macOS:**

\`\`\`bash
curl -fsSL https://raw.githubusercontent.com/AI-Eden/eden-skills/main/install.sh | bash
\`\`\`

**Windows (PowerShell):**

\`\`\`powershell
irm https://raw.githubusercontent.com/AI-Eden/eden-skills/main/install.ps1 | iex
\`\`\`

**Alternative: cargo install**

\`\`\`bash
cargo install eden-skills --locked
\`\`\`
```

### 4.2 docs/01-quickstart.md

Update the prerequisites section to reflect the new install method.

## 5. Normative Requirements

| ID | Owner | Priority | Statement | Verification |
| :--- | :--- | :--- | :--- | :--- |
| **ISC-001** | Builder | **P0** | `install.sh` MUST detect OS and architecture, download the correct binary, verify SHA-256, and install to `~/.eden-skills/bin/`. | Script runs successfully on Linux x86_64 and macOS arm64. |
| **ISC-002** | Builder | **P0** | `install.ps1` MUST detect architecture, download the correct binary, verify SHA-256, and install to `$USERPROFILE\.eden-skills\bin\`. | Script runs successfully on Windows x86_64. |
| **ISC-003** | Builder | **P0** | Platform detection MUST map to correct Rust target triples per Section 2.1. | Unsupported platform produces clear error. |
| **ISC-004** | Builder | **P0** | SHA-256 integrity verification MUST abort on mismatch. | Tampered archive produces error and non-zero exit. |
| **ISC-005** | Builder | **P1** | Both scripts MUST ensure PATH is configured when needed. `install.sh` MUST use the shell-selected rc file without duplicating entries and print reload guidance; `install.ps1` MUST continue to update the user-level `Path`. | `install.sh` updates the expected rc file or prints fallback guidance; `install.ps1` updates the user `Path`. |
| **ISC-006** | Builder | **P1** | `cargo-binstall` metadata MUST be added to `Cargo.toml`. | `cargo binstall eden-skills` downloads pre-built binary. |
| **ISC-007** | Builder | **P2** | Both scripts MUST support `EDEN_SKILLS_VERSION` env var for version pinning. | Setting env var installs specified version. |

## 6. Backward Compatibility

| Existing Feature | Phase 2.95 Behavior |
| :--- | :--- |
| `cargo install eden-skills` | Unchanged. Still works as alternative. |
| GitHub Releases artifacts | Unchanged. Same archives, same checksums. |
| `release.yml` workflow | No changes required. |
