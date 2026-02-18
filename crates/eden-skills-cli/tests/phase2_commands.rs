use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::tempdir;

#[test]
fn update_clones_configured_registries() {
    let temp = tempdir().expect("tempdir");
    let skill_repo = init_git_repo(
        temp.path(),
        "skill-origin",
        &[("packages/browser/README.md", "seed")],
    );
    let official_registry = init_git_repo(
        temp.path(),
        "registry-official",
        &[("manifest.toml", "format_version = 1\nname = \"official\"\n")],
    );
    let forge_registry = init_git_repo(
        temp.path(),
        "registry-forge",
        &[("manifest.toml", "format_version = 1\nname = \"forge\"\n")],
    );

    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("agent-target");
    let config_path = temp.path().join("skills.toml");
    fs::write(
        &config_path,
        format!(
            r#"
version = 1

[storage]
root = "{storage_root}"

[registries]
official = {{ url = "{official_url}", priority = 100 }}
forge = {{ url = "{forge_url}", priority = 10 }}

[[skills]]
id = "phase1-skill"

[skills.source]
repo = "{skill_url}"
subpath = "packages/browser"
ref = "main"

[[skills.targets]]
agent = "custom"
path = "{target_root}"
"#,
            storage_root = toml_escape_path(&storage_root),
            official_url = as_file_url(&official_registry),
            forge_url = as_file_url(&forge_registry),
            skill_url = as_file_url(&skill_repo),
            target_root = toml_escape_path(&target_root),
        ),
    )
    .expect("write config");

    let output = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args(["update", "--config"])
        .arg(&config_path)
        .output()
        .expect("run update");

    assert_eq!(
        output.status.code(),
        Some(0),
        "update should succeed, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("registry sync:"),
        "expected registry sync summary, got: {stdout}"
    );
    assert!(storage_root.join("registries/official/.git").exists());
    assert!(storage_root.join("registries/forge/.git").exists());
}

#[test]
fn install_resolves_skill_from_registry_and_persists_mode_b_entry() {
    let temp = tempdir().expect("tempdir");
    let skill_repo = init_git_repo(
        temp.path(),
        "google-search-origin",
        &[("skill/README.md", "google-search")],
    );
    let head = git_head(&skill_repo);

    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("agent-target");
    let registry_cache = storage_root.join("registries").join("official");
    write_registry_index_entry(
        &registry_cache,
        "google-search",
        &as_file_url(&skill_repo),
        "1.2.0",
        "main",
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
repo = "{skill_url}"
subpath = "skill"
ref = "main"

[[skills.targets]]
agent = "custom"
path = "{target_root}"
"#,
            storage_root = toml_escape_path(&storage_root),
            skill_url = as_file_url(&skill_repo),
            target_root = toml_escape_path(&target_root),
        ),
    )
    .expect("write config");

    let output = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args(["install", "google-search", "--config"])
        .arg(&config_path)
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
        written.contains("name = \"google-search\""),
        "expected mode b name entry in config, got:\n{written}"
    );
    assert!(
        written.contains("version = \"*\""),
        "expected default version constraint in config, got:\n{written}"
    );
}

#[test]
fn apply_and_repair_resolve_mode_b_skills_before_source_sync() {
    let temp = tempdir().expect("tempdir");
    let skill_repo = init_git_repo(
        temp.path(),
        "google-search-origin",
        &[("skill/README.md", "google-search")],
    );
    let head = git_head(&skill_repo);

    let storage_root = temp.path().join("storage");
    let target_root = temp.path().join("agent-target");
    let registry_cache = storage_root.join("registries").join("official");
    write_registry_index_entry(
        &registry_cache,
        "google-search",
        &as_file_url(&skill_repo),
        "1.2.0",
        "main",
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
name = "google-search"
version = "^1.0"
registry = "official"

[skills.install]
mode = "symlink"

[[skills.targets]]
agent = "custom"
path = "{target_root}"
"#,
            storage_root = toml_escape_path(&storage_root),
            target_root = toml_escape_path(&target_root),
        ),
    )
    .expect("write config");

    let apply_output = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args(["apply", "--config"])
        .arg(&config_path)
        .output()
        .expect("run apply");
    assert_eq!(
        apply_output.status.code(),
        Some(0),
        "apply should succeed, stderr={}",
        String::from_utf8_lossy(&apply_output.stderr)
    );

    let target_skill = target_root.join("google-search");
    assert!(
        fs::symlink_metadata(&target_skill)
            .expect("target metadata")
            .file_type()
            .is_symlink(),
        "apply should create symlink target for mode b skill"
    );

    remove_symlink(&target_skill).expect("remove target symlink");
    let broken = temp.path().join("broken-link-target");
    create_symlink(&broken, &target_skill).expect("create broken symlink");

    let repair_output = Command::new(env!("CARGO_BIN_EXE_eden-skills"))
        .args(["repair", "--config"])
        .arg(&config_path)
        .output()
        .expect("run repair");
    assert_eq!(
        repair_output.status.code(),
        Some(0),
        "repair should succeed, stderr={}",
        String::from_utf8_lossy(&repair_output.stderr)
    );
}

fn init_git_repo(base: &Path, name: &str, files: &[(&str, &str)]) -> PathBuf {
    let repo = base.join(name);
    for (rel, content) in files {
        let path = repo.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent");
        }
        fs::write(path, content).expect("write file");
    }
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
    version: &str,
    git_ref: &str,
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

    let body = format!(
        r#"
[skill]
name = "{skill_name}"
repo = "{repo_url}"
subpath = "skill"

[[versions]]
version = "{version}"
ref = "{git_ref}"
commit = "{commit}"
yanked = false
"#
    );
    fs::write(index_dir.join(format!("{skill_name}.toml")), body).expect("write index entry");
}

fn git_head(repo: &Path) -> String {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo)
        .output()
        .expect("read head");
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
    if output.status.success() {
        return;
    }

    panic!(
        "git {:?} failed in {}: status={} stderr=`{}` stdout=`{}`",
        args,
        cwd.display(),
        output.status,
        String::from_utf8_lossy(&output.stderr).trim(),
        String::from_utf8_lossy(&output.stdout).trim()
    );
}

fn as_file_url(path: &Path) -> String {
    format!("file://{}", path.display())
}

fn toml_escape_path(path: &Path) -> String {
    path.display().to_string().replace('\\', "\\\\")
}

#[cfg(unix)]
fn create_symlink(source: &Path, target: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(source, target)
}

#[cfg(windows)]
fn create_symlink(source: &Path, target: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_dir(source, target)
}

#[cfg(unix)]
fn remove_symlink(path: &Path) -> std::io::Result<()> {
    fs::remove_file(path)
}

#[cfg(windows)]
fn remove_symlink(path: &Path) -> std::io::Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => fs::remove_dir(path),
        Err(err) => Err(err),
    }
}
