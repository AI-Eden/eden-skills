mod common;

use std::fs;
use std::path::Path;
use std::process::Command;

use tempfile::tempdir;

#[test]
fn local_path_install_persists_absolute_repo_and_stages_source_into_storage_root() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let source_dir = temp.path().join("test-skills");
    fs::create_dir_all(&source_dir).expect("create source dir");
    fs::write(source_dir.join("README.md"), "demo skill").expect("write source file");

    let config_path = temp.path().join("skills.toml");
    let output = eden_command(&home_dir)
        .current_dir(temp.path())
        .args(["install", "./test-skills", "--config"])
        .arg(&config_path)
        .output()
        .expect("run install");

    assert_eq!(
        output.status.code(),
        Some(0),
        "install should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Created config at"),
        "install should auto-create config when missing, stdout={stdout}"
    );

    let config_text = fs::read_to_string(&config_path).expect("read config");
    let config_value: toml::Value = toml::from_str(&config_text).expect("valid toml");
    let skills = config_value
        .get("skills")
        .and_then(|value| value.as_array())
        .expect("skills array");
    assert_eq!(skills.len(), 1);
    let actual_repo = skills[0]["source"]["repo"]
        .as_str()
        .expect("source repo should be string");
    common::assert_paths_resolve_to_same_location(&source_dir, Path::new(actual_repo));

    let storage_root = home_dir.join(".eden-skills").join("skills");
    let staged_skill_root = storage_root.join("test-skills");
    assert!(
        staged_skill_root.exists(),
        "local path install should stage source into storage root"
    );
    assert!(
        staged_skill_root.join("README.md").exists(),
        "staged skill root should contain copied source contents"
    );
}

#[test]
fn install_fails_when_config_parent_directory_is_missing() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let source_dir = temp.path().join("test-skills");
    fs::create_dir_all(&source_dir).expect("create source dir");
    fs::write(source_dir.join("README.md"), "demo skill").expect("write source file");

    let missing_parent_config = temp.path().join("missing").join("skills.toml");
    let output = eden_command(&home_dir)
        .current_dir(temp.path())
        .args(["install", "./test-skills", "--config"])
        .arg(&missing_parent_config)
        .output()
        .expect("run install");

    assert_eq!(
        output.status.code(),
        Some(1),
        "install should fail when config parent is missing, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn install_default_config_path_auto_creates_missing_parent_directory() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let source_dir = temp.path().join("test-skills");
    fs::create_dir_all(&source_dir).expect("create source dir");
    fs::write(source_dir.join("README.md"), "demo skill").expect("write source file");

    let default_config_parent = home_dir.join(".eden-skills");
    let default_config_path = default_config_parent.join("skills.toml");
    assert!(
        !default_config_parent.exists(),
        "test precondition: default config parent should be missing"
    );

    let output = eden_command(&home_dir)
        .current_dir(temp.path())
        .args(["install", "./test-skills"])
        .output()
        .expect("run install");

    assert_eq!(
        output.status.code(),
        Some(0),
        "install should succeed for default config path, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        default_config_parent.exists(),
        "default config parent should be auto-created"
    );
    assert!(
        default_config_path.exists(),
        "default config file should be created"
    );
}

#[test]
fn local_path_precedence_wins_over_shorthand_shape() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let source_dir = temp.path().join("owner").join("repo");
    fs::create_dir_all(&source_dir).expect("create source dir");
    fs::write(source_dir.join("README.md"), "demo skill").expect("write source file");

    let config_path = temp.path().join("skills.toml");
    let output = eden_command(&home_dir)
        .current_dir(temp.path())
        .args(["install", "./owner/repo", "--config"])
        .arg(&config_path)
        .output()
        .expect("run install");

    assert_eq!(
        output.status.code(),
        Some(0),
        "install should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let config_text = fs::read_to_string(&config_path).expect("read config");
    let config_value: toml::Value = toml::from_str(&config_text).expect("valid toml");
    let skills = config_value
        .get("skills")
        .and_then(|value| value.as_array())
        .expect("skills array");
    assert_eq!(skills.len(), 1);
    let actual_repo = skills[0]["source"]["repo"]
        .as_str()
        .expect("source repo should be string");
    common::assert_paths_resolve_to_same_location(&source_dir, Path::new(actual_repo));
}

#[test]
fn registry_name_input_still_uses_registry_mode() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("agent-target");
    let skill_repo = init_git_repo(temp.path(), "browser-tool-origin");
    let head = git_head(&skill_repo);

    write_registry_index_entry(
        &storage_root.join("registries").join("official"),
        "browser-tool",
        &as_file_url(&skill_repo),
        &head,
    );

    let config_path = temp.path().join("skills.toml");
    fs::write(
        &config_path,
        format!(
            r#"
version = 1

[storage]
root = "{storage_root}"

[registries]
official = {{ url = "https://example.com/official.git", priority = 100 }}

[[skills]]
id = "phase1-skill"

[skills.source]
repo = "{skill_repo_url}"
subpath = "."
ref = "main"

[[skills.targets]]
agent = "custom"
path = "{target_root}"
"#,
            storage_root = toml_escape_path(&storage_root),
            skill_repo_url = as_file_url(&skill_repo),
            target_root = toml_escape_path(&target_root),
        ),
    )
    .expect("write config");

    let output = eden_command(&home_dir)
        .args(["install", "browser-tool", "--config"])
        .arg(&config_path)
        .output()
        .expect("run install");

    assert_eq!(
        output.status.code(),
        Some(0),
        "registry fallback install should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let written = fs::read_to_string(&config_path).expect("read config");
    assert!(
        written.contains("name = \"browser-tool\""),
        "registry mode should persist name/version entry, config=\n{written}"
    );
}

#[test]
fn install_url_mode_respects_id_override() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let source_dir = temp.path().join("repo-name");
    fs::create_dir_all(&source_dir).expect("create source dir");
    fs::write(source_dir.join("README.md"), "demo skill").expect("write source file");

    let config_path = temp.path().join("skills.toml");
    let output = eden_command(&home_dir)
        .args(["install", "./repo-name", "--id", "custom-name", "--config"])
        .arg(&config_path)
        .current_dir(temp.path())
        .output()
        .expect("run install");

    assert_eq!(
        output.status.code(),
        Some(0),
        "install should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let written = fs::read_to_string(&config_path).expect("read config");
    assert!(
        written.contains("id = \"custom-name\""),
        "expected id override to be persisted, config=\n{written}"
    );
}

#[test]
fn install_url_mode_upserts_existing_id_instead_of_duplicating() {
    let temp = tempdir().expect("tempdir");
    let home_dir = temp.path().join("home");
    let source_dir = temp.path().join("my-skill");
    fs::create_dir_all(&source_dir).expect("create source dir");
    fs::write(source_dir.join("README.md"), "demo skill").expect("write source file");

    let config_path = temp.path().join("skills.toml");
    fs::write(
        &config_path,
        r#"
version = 1

[storage]
root = "~/.eden-skills/skills"

[[skills]]
id = "my-skill"

[skills.source]
repo = "https://example.com/old.git"
subpath = "."
ref = "main"

[[skills.targets]]
agent = "claude-code"
"#,
    )
    .expect("write initial config");

    let output = eden_command(&home_dir)
        .args(["install", "./my-skill", "--config"])
        .arg(&config_path)
        .current_dir(temp.path())
        .output()
        .expect("run install");

    assert_eq!(
        output.status.code(),
        Some(0),
        "install should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let written = fs::read_to_string(&config_path).expect("read config");
    let value: toml::Value = toml::from_str(&written).expect("valid toml");
    let skills = value
        .get("skills")
        .and_then(|v| v.as_array())
        .expect("skills array");
    assert_eq!(
        skills.len(),
        1,
        "expected upsert semantics, config=\n{written}"
    );
    assert_eq!(skills[0]["id"].as_str(), Some("my-skill"));
    let actual_repo = skills[0]["source"]["repo"]
        .as_str()
        .expect("source repo should be string");
    common::assert_paths_resolve_to_same_location(&source_dir, Path::new(actual_repo));
}

fn eden_command(home_dir: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_eden-skills"));
    command.env("HOME", home_dir);
    #[cfg(windows)]
    command.env("USERPROFILE", home_dir);
    command
}

fn init_git_repo(base: &Path, name: &str) -> std::path::PathBuf {
    let repo = base.join(name);
    fs::create_dir_all(repo.join("skill")).expect("create skill dir");
    fs::write(repo.join("skill/README.md"), "seed").expect("write file");
    run_git(&repo, &["init"]);
    run_git(&repo, &["config", "user.email", "test@example.com"]);
    run_git(&repo, &["config", "user.name", "eden-skills-test"]);
    run_git(&repo, &["add", "."]);
    run_git(&repo, &["commit", "-m", "init"]);
    run_git(&repo, &["branch", "-M", "main"]);
    repo
}

fn write_registry_index_entry(
    registry_root: &Path,
    skill_name: &str,
    repo_url: &str,
    commit: &str,
) {
    let first = skill_name
        .chars()
        .next()
        .expect("skill name")
        .to_ascii_lowercase();
    let index_dir = registry_root.join("index").join(first.to_string());
    fs::create_dir_all(&index_dir).expect("create index dir");
    fs::write(
        registry_root.join("manifest.toml"),
        "format_version = 1\nname = \"official\"\n",
    )
    .expect("write manifest");
    fs::write(
        index_dir.join(format!("{skill_name}.toml")),
        format!(
            r#"
[skill]
name = "{skill_name}"
repo = "{repo_url}"
subpath = "skill"

[[versions]]
version = "1.0.0"
ref = "main"
commit = "{commit}"
yanked = false
"#
        ),
    )
    .expect("write registry entry");
}

fn git_head(repo: &Path) -> String {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo)
        .output()
        .expect("run git rev-parse");
    assert!(
        output.status.success(),
        "git rev-parse failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn run_git(cwd: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("spawn git");
    assert!(
        output.status.success(),
        "git {:?} failed in {}: status={} stderr=`{}` stdout=`{}`",
        args,
        cwd.display(),
        output.status,
        String::from_utf8_lossy(&output.stderr).trim(),
        String::from_utf8_lossy(&output.stdout).trim()
    );
}

fn as_file_url(path: &Path) -> String {
    let mut normalized = path.display().to_string().replace('\\', "/");
    if normalized
        .as_bytes()
        .get(1)
        .is_some_and(|candidate| *candidate == b':')
    {
        normalized.insert(0, '/');
    }
    format!("file://{normalized}")
}

fn toml_escape_path(path: &Path) -> String {
    path.display().to_string().replace('\\', "\\\\")
}
