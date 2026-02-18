use std::fs;
use std::path::Path;
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
        "#!/bin/sh\nset -eu\nstate=\"{}\"\ncmd=\"$1\"\nshift\nif [ \"$cmd\" = \"--version\" ]; then\n  echo \"Docker version 27.0.0\"\n  exit 0\nfi\nif [ \"$cmd\" = \"inspect\" ]; then\n  if [ \"$1\" = \"--format\" ] && [ \"$2\" = \"{{{{.State.Running}}}}\" ] && [ \"$3\" = \"test-container\" ]; then\n    echo \"true\"\n    exit 0\n  fi\n  echo \"false\"\n  exit 0\nfi\nif [ \"$cmd\" = \"cp\" ]; then\n  src=\"$1\"\n  dst=\"$2\"\n  printf \"%s\\n\" \"$dst\" > \"$state\"\n  if [ -d \"${{src%/.}}\" ]; then\n    exit 0\n  fi\n  exit 0\nfi\nif [ \"$cmd\" = \"exec\" ]; then\n  container=\"$1\"\n  shift\n  if [ \"$container\" != \"test-container\" ]; then\n    echo \"container not found\" >&2\n    exit 1\n  fi\n  if [ \"$1\" = \"sh\" ] && [ \"$2\" = \"-c\" ]; then\n    case \"$3\" in\n      test\\ -e\\ *)\n        path=\"${{3#test -e }}\"\n        path=\"${{path#\\\"}}\"\n        path=\"${{path%\\\"}}\"\n        if [ -f \"$state\" ] && grep -q \":$path$\" \"$state\"; then\n          exit 0\n        fi\n        exit 1\n        ;;\n      *)\n        echo \"$3\"\n        exit 0\n        ;;\n    esac\n  fi\nfi\necho \"unsupported docker call\" >&2\nexit 1\n",
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
