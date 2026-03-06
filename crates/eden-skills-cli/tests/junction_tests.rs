use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use eden_skills_core::lock::{lock_path_for_config, read_lock_file};
use tempfile::TempDir;

const JUNCTION_WARNING_SUBSTR: &str = "using NTFS junction";
const HARDCOPY_WARNING_SUBSTR: &str = "falling back to hardcopy mode";

#[test]
fn windows_symlink_available_uses_symlink_without_fallback_warning() {
    let fixture = InstallFixture::new("tm-p295-016");
    let output = fixture.run_install("1", Some("1"));
    assert_success(&output);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains(JUNCTION_WARNING_SUBSTR),
        "junction warning should not be emitted when symlink creation is available, stderr={stderr}"
    );
    assert!(
        !stderr.contains(HARDCOPY_WARNING_SUBSTR),
        "hardcopy warning should not be emitted when symlink creation is available, stderr={stderr}"
    );
    assert_eq!(
        fixture.config_install_mode(),
        "symlink",
        "expected symlink mode when Windows symlinks are available"
    );

    #[cfg(not(windows))]
    {
        let metadata = fs::symlink_metadata(fixture.target_path()).expect("target metadata");
        assert!(
            metadata.file_type().is_symlink(),
            "non-Windows test path should still install as a symlink"
        );
    }
}

#[test]
fn windows_symlink_unavailable_falls_back_to_junction_backed_symlink_mode() {
    let fixture = InstallFixture::new("tm-p295-017");
    let output = fixture.run_install("0", Some("1"));
    assert_success(&output);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(JUNCTION_WARNING_SUBSTR),
        "expected junction fallback warning, stderr={stderr}"
    );
    assert!(
        stderr.contains("functionally equivalent"),
        "junction fallback warning should explain behavioral parity, stderr={stderr}"
    );
    assert_eq!(
        fixture.config_install_mode(),
        "symlink",
        "junction fallback must remain transparent in config"
    );
}

#[test]
fn windows_symlink_and_junction_unavailable_fall_back_to_copy_mode() {
    let fixture = InstallFixture::new("tm-p295-018");
    let output = fixture.run_install("0", Some("0"));
    assert_success(&output);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("symlink and junction unavailable"),
        "hardcopy warning should mention both failed Windows link strategies, stderr={stderr}"
    );
    assert!(
        stderr.contains(HARDCOPY_WARNING_SUBSTR),
        "hardcopy warning should mention the copy fallback, stderr={stderr}"
    );
    assert!(
        stderr.contains("may slow down installs"),
        "hardcopy warning should mention performance impact, stderr={stderr}"
    );
    assert_eq!(
        fixture.config_install_mode(),
        "copy",
        "expected copy mode when both Windows link strategies are unavailable"
    );

    let metadata = fs::symlink_metadata(fixture.target_path()).expect("target metadata");
    assert!(
        !metadata.file_type().is_symlink(),
        "copy fallback should produce a real directory instead of a symlink"
    );
}

#[test]
fn junction_backed_install_is_recorded_as_symlink_in_lock_file() {
    let fixture = InstallFixture::new("tm-p295-019");
    let output = fixture.run_install("0", Some("1"));
    assert_success(&output);

    assert_eq!(
        fixture.config_install_mode(),
        "symlink",
        "config should continue to record transparent symlink mode"
    );
    assert_eq!(
        fixture.lock_install_mode(),
        "symlink",
        "lock file should record junction-backed installs as symlink mode"
    );
}

#[cfg(windows)]
#[test]
fn junction_probe_creates_and_cleans_up_temp_junction() {
    let fixture = InstallFixture::new("tm-p295-022");
    let probe_log = fixture.temp.path().join("junction-probe.log");

    let output = fixture.run_probe_install(&probe_log);
    assert_success(&output);

    let log = fs::read_to_string(&probe_log).expect("read probe log");
    assert!(
        log.contains("created=true"),
        "expected probe log to record junction creation, log={log}"
    );
    assert!(
        log.contains("cleaned=true"),
        "expected probe log to record junction cleanup, log={log}"
    );

    let probe_root = parse_logged_path(&log, "probe_root=").expect("probe root entry");
    let junction_path = parse_logged_path(&log, "junction_path=").expect("junction path entry");

    assert!(
        !Path::new(probe_root).exists(),
        "probe root should be removed after junction probe, path={probe_root}"
    );
    assert!(
        !Path::new(junction_path).exists(),
        "temporary probe junction should be removed after probing, path={junction_path}"
    );
}

struct InstallFixture {
    temp: TempDir,
    home_dir: PathBuf,
    source_dir: PathBuf,
    target_root: PathBuf,
    config_path: PathBuf,
}

impl InstallFixture {
    fn new(skill_name: &str) -> Self {
        let temp = tempfile::tempdir().expect("tempdir");
        let home_dir = temp.path().join("home");
        let source_dir = temp.path().join(skill_name);
        let target_root = temp.path().join("target-root");
        let config_path = temp.path().join("skills.toml");

        fs::create_dir_all(&home_dir).expect("create home dir");
        fs::create_dir_all(&source_dir).expect("create source dir");
        fs::create_dir_all(&target_root).expect("create target root");
        fs::write(source_dir.join("README.md"), "demo skill\n").expect("write source file");

        Self {
            temp,
            home_dir,
            source_dir,
            target_root,
            config_path,
        }
    }

    fn run_install(&self, symlink_supported: &str, junction_supported: Option<&str>) -> Output {
        let mut command = eden_command(&self.home_dir);
        command.current_dir(self.temp.path());
        command.env(
            "EDEN_SKILLS_TEST_WINDOWS_SYMLINK_SUPPORTED",
            symlink_supported,
        );
        match junction_supported {
            Some(value) => {
                command.env("EDEN_SKILLS_TEST_WINDOWS_JUNCTION_SUPPORTED", value);
            }
            None => {
                command.env_remove("EDEN_SKILLS_TEST_WINDOWS_JUNCTION_SUPPORTED");
            }
        }
        command.arg("install");
        command.arg(format!("./{}", self.skill_name()));
        command.arg("--target");
        command.arg(format!("custom:{}", self.target_root.display()));
        command.arg("--config");
        command.arg(&self.config_path);
        command.output().expect("run install")
    }

    #[cfg(windows)]
    fn run_probe_install(&self, probe_log: &Path) -> Output {
        let mut command = eden_command(&self.home_dir);
        command.current_dir(self.temp.path());
        command.env("EDEN_SKILLS_TEST_WINDOWS_SYMLINK_SUPPORTED", "0");
        command.env_remove("EDEN_SKILLS_TEST_WINDOWS_JUNCTION_SUPPORTED");
        command.env("EDEN_SKILLS_TEST_WINDOWS_JUNCTION_PROBE_LOG", probe_log);
        command.arg("install");
        command.arg(format!("./{}", self.skill_name()));
        command.arg("--target");
        command.arg(format!("custom:{}", self.target_root.display()));
        command.arg("--config");
        command.arg(&self.config_path);
        command.arg("--dry-run");
        command.output().expect("run dry-run install")
    }

    fn config_install_mode(&self) -> String {
        let config_text = fs::read_to_string(&self.config_path).expect("read config");
        let config_value: toml::Value = toml::from_str(&config_text).expect("valid toml");
        config_value
            .get("skills")
            .and_then(|value| value.as_array())
            .and_then(|skills| skills.first())
            .and_then(|skill| skill.get("install"))
            .and_then(|install| install.get("mode"))
            .and_then(|mode| mode.as_str())
            .expect("install mode")
            .to_string()
    }

    fn lock_install_mode(&self) -> String {
        let lock = read_lock_file(&lock_path_for_config(&self.config_path))
            .expect("read lock")
            .expect("lock should exist after install");
        lock.skills
            .first()
            .expect("lock skill entry")
            .install_mode
            .clone()
    }

    fn skill_name(&self) -> &str {
        self.source_dir
            .file_name()
            .and_then(|value| value.to_str())
            .expect("skill name")
    }

    fn target_path(&self) -> PathBuf {
        self.target_root.join(self.skill_name())
    }
}

fn assert_success(output: &Output) {
    assert_eq!(
        output.status.code(),
        Some(0),
        "command should succeed, stdout=`{}` stderr=`{}`",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[cfg(windows)]
fn parse_logged_path<'a>(log: &'a str, prefix: &str) -> Option<&'a str> {
    log.lines().find_map(|line| line.strip_prefix(prefix))
}

fn eden_command(home_dir: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_eden-skills"));
    command.env("HOME", home_dir);
    #[cfg(windows)]
    command.env("USERPROFILE", home_dir);
    command
}
