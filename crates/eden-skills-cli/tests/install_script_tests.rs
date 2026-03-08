mod common;

use std::fs;
use std::path::PathBuf;

use toml::Value;

#[cfg(not(windows))]
use std::os::unix::fs::PermissionsExt;
#[cfg(not(windows))]
use std::path::Path;
#[cfg(not(windows))]
use std::process::{Command, Output};
#[cfg(not(windows))]
use tempfile::tempdir;

#[cfg(not(windows))]
struct UnixReleaseFixture {
    api_url: String,
    release_base_url: String,
}

#[cfg(not(windows))]
#[test]
fn tm_p295_001_install_sh_detects_linux_x86_64_and_downloads_correct_archive() {
    let temp = tempdir().expect("tempdir");
    let fixture = write_unix_release_fixture(temp.path(), "0.5.0", "x86_64-unknown-linux-gnu");
    let home_dir = temp.path().join("home");
    let install_dir = home_dir.join(".eden-skills/bin");
    fs::create_dir_all(&home_dir).expect("create home");

    let output = run_install_sh(
        temp.path(),
        &home_dir,
        &install_dir,
        &fixture,
        Some("0.5.0"),
        "Linux",
        "x86_64",
        "/bin/bash",
        &format!("{}:/usr/bin:/bin", install_dir.display()),
    );

    assert_success(&output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("x86_64-unknown-linux-gnu"),
        "expected Linux x86_64 target triple in stdout={stdout}"
    );
    assert!(
        stdout.contains("Installed eden-skills 0.5.0"),
        "expected success line in stdout={stdout}"
    );
    assert_eq!(
        fs::read_to_string(install_dir.join("eden-skills")).expect("read installed binary"),
        "#!/bin/sh\nprintf 'eden-skills 0.5.0\\n'\n",
        "expected correct archive contents to be installed"
    );
}

#[cfg(not(windows))]
#[test]
fn tm_p295_002_install_sh_detects_macos_arm64_and_downloads_correct_archive() {
    let temp = tempdir().expect("tempdir");
    let fixture = write_unix_release_fixture(temp.path(), "0.6.0", "aarch64-apple-darwin");
    let home_dir = temp.path().join("home");
    let install_dir = home_dir.join(".eden-skills/bin");
    fs::create_dir_all(&home_dir).expect("create home");

    let output = run_install_sh(
        temp.path(),
        &home_dir,
        &install_dir,
        &fixture,
        Some("0.6.0"),
        "Darwin",
        "arm64",
        "/bin/zsh",
        &format!("{}:/usr/bin:/bin", install_dir.display()),
    );

    assert_success(&output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("aarch64-apple-darwin"),
        "expected macOS arm64 target triple in stdout={stdout}"
    );
    assert!(
        stdout.contains("Installed eden-skills 0.6.0"),
        "expected success line in stdout={stdout}"
    );
}

#[cfg(not(windows))]
#[test]
fn tm_p295_003_install_sh_aborts_on_unsupported_platform_with_clear_error() {
    let temp = tempdir().expect("tempdir");
    let fixture = write_unix_release_fixture(temp.path(), "0.7.0", "x86_64-unknown-linux-gnu");
    let home_dir = temp.path().join("home");
    let install_dir = home_dir.join(".eden-skills/bin");
    fs::create_dir_all(&home_dir).expect("create home");

    let output = run_install_sh(
        temp.path(),
        &home_dir,
        &install_dir,
        &fixture,
        Some("0.7.0"),
        "FreeBSD",
        "x86_64",
        "/bin/bash",
        "/usr/bin:/bin",
    );

    assert_failure(&output);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Unsupported platform"),
        "expected unsupported platform error, stderr={stderr}"
    );
    assert!(
        stderr.contains("FreeBSD") && stderr.contains("x86_64"),
        "expected platform details in stderr={stderr}"
    );
}

#[cfg(not(windows))]
#[test]
fn tm_p295_004_install_sh_aborts_on_sha256_mismatch() {
    let temp = tempdir().expect("tempdir");
    let fixture = write_unix_release_fixture(temp.path(), "0.8.0", "x86_64-unknown-linux-gnu");
    let checksum_path = temp
        .path()
        .join("release-root/download/v0.8.0/eden-skills-0.8.0-checksums.txt");
    fs::write(
        &checksum_path,
        "0000000000000000000000000000000000000000000000000000000000000000  eden-skills-0.8.0-x86_64-unknown-linux-gnu.tar.gz\n",
    )
    .expect("overwrite checksum");

    let home_dir = temp.path().join("home");
    let install_dir = home_dir.join(".eden-skills/bin");
    fs::create_dir_all(&home_dir).expect("create home");

    let output = run_install_sh(
        temp.path(),
        &home_dir,
        &install_dir,
        &fixture,
        Some("0.8.0"),
        "Linux",
        "x86_64",
        "/bin/bash",
        "/usr/bin:/bin",
    );

    assert_failure(&output);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("SHA-256 mismatch"),
        "expected checksum mismatch error, stderr={stderr}"
    );
}

#[cfg(not(windows))]
#[test]
fn tm_p295_005_install_sh_updates_selected_shell_rc_when_dir_not_in_path() {
    let temp = tempdir().expect("tempdir");
    let fixture = write_unix_release_fixture(temp.path(), "0.9.0", "x86_64-unknown-linux-gnu");
    let home_dir = temp.path().join("home");
    let install_dir = home_dir.join(".eden-skills/bin");
    let rc_path = home_dir.join(".zshrc");
    fs::create_dir_all(&home_dir).expect("create home");

    let output = run_install_sh(
        temp.path(),
        &home_dir,
        &install_dir,
        &fixture,
        Some("0.9.0"),
        "Linux",
        "x86_64",
        "/bin/zsh",
        "/usr/bin:/bin",
    );

    assert_success(&output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let rc_contents = fs::read_to_string(&rc_path).expect("read rc file");
    assert!(
        stdout.contains("Added") && stdout.contains("~/.zshrc"),
        "expected installer to report rc update, stdout={stdout}"
    );
    assert!(
        stdout.contains(". ~/.zshrc"),
        "expected reload hint in stdout={stdout}"
    );
    assert!(
        rc_contents.contains("export PATH=\"$HOME/.eden-skills/bin:$PATH\""),
        "expected PATH export in rc file, contents={rc_contents}"
    );
    assert_eq!(
        rc_contents
            .matches("export PATH=\"$HOME/.eden-skills/bin:$PATH\"")
            .count(),
        1,
        "expected PATH export to be appended exactly once, contents={rc_contents}"
    );
}

#[test]
fn tm_p295_006_install_ps1_declares_windows_x86_64_install_flow() {
    let script = fs::read_to_string(install_ps1_path()).expect("read install.ps1");
    for required in [
        "RuntimeInformation]::OSArchitecture",
        "x86_64-pc-windows-msvc",
        "Invoke-WebRequest",
        "Expand-Archive",
        "$env:USERPROFILE",
        "$env:EDEN_SKILLS_VERSION",
        "SetEnvironmentVariable",
        "eden-skills.exe",
        "--version",
    ] {
        assert!(
            script.contains(required),
            "install.ps1 missing required snippet `{required}`"
        );
    }
}

#[test]
fn tm_p295_007_install_ps1_declares_sha256_mismatch_guard() {
    let script = fs::read_to_string(install_ps1_path()).expect("read install.ps1");
    for required in [
        "Get-FileHash -Algorithm SHA256",
        "SHA-256 mismatch",
        "checksums",
        "Fail ",
    ] {
        assert!(
            script.contains(required),
            "install.ps1 missing SHA-256 verification snippet `{required}`"
        );
    }
}

#[cfg(not(windows))]
#[test]
fn install_sh_does_not_duplicate_existing_path_entry_in_shell_rc() {
    let temp = tempdir().expect("tempdir");
    let fixture = write_unix_release_fixture(temp.path(), "0.9.1", "x86_64-unknown-linux-gnu");
    let home_dir = temp.path().join("home");
    let install_dir = home_dir.join(".eden-skills/bin");
    let rc_path = home_dir.join(".zshrc");
    fs::create_dir_all(&home_dir).expect("create home");
    fs::write(
        &rc_path,
        "# existing config\nexport PATH=\"$HOME/.eden-skills/bin:$PATH\"\n",
    )
    .expect("seed rc file");

    let output = run_install_sh(
        temp.path(),
        &home_dir,
        &install_dir,
        &fixture,
        Some("0.9.1"),
        "Linux",
        "x86_64",
        "/bin/zsh",
        "/usr/bin:/bin",
    );

    assert_success(&output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let rc_contents = fs::read_to_string(&rc_path).expect("read rc file");
    assert!(
        stdout.contains("already appears to be configured in ~/.zshrc"),
        "expected existing-config message, stdout={stdout}"
    );
    assert_eq!(
        rc_contents
            .matches("export PATH=\"$HOME/.eden-skills/bin:$PATH\"")
            .count(),
        1,
        "expected installer to avoid duplicate PATH exports, contents={rc_contents}"
    );
}

#[cfg(not(windows))]
#[test]
fn tm_p295_008_eden_skills_version_env_var_pins_install_to_specific_version() {
    let temp = tempdir().expect("tempdir");
    let fixture = write_unix_release_fixture(temp.path(), "1.2.3", "x86_64-unknown-linux-gnu");
    let home_dir = temp.path().join("home");
    let install_dir = home_dir.join(".eden-skills/bin");
    fs::create_dir_all(&home_dir).expect("create home");

    let output = run_install_sh(
        temp.path(),
        &home_dir,
        &install_dir,
        &fixture,
        Some("1.2.3"),
        "Linux",
        "x86_64",
        "/bin/bash",
        &format!("{}:/usr/bin:/bin", install_dir.display()),
    );

    assert_success(&output);
    let version_output = Command::new(install_dir.join("eden-skills"))
        .arg("--version")
        .output()
        .expect("run installed binary");
    assert_eq!(
        String::from_utf8_lossy(&version_output.stdout).trim(),
        "eden-skills 1.2.3",
        "expected pinned version binary to be installed"
    );
}

#[test]
fn tm_p295_009_cargo_binstall_metadata_matches_release_contract() {
    let manifest_path = workspace_root().join("crates/eden-skills-cli/Cargo.toml");
    let manifest_text = fs::read_to_string(&manifest_path).expect("read cli Cargo.toml");
    let manifest: Value = toml::from_str(&manifest_text).expect("parse cli Cargo.toml");

    let package = manifest
        .get("package")
        .and_then(Value::as_table)
        .expect("package table");
    let metadata = package
        .get("metadata")
        .and_then(Value::as_table)
        .expect("package.metadata");
    let binstall = metadata
        .get("binstall")
        .and_then(Value::as_table)
        .expect("package.metadata.binstall");

    assert_eq!(
        binstall.get("pkg-url").and_then(Value::as_str),
        Some("{ repo }/releases/download/v{ version }/eden-skills-{ version }-{ target }.{ archive-suffix }"),
        "pkg-url should match release archive contract"
    );
    assert_eq!(
        binstall.get("bin-dir").and_then(Value::as_str),
        Some("{ bin }{ binary-ext }"),
        "bin-dir should expose the binary directly"
    );
    assert_eq!(
        binstall.get("pkg-fmt").and_then(Value::as_str),
        Some("tgz"),
        "default package format should be tgz"
    );

    let overrides = binstall
        .get("overrides")
        .and_then(Value::as_table)
        .expect("binstall overrides table");
    let windows = overrides
        .get("x86_64-pc-windows-msvc")
        .and_then(Value::as_table)
        .expect("windows binstall override");
    assert_eq!(
        windows.get("pkg-fmt").and_then(Value::as_str),
        Some("zip"),
        "Windows package format should override to zip"
    );
}

#[cfg(not(windows))]
fn write_unix_release_fixture(base: &Path, version: &str, target: &str) -> UnixReleaseFixture {
    let release_root = base.join("release-root");
    let download_dir = release_root.join("download").join(format!("v{version}"));
    let package_dir = base.join("package");
    let archive_name = format!("eden-skills-{version}-{target}.tar.gz");
    let archive_path = download_dir.join(&archive_name);
    let checksum_path = download_dir.join(format!("eden-skills-{version}-checksums.txt"));
    let latest_json_path = release_root.join("latest.json");
    let binary_path = package_dir.join("eden-skills");

    fs::create_dir_all(&download_dir).expect("create download dir");
    fs::create_dir_all(&package_dir).expect("create package dir");
    fs::write(
        &binary_path,
        format!("#!/bin/sh\nprintf 'eden-skills {version}\\n'\n"),
    )
    .expect("write fake binary");

    let mut permissions = fs::metadata(&binary_path)
        .expect("binary metadata")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&binary_path, permissions).expect("mark fake binary executable");

    let tar_status = Command::new("tar")
        .args(["-czf"])
        .arg(&archive_path)
        .args(["-C"])
        .arg(&package_dir)
        .arg("eden-skills")
        .status()
        .expect("create tar.gz archive");
    assert!(tar_status.success(), "tar should succeed");

    let archive_hash = compute_sha256(&archive_path);
    fs::write(&checksum_path, format!("{archive_hash}  {archive_name}\n"))
        .expect("write checksum file");
    fs::write(
        &latest_json_path,
        format!("{{\"tag_name\":\"v{version}\"}}\n"),
    )
    .expect("write latest release json");

    UnixReleaseFixture {
        api_url: common::as_file_url(&latest_json_path),
        release_base_url: common::as_file_url(&release_root.join("download")),
    }
}

#[cfg(not(windows))]
fn compute_sha256(path: &Path) -> String {
    if let Some(hash) = read_hash_from_command("sha256sum", &[path.as_os_str()]) {
        return hash;
    }
    if let Some(hash) =
        read_hash_from_command("shasum", &["-a".as_ref(), "256".as_ref(), path.as_os_str()])
    {
        return hash;
    }

    let output = Command::new("openssl")
        .args(["dgst", "-sha256"])
        .arg(path)
        .output()
        .expect("run openssl dgst -sha256");
    assert!(
        output.status.success(),
        "openssl dgst -sha256 failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout)
        .split_whitespace()
        .last()
        .expect("openssl hash output")
        .to_string()
}

#[cfg(not(windows))]
fn read_hash_from_command(program: &str, args: &[impl AsRef<std::ffi::OsStr>]) -> Option<String> {
    let output = Command::new(program).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    Some(
        String::from_utf8_lossy(&output.stdout)
            .split_whitespace()
            .next()
            .expect("hash output")
            .to_string(),
    )
}

#[cfg(not(windows))]
#[allow(clippy::too_many_arguments)]
fn run_install_sh(
    cwd: &Path,
    home_dir: &Path,
    install_dir: &Path,
    fixture: &UnixReleaseFixture,
    version: Option<&str>,
    os_name: &str,
    arch: &str,
    shell_name: &str,
    path_value: &str,
) -> Output {
    let mut command = Command::new("sh");
    command
        .current_dir(cwd)
        .arg(install_sh_path())
        .env("HOME", home_dir)
        .env("PATH", path_value)
        .env("SHELL", shell_name)
        .env("EDEN_SKILLS_INSTALL_DIR", install_dir)
        .env("EDEN_SKILLS_RELEASE_API_URL", &fixture.api_url)
        .env("EDEN_SKILLS_RELEASE_BASE_URL", &fixture.release_base_url)
        .env("EDEN_SKILLS_TEST_UNAME_S", os_name)
        .env("EDEN_SKILLS_TEST_UNAME_M", arch);

    match version {
        Some(version) => {
            command.env("EDEN_SKILLS_VERSION", version);
        }
        None => {
            command.env_remove("EDEN_SKILLS_VERSION");
        }
    }

    command.output().expect("run install.sh")
}

fn install_ps1_path() -> PathBuf {
    workspace_root().join("install.ps1")
}

#[cfg(not(windows))]
fn install_sh_path() -> PathBuf {
    workspace_root().join("install.sh")
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace crates dir")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

#[cfg(not(windows))]
fn assert_success(output: &Output) {
    assert_eq!(
        output.status.code(),
        Some(0),
        "expected command success, stdout=`{}` stderr=`{}`",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[cfg(not(windows))]
fn assert_failure(output: &Output) {
    assert_ne!(
        output.status.code(),
        Some(0),
        "expected command failure, stdout=`{}` stderr=`{}`",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
