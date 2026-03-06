use std::fs;
use std::path::Path;
#[cfg(windows)]
use std::process::Command;
use std::sync::Arc;

use eden_skills_core::adapter::{
    create_adapter, parse_environment, AdapterEnvironment, DockerAdapter, LocalAdapter,
    TargetAdapter,
};
use eden_skills_core::config::InstallMode;
use tempfile::tempdir;
use tokio::task::JoinSet;

#[tokio::test]
async fn target_adapter_trait_object_can_be_spawned_via_joinset() {
    let adapter: Arc<dyn TargetAdapter> = Arc::new(LocalAdapter::new());
    let mut join_set = JoinSet::new();
    let cloned = Arc::clone(&adapter);
    join_set.spawn(async move {
        cloned.health_check().await?;
        Ok::<String, eden_skills_core::error::AdapterError>(cloned.adapter_type().to_string())
    });

    let output = join_set
        .join_next()
        .await
        .expect("task result")
        .expect("join")
        .expect("adapter result");
    assert_eq!(output, "local");
}

#[tokio::test]
async fn local_adapter_install_copy_and_path_exists_work() {
    let temp = tempdir().expect("tempdir");
    let source = temp.path().join("source");
    let target = temp.path().join("target");
    fs::create_dir_all(&source).expect("create source");
    fs::write(source.join("README.md"), "hello\n").expect("write source");

    let adapter = LocalAdapter::new();
    adapter
        .install(&source, &target, InstallMode::Copy)
        .await
        .expect("install copy");

    assert!(adapter.path_exists(&target).await.expect("target exists"));
    assert!(!adapter
        .path_exists(&temp.path().join("missing"))
        .await
        .expect("missing path result"));
    let copied = fs::read_to_string(target.join("README.md")).expect("read copied");
    assert_eq!(copied, "hello\n");
}

#[tokio::test]
async fn local_adapter_uninstall_removes_existing_target() {
    let temp = tempdir().expect("tempdir");
    let source = temp.path().join("source");
    let target = temp.path().join("target");
    fs::create_dir_all(&source).expect("create source");
    fs::write(source.join("README.md"), "hello\n").expect("write source");

    let adapter = LocalAdapter::new();
    adapter
        .install(&source, &target, InstallMode::Copy)
        .await
        .expect("install copy");
    assert!(target.exists(), "target should exist before uninstall");

    adapter.uninstall(&target).await.expect("uninstall");
    assert!(!target.exists(), "target should be removed by uninstall");
}

#[tokio::test]
async fn local_adapter_symlink_supports_directory_and_file_sources() {
    let temp = tempdir().expect("tempdir");
    let source_dir = temp.path().join("source-dir");
    let source_file = temp.path().join("source-file.txt");
    let target_dir = temp.path().join("target-dir-link");
    let target_file = temp.path().join("target-file-link.txt");

    fs::create_dir_all(&source_dir).expect("create source dir");
    fs::write(source_dir.join("nested.txt"), "nested\n").expect("write nested");
    fs::write(&source_file, "plain\n").expect("write source file");

    let adapter = LocalAdapter::new();
    adapter
        .install(&source_dir, &target_dir, InstallMode::Symlink)
        .await
        .expect("install dir symlink");
    adapter
        .install(&source_file, &target_file, InstallMode::Symlink)
        .await
        .expect("install file symlink");

    let dir_meta = fs::symlink_metadata(&target_dir).expect("dir symlink metadata");
    let file_meta = fs::symlink_metadata(&target_file).expect("file symlink metadata");
    assert!(dir_meta.file_type().is_symlink());
    assert!(file_meta.file_type().is_symlink());
}

#[tokio::test]
async fn local_adapter_exec_runs_command() {
    let adapter = LocalAdapter::new();
    let output = adapter.exec("echo local-adapter-ok").await.expect("exec");
    assert!(
        output.contains("local-adapter-ok"),
        "unexpected exec output: {output}"
    );
}

#[test]
fn parse_environment_is_deterministic() {
    let local_a = parse_environment("local").expect("local env");
    let local_b = parse_environment("local").expect("local env");
    assert_eq!(local_a, local_b);
    assert_eq!(local_a, AdapterEnvironment::Local);

    let docker_a = parse_environment("docker:test-container").expect("docker env");
    let docker_b = parse_environment("docker:test-container").expect("docker env");
    assert_eq!(docker_a, docker_b);
    assert_eq!(
        docker_a,
        AdapterEnvironment::Docker {
            container_name: "test-container".to_string()
        }
    );
}

#[test]
fn create_adapter_rejects_invalid_environment() {
    let err = match create_adapter("docker:") {
        Ok(_) => panic!("invalid docker env should fail"),
        Err(err) => err,
    };
    assert!(
        err.to_string().contains("environment"),
        "error should mention environment, got: {err}"
    );

    let err = match create_adapter("ssh:my-host") {
        Ok(_) => panic!("unknown env should fail"),
        Err(err) => err,
    };
    assert!(
        err.to_string().contains("environment"),
        "error should mention environment, got: {err}"
    );
}

#[test]
fn docker_adapter_reports_missing_binary_at_construction() {
    let missing = Path::new("/definitely/missing/docker-bin");
    let err = DockerAdapter::with_binary("test-container", missing)
        .expect_err("missing docker binary should fail");
    assert!(
        err.to_string().contains("Docker CLI not found"),
        "unexpected error: {err}"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn docker_adapter_uses_docker_cli_for_health_install_and_exec() {
    let temp = tempdir().expect("tempdir");
    let state_path = temp.path().join("docker-state.txt");
    let docker_bin = temp.path().join("docker");
    let script = format!(
        "#!/bin/sh\nset -eu\nstate=\"{}\"\ncmd=\"$1\"\nshift\nif [ \"$cmd\" = \"--version\" ]; then\n  echo \"Docker version 27.0.0\"\n  exit 0\nfi\nif [ \"$cmd\" = \"inspect\" ]; then\n  if [ \"$1\" = \"--format\" ] && [ \"$2\" = \"{{{{.State.Running}}}}\" ] && [ \"$3\" = \"test-container\" ]; then\n    echo \"true\"\n    exit 0\n  fi\n  if [ \"$1\" = \"--format\" ] && [ \"$2\" = \"{{{{json .Mounts}}}}\" ] && [ \"$3\" = \"test-container\" ]; then\n    echo \"[]\"\n    exit 0\n  fi\n  echo \"false\"\n  exit 0\nfi\nif [ \"$cmd\" = \"cp\" ]; then\n  src=\"$1\"\n  dst=\"$2\"\n  printf \"%s\\n\" \"$dst\" > \"$state\"\n  if [ -d \"${{src%/.}}\" ]; then\n    exit 0\n  fi\n  exit 0\nfi\nif [ \"$cmd\" = \"exec\" ]; then\n  container=\"$1\"\n  shift\n  if [ \"$container\" != \"test-container\" ]; then\n    echo \"container not found\" >&2\n    exit 1\n  fi\n  if [ \"$1\" = \"sh\" ] && [ \"$2\" = \"-c\" ]; then\n    case \"$3\" in\n      test\\ -e\\ *)\n        path=\"${{3#test -e }}\"\n        path=\"${{path#\\\"}}\"\n        path=\"${{path%\\\"}}\"\n        if [ -f \"$state\" ] && grep -q \":$path$\" \"$state\"; then\n          exit 0\n        fi\n        exit 1\n        ;;\n      *)\n        echo \"$3\"\n        exit 0\n        ;;\n    esac\n  fi\nfi\necho \"unsupported docker call\" >&2\nexit 1\n",
        state_path.display()
    );
    fs::write(&docker_bin, script).expect("write docker stub");
    let mut perms = fs::metadata(&docker_bin)
        .expect("docker metadata")
        .permissions();
    use std::os::unix::fs::PermissionsExt;
    perms.set_mode(0o755);
    fs::set_permissions(&docker_bin, perms).expect("set executable");

    let adapter = docker_adapter_with_retry("test-container", &docker_bin);
    adapter.health_check().await.expect("health check");

    let source = temp.path().join("source");
    let target = Path::new("/workspace/skills/demo");
    fs::create_dir_all(&source).expect("create source");
    fs::write(source.join("README.md"), "hello\n").expect("write source");

    adapter
        .install(&source, target, InstallMode::Symlink)
        .await
        .expect("install via docker cp");
    assert!(
        adapter.path_exists(target).await.expect("path exists"),
        "docker target should exist after install"
    );

    let output = adapter.exec("echo inside-container").await.expect("exec");
    assert_eq!(output.trim(), "echo inside-container");

    adapter.uninstall(target).await.expect("docker uninstall");
}

#[cfg(unix)]
#[tokio::test]
async fn docker_adapter_health_check_fails_when_container_not_running() {
    let temp = tempdir().expect("tempdir");
    let docker_bin = temp.path().join("docker");
    let script = r#"#!/bin/sh
set -eu
cmd="$1"
shift
if [ "$cmd" = "--version" ]; then
  echo "Docker version 27.0.0"
  exit 0
fi
if [ "$cmd" = "inspect" ]; then
  echo "false"
  exit 0
fi
echo "unsupported docker call" >&2
exit 1
"#;
    fs::write(&docker_bin, script).expect("write docker stub");
    let mut perms = fs::metadata(&docker_bin)
        .expect("docker metadata")
        .permissions();
    use std::os::unix::fs::PermissionsExt;
    perms.set_mode(0o755);
    fs::set_permissions(&docker_bin, perms).expect("set executable");

    let adapter = docker_adapter_with_retry("stopped-container", &docker_bin);
    let err = adapter
        .health_check()
        .await
        .expect_err("health check must fail");
    assert!(
        err.to_string().contains("docker start stopped-container"),
        "unexpected health check error: {err}"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn docker_adapter_install_permission_failure_includes_container_and_target_path() {
    let temp = tempdir().expect("tempdir");
    let docker_bin = temp.path().join("docker");
    let script = r#"#!/bin/sh
set -eu
cmd="$1"
shift
if [ "$cmd" = "--version" ]; then
  echo "Docker version 27.0.0"
  exit 0
fi
if [ "$cmd" = "inspect" ]; then
  if [ "$1" = "--format" ] && [ "$2" = "{{.State.Running}}" ]; then
    echo "true"
    exit 0
  fi
  if [ "$1" = "--format" ] && [ "$2" = "{{json .Mounts}}" ]; then
    echo "[]"
    exit 0
  fi
fi
if [ "$cmd" = "cp" ]; then
  echo "read-only file system" >&2
  exit 1
fi
echo "unsupported docker call" >&2
exit 1
"#;
    fs::write(&docker_bin, script).expect("write docker stub");
    let mut perms = fs::metadata(&docker_bin)
        .expect("docker metadata")
        .permissions();
    use std::os::unix::fs::PermissionsExt;
    perms.set_mode(0o755);
    fs::set_permissions(&docker_bin, perms).expect("set executable");

    let adapter = docker_adapter_with_retry("readonly-container", &docker_bin);
    let source = temp.path().join("source");
    fs::create_dir_all(&source).expect("create source");
    fs::write(source.join("README.md"), "hello\n").expect("write source");

    let target = Path::new("/workspace/skills/demo");
    let err = adapter
        .install(&source, target, InstallMode::Copy)
        .await
        .expect_err("install should fail on docker cp permission error");
    let message = err.to_string();
    assert!(
        message.contains("readonly-container"),
        "expected container name in error message, got: {message}"
    );
    assert!(
        message.contains("/workspace/skills/demo"),
        "expected target path in error message, got: {message}"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn docker_adapter_symlink_mode_emits_warning_and_falls_back_to_copy() {
    let temp = tempdir().expect("tempdir");
    let state_path = temp.path().join("docker-state.txt");
    let docker_bin = temp.path().join("docker");
    let script = format!(
        "#!/bin/sh\nset -eu\nstate=\"{}\"\ncmd=\"$1\"\nshift\nif [ \"$cmd\" = \"--version\" ]; then\n  echo \"Docker version 27.0.0\"\n  exit 0\nfi\nif [ \"$cmd\" = \"inspect\" ]; then\n  if [ \"$1\" = \"--format\" ] && [ \"$2\" = \"{{{{.State.Running}}}}\" ] && [ \"$3\" = \"test-container\" ]; then\n    echo \"true\"\n    exit 0\n  fi\n  if [ \"$1\" = \"--format\" ] && [ \"$2\" = \"{{{{json .Mounts}}}}\" ] && [ \"$3\" = \"test-container\" ]; then\n    echo \"[]\"\n    exit 0\n  fi\n  echo \"false\"\n  exit 0\nfi\nif [ \"$cmd\" = \"cp\" ]; then\n  src=\"$1\"\n  dst=\"$2\"\n  printf \"%s\\n\" \"$dst\" > \"$state\"\n  if [ -d \"${{src%/.}}\" ]; then\n    exit 0\n  fi\n  exit 0\nfi\nif [ \"$cmd\" = \"exec\" ]; then\n  container=\"$1\"\n  shift\n  if [ \"$container\" != \"test-container\" ]; then\n    echo \"container not found\" >&2\n    exit 1\n  fi\n  if [ \"$1\" = \"sh\" ] && [ \"$2\" = \"-c\" ]; then\n    case \"$3\" in\n      test\\ -e\\ *)\n        path=\"${{3#test -e }}\"\n        path=\"${{path#\\\"}}\"\n        path=\"${{path%\\\"}}\"\n        if [ -f \"$state\" ] && grep -q \":$path$\" \"$state\"; then\n          exit 0\n        fi\n        exit 1\n        ;;\n      *)\n        exit 0\n        ;;\n    esac\n  fi\nfi\necho \"unsupported docker call\" >&2\nexit 1\n",
        state_path.display()
    );
    fs::write(&docker_bin, script).expect("write docker stub");
    let mut perms = fs::metadata(&docker_bin)
        .expect("docker metadata")
        .permissions();
    use std::os::unix::fs::PermissionsExt;
    perms.set_mode(0o755);
    fs::set_permissions(&docker_bin, perms).expect("set executable");

    let adapter = docker_adapter_with_retry("test-container", &docker_bin);
    let source = temp.path().join("source");
    fs::create_dir_all(&source).expect("create source");
    fs::write(source.join("README.md"), "hello\n").expect("write source");
    let target = Path::new("/workspace/skills/demo");

    let (effective_mode, warning) =
        DockerAdapter::resolve_install_mode(InstallMode::Symlink, "test-container");
    assert_eq!(
        effective_mode,
        InstallMode::Copy,
        "docker symlink mode should fallback to copy"
    );
    assert!(
        warning
            .as_deref()
            .is_some_and(|value| value.contains("falling back to copy")),
        "expected symlink fallback warning payload, got: {warning:?}"
    );

    adapter
        .install(&source, target, InstallMode::Symlink)
        .await
        .expect("docker install should fallback to copy");
    assert!(
        adapter.path_exists(target).await.expect("path exists"),
        "docker target should exist after fallback copy"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn docker_adapter_bind_mount_for_path_resolves_matching_host_path() {
    let temp = tempdir().expect("tempdir");
    let host_mount = temp.path().join("host-claude-skills");
    fs::create_dir_all(&host_mount).expect("create host mount");
    let docker_bin = temp.path().join("docker");
    let script = format!(
        r#"#!/bin/sh
set -eu
cmd="$1"
shift
if [ "$cmd" = "--version" ]; then
  echo "Docker version 27.0.0"
  exit 0
fi
if [ "$cmd" = "inspect" ]; then
  if [ "$1" = "--format" ] && [ "$2" = "{{{{.State.Running}}}}" ]; then
    echo "true"
    exit 0
  fi
  if [ "$1" = "--format" ] && [ "$2" = "{{{{json .Mounts}}}}" ]; then
    printf '%s\n' '[{{"Type":"bind","Source":"{host_mount}","Destination":"/root/.claude/skills","RW":true}}]'
    exit 0
  fi
fi
echo "unsupported docker call: $cmd" >&2
exit 1
"#,
        host_mount = host_mount.display()
    );
    write_unix_executable(&docker_bin, &script);

    let adapter = docker_adapter_with_retry("test-container", &docker_bin);
    let host_path = adapter
        .bind_mount_for_path(Path::new("/root/.claude/skills/demo-skill"))
        .await
        .expect("bind mount lookup should succeed")
        .expect("bind mount should resolve to host path");
    assert_eq!(host_path, host_mount.join("demo-skill"));
}

#[cfg(unix)]
#[tokio::test]
async fn docker_adapter_install_uses_host_bind_mount_instead_of_docker_cp() {
    let temp = tempdir().expect("tempdir");
    let host_mount = temp.path().join("host-claude-skills");
    let cp_log = temp.path().join("docker-cp.log");
    let copied_targets = temp.path().join("copied-targets.log");
    fs::create_dir_all(&host_mount).expect("create host mount");

    let docker_bin = temp.path().join("docker");
    let script = format!(
        r#"#!/bin/sh
set -eu
cp_log="{cp_log}"
copied_targets="{copied_targets}"
cmd="$1"
shift
if [ "$cmd" = "--version" ]; then
  echo "Docker version 27.0.0"
  exit 0
fi
if [ "$cmd" = "inspect" ]; then
  if [ "$1" = "--format" ] && [ "$2" = "{{{{.State.Running}}}}" ]; then
    echo "true"
    exit 0
  fi
  if [ "$1" = "--format" ] && [ "$2" = "{{{{json .Mounts}}}}" ]; then
    printf '%s\n' '[{{"Type":"bind","Source":"{host_mount}","Destination":"/root/.claude/skills","RW":true}}]'
    exit 0
  fi
fi
if [ "$cmd" = "cp" ]; then
  printf '%s\n' "$2" >> "$cp_log"
  dst="$2"
  printf '%s\n' "${{dst#*:}}" >> "$copied_targets"
  exit 0
fi
if [ "$cmd" = "exec" ]; then
  container="$1"
  shift
  if [ "$container" != "test-container" ]; then
    echo "container not found" >&2
    exit 1
  fi
  if [ "$1" = "sh" ] && [ "$2" = "-c" ]; then
    case "$3" in
      test\ -e\ *)
        path="${{3#test -e }}"
        path="${{path#\"}}"
        path="${{path%\"}}"
        if [ -f "$copied_targets" ] && grep -Fxq "$path" "$copied_targets"; then
          exit 0
        fi
        exit 1
        ;;
      *)
        exit 0
        ;;
    esac
  fi
fi
echo "unsupported docker call: $cmd" >&2
exit 1
"#,
        cp_log = cp_log.display(),
        copied_targets = copied_targets.display(),
        host_mount = host_mount.display()
    );
    write_unix_executable(&docker_bin, &script);

    let source = temp.path().join("source");
    fs::create_dir_all(&source).expect("create source");
    fs::write(source.join("README.md"), "hello\n").expect("write source");

    let adapter = docker_adapter_with_retry("test-container", &docker_bin);
    adapter
        .install(
            &source,
            Path::new("/root/.claude/skills/demo-skill"),
            InstallMode::Symlink,
        )
        .await
        .expect("install should succeed");

    let host_target = host_mount.join("demo-skill");
    let metadata = fs::symlink_metadata(&host_target).expect("host target metadata");
    assert!(
        metadata.file_type().is_symlink(),
        "bind-mounted install should create host-side symlink"
    );
    assert_eq!(
        fs::read_link(&host_target).expect("read host symlink"),
        source,
        "host-side symlink should point at source path"
    );
    let cp_calls = fs::read_to_string(&cp_log).unwrap_or_default();
    assert!(
        cp_calls.trim().is_empty(),
        "bind-mounted install must not fall back to docker cp, log={cp_calls:?}"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn docker_adapter_install_checks_mounts_before_falling_back_to_docker_cp() {
    let temp = tempdir().expect("tempdir");
    let mount_inspect_log = temp.path().join("mount-inspect.log");
    let cp_log = temp.path().join("docker-cp.log");
    let copied_targets = temp.path().join("copied-targets.log");
    let docker_bin = temp.path().join("docker");
    let script = format!(
        r#"#!/bin/sh
set -eu
mount_inspect_log="{mount_inspect_log}"
cp_log="{cp_log}"
copied_targets="{copied_targets}"
cmd="$1"
shift
if [ "$cmd" = "--version" ]; then
  echo "Docker version 27.0.0"
  exit 0
fi
if [ "$cmd" = "inspect" ]; then
  if [ "$1" = "--format" ] && [ "$2" = "{{{{.State.Running}}}}" ]; then
    echo "true"
    exit 0
  fi
  if [ "$1" = "--format" ] && [ "$2" = "{{{{json .Mounts}}}}" ]; then
    printf 'inspected\n' >> "$mount_inspect_log"
    printf '%s\n' '[]'
    exit 0
  fi
fi
if [ "$cmd" = "cp" ]; then
  printf '%s\n' "$2" >> "$cp_log"
  dst="$2"
  printf '%s\n' "${{dst#*:}}" >> "$copied_targets"
  exit 0
fi
if [ "$cmd" = "exec" ]; then
  container="$1"
  shift
  if [ "$container" != "test-container" ]; then
    echo "container not found" >&2
    exit 1
  fi
  if [ "$1" = "sh" ] && [ "$2" = "-c" ]; then
    case "$3" in
      test\ -e\ *)
        path="${{3#test -e }}"
        path="${{path#\"}}"
        path="${{path%\"}}"
        if [ -f "$copied_targets" ] && grep -Fxq "$path" "$copied_targets"; then
          exit 0
        fi
        exit 1
        ;;
      *)
        exit 0
        ;;
    esac
  fi
fi
echo "unsupported docker call: $cmd" >&2
exit 1
"#,
        mount_inspect_log = mount_inspect_log.display(),
        cp_log = cp_log.display(),
        copied_targets = copied_targets.display()
    );
    write_unix_executable(&docker_bin, &script);

    let source = temp.path().join("source");
    fs::create_dir_all(&source).expect("create source");
    fs::write(source.join("README.md"), "hello\n").expect("write source");

    let adapter = docker_adapter_with_retry("test-container", &docker_bin);
    adapter
        .install(
            &source,
            Path::new("/root/.claude/skills/demo-skill"),
            InstallMode::Copy,
        )
        .await
        .expect("install should fall back to docker cp");

    assert!(
        mount_inspect_log.exists(),
        "install should inspect mounts before falling back to docker cp"
    );
    let cp_calls = fs::read_to_string(&cp_log).expect("read docker cp log");
    assert!(
        cp_calls.contains("test-container:/root/.claude/skills/demo-skill"),
        "expected docker cp fallback log entry, got: {cp_calls:?}"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn docker_adapter_uninstall_removes_host_bind_mount_without_docker_exec_rm() {
    let temp = tempdir().expect("tempdir");
    let host_mount = temp.path().join("host-claude-skills");
    let exec_log = temp.path().join("docker-exec.log");
    let docker_bin = temp.path().join("docker");
    fs::create_dir_all(host_mount.join("demo-skill")).expect("create bind-mounted target");
    fs::write(host_mount.join("demo-skill/README.md"), "hello\n").expect("write bind target");

    let script = format!(
        r#"#!/bin/sh
set -eu
exec_log="{exec_log}"
cmd="$1"
shift
if [ "$cmd" = "--version" ]; then
  echo "Docker version 27.0.0"
  exit 0
fi
if [ "$cmd" = "inspect" ]; then
  if [ "$1" = "--format" ] && [ "$2" = "{{{{.State.Running}}}}" ]; then
    echo "true"
    exit 0
  fi
  if [ "$1" = "--format" ] && [ "$2" = "{{{{json .Mounts}}}}" ]; then
    printf '%s\n' '[{{"Type":"bind","Source":"{host_mount}","Destination":"/root/.claude/skills","RW":true}}]'
    exit 0
  fi
fi
if [ "$cmd" = "exec" ]; then
  printf '%s\n' "$*" >> "$exec_log"
  exit 0
fi
echo "unsupported docker call: $cmd" >&2
exit 1
"#,
        exec_log = exec_log.display(),
        host_mount = host_mount.display()
    );
    write_unix_executable(&docker_bin, &script);

    let adapter = docker_adapter_with_retry("test-container", &docker_bin);
    adapter
        .uninstall(Path::new("/root/.claude/skills/demo-skill"))
        .await
        .expect("uninstall should succeed");

    assert!(
        !host_mount.join("demo-skill").exists(),
        "bind-mounted uninstall should remove host-side target"
    );
    let exec_calls = fs::read_to_string(&exec_log).unwrap_or_default();
    assert!(
        exec_calls.trim().is_empty(),
        "bind-mounted uninstall must not shell into docker for rm, log={exec_calls:?}"
    );
}

#[cfg(windows)]
#[tokio::test]
async fn local_adapter_windows_symlink_and_junction_permission_denied_reports_guidance() {
    let temp = tempdir().expect("tempdir");
    let source = temp.path().join("source");
    fs::create_dir_all(&source).expect("create source");
    fs::write(source.join("README.md"), "hello\n").expect("write source");

    let restricted = temp.path().join("restricted-target-root");
    fs::create_dir_all(&restricted).expect("create restricted root");
    let original_permissions = make_read_only_dir_windows(&restricted);

    let target = restricted.join("demo-skill-link");
    let adapter = LocalAdapter::new();
    let install_result = adapter
        .install(&source, &target, InstallMode::Symlink)
        .await;

    restore_permissions_windows(&restricted, original_permissions);

    let err = install_result.expect_err("symlink install should fail with permission denied");
    let message = err.to_string();
    assert!(
        message.contains("Enable Developer Mode or run as Administrator"),
        "expected Windows remediation hint, got: {message}"
    );
    assert!(
        message.contains("junction fallback failed"),
        "expected compound fallback context in error message, got: {message}"
    );
}

#[cfg(unix)]
fn docker_adapter_with_retry(container_name: &str, docker_bin: &Path) -> DockerAdapter {
    let mut last_err = None;
    for _ in 0..20 {
        match DockerAdapter::with_binary(container_name, docker_bin) {
            Ok(adapter) => return adapter,
            Err(err) => {
                let message = err.to_string();
                let is_text_file_busy =
                    message.contains("Text file busy") || message.contains("(os error 26)");
                if !is_text_file_busy {
                    panic!("docker adapter: {err}");
                }
                last_err = Some(err);
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }
    }

    panic!(
        "docker adapter: {}",
        last_err.expect("expected text-file-busy error")
    );
}

#[cfg(unix)]
fn write_unix_executable(path: &Path, contents: &str) {
    fs::write(path, contents).expect("write unix executable");
    let mut perms = fs::metadata(path)
        .expect("unix executable metadata")
        .permissions();
    use std::os::unix::fs::PermissionsExt;
    perms.set_mode(0o755);
    fs::set_permissions(path, perms).expect("set unix executable permissions");
}

#[cfg(windows)]
fn make_read_only_dir_windows(path: &Path) -> fs::Permissions {
    let original = fs::metadata(path)
        .expect("restricted metadata")
        .permissions();
    let principal = current_windows_principal();
    let grant_rule_self = format!("{principal}:RX");
    let grant_rule_children = format!("{principal}:(OI)(CI)RX");
    let deny_rule_self = format!("{principal}:W");
    let deny_rule_children = format!("{principal}:(OI)(CI)W");
    run_icacls(path, &["/inheritance:r"]);
    run_icacls(path, &["/grant:r", &grant_rule_self]);
    run_icacls(path, &["/grant", &grant_rule_children]);
    run_icacls(path, &["/deny", &deny_rule_self]);
    run_icacls(path, &["/deny", &deny_rule_children]);
    original
}

#[cfg(windows)]
fn restore_permissions_windows(path: &Path, _permissions: fs::Permissions) {
    run_icacls(path, &["/reset", "/T", "/C"]);
}

#[cfg(windows)]
fn run_icacls(path: &Path, args: &[&str]) {
    let output = Command::new("icacls")
        .arg(path)
        .args(args)
        .output()
        .expect("spawn icacls");
    if output.status.success() {
        return;
    }
    panic!(
        "icacls {:?} failed for {}: status={} stderr=`{}` stdout=`{}`",
        args,
        path.display(),
        output.status,
        String::from_utf8_lossy(&output.stderr).trim(),
        String::from_utf8_lossy(&output.stdout).trim()
    );
}

#[cfg(windows)]
fn current_windows_principal() -> String {
    let output = Command::new("whoami").output().expect("spawn whoami");
    if !output.status.success() {
        panic!(
            "whoami failed: status={} stderr=`{}` stdout=`{}`",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim(),
            String::from_utf8_lossy(&output.stdout).trim()
        );
    }
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}
