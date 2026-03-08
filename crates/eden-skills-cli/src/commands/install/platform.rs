//! Platform-specific install mode detection (symlink, junction, copy).

use eden_skills_core::config::InstallMode;

use crate::ui::UiContext;

use crate::commands::common::print_warning;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct DefaultInstallModeDecision {
    pub(super) mode: InstallMode,
    pub(super) warn_windows_junction_fallback: bool,
    pub(super) warn_windows_hardcopy_fallback: bool,
}

pub(super) fn default_install_mode() -> InstallMode {
    resolve_default_install_mode_decision().mode
}

pub(super) fn requested_install_mode(copy: bool) -> Option<InstallMode> {
    if copy {
        Some(InstallMode::Copy)
    } else {
        None
    }
}

pub(super) fn resolve_default_install_mode_decision() -> DefaultInstallModeDecision {
    if let Some(forced_symlink_supported) = forced_windows_symlink_support_for_tests() {
        #[cfg(windows)]
        let junction_supported = if forced_symlink_supported {
            false
        } else {
            forced_windows_junction_support_for_tests()
                .unwrap_or_else(windows_supports_junction_creation)
        };

        #[cfg(not(windows))]
        let junction_supported = if forced_symlink_supported {
            false
        } else {
            forced_windows_junction_support_for_tests().unwrap_or(false)
        };

        return decide_default_install_mode(true, forced_symlink_supported, junction_supported);
    }

    #[cfg(windows)]
    {
        let symlink_supported = windows_supports_symlink_creation();
        let junction_supported = if symlink_supported {
            false
        } else {
            windows_supports_junction_creation()
        };
        decide_default_install_mode(true, symlink_supported, junction_supported)
    }
    #[cfg(not(windows))]
    {
        decide_default_install_mode(false, true, false)
    }
}

fn decide_default_install_mode(
    is_windows: bool,
    symlink_supported: bool,
    junction_supported: bool,
) -> DefaultInstallModeDecision {
    if is_windows {
        if symlink_supported {
            return DefaultInstallModeDecision {
                mode: InstallMode::Symlink,
                warn_windows_junction_fallback: false,
                warn_windows_hardcopy_fallback: false,
            };
        }

        if junction_supported {
            return DefaultInstallModeDecision {
                mode: InstallMode::Symlink,
                warn_windows_junction_fallback: true,
                warn_windows_hardcopy_fallback: false,
            };
        }

        return DefaultInstallModeDecision {
            mode: InstallMode::Copy,
            warn_windows_junction_fallback: false,
            warn_windows_hardcopy_fallback: true,
        };
    }

    DefaultInstallModeDecision {
        mode: InstallMode::Symlink,
        warn_windows_junction_fallback: false,
        warn_windows_hardcopy_fallback: false,
    }
}

pub(super) fn warn_windows_hardcopy_fallback_if_needed(
    ui: &UiContext,
    decision: DefaultInstallModeDecision,
) {
    if ui.json_mode() {
        return;
    }

    if decision.warn_windows_junction_fallback {
        print_warning(
            ui,
            "Windows symlink permission unavailable; using NTFS junction (functionally equivalent, no admin required).",
        );
    } else if decision.warn_windows_hardcopy_fallback {
        print_warning(
            ui,
            "Windows symlink and junction unavailable; falling back to hardcopy mode (this may slow down installs).",
        );
    }
}

fn forced_windows_symlink_support_for_tests() -> Option<bool> {
    match std::env::var("EDEN_SKILLS_TEST_WINDOWS_SYMLINK_SUPPORTED")
        .ok()
        .as_deref()
    {
        Some("1") => Some(true),
        Some("0") => Some(false),
        _ => None,
    }
}

fn forced_windows_junction_support_for_tests() -> Option<bool> {
    match std::env::var("EDEN_SKILLS_TEST_WINDOWS_JUNCTION_SUPPORTED")
        .ok()
        .as_deref()
    {
        Some("1") => Some(true),
        Some("0") => Some(false),
        _ => None,
    }
}

#[cfg(windows)]
fn windows_supports_symlink_creation() -> bool {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let probe_root = std::env::temp_dir().join(format!("eden-skills-symlink-probe-{nonce}"));
    let source_dir = probe_root.join("source");
    let link_dir = probe_root.join("link");

    let created = fs::create_dir_all(&source_dir)
        .and_then(|_| std::os::windows::fs::symlink_dir(&source_dir, &link_dir))
        .is_ok();

    let _ = fs::remove_dir_all(&probe_root);
    created
}

#[cfg(windows)]
fn windows_supports_junction_creation() -> bool {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let probe_root = std::env::temp_dir().join(format!("eden-skills-junction-probe-{nonce}"));
    let source_dir = probe_root.join("source");
    let junction_path = probe_root.join("link");

    let mut created = false;
    let supported = match fs::create_dir_all(&source_dir) {
        Ok(()) => match junction::create(&source_dir, &junction_path) {
            Ok(()) => {
                created = true;
                junction::exists(&junction_path).unwrap_or(false)
            }
            Err(_) => false,
        },
        Err(_) => false,
    };

    let cleaned = cleanup_windows_junction_probe(&probe_root, &junction_path);

    if let Ok(log_path) = std::env::var("EDEN_SKILLS_TEST_WINDOWS_JUNCTION_PROBE_LOG") {
        let _ = fs::write(
            log_path,
            format!(
                "probe_root={}\njunction_path={}\ncreated={created}\ncleaned={cleaned}\n",
                probe_root.display(),
                junction_path.display()
            ),
        );
    }

    supported && cleaned
}

#[cfg(windows)]
fn cleanup_windows_junction_probe(
    probe_root: &std::path::Path,
    junction_path: &std::path::Path,
) -> bool {
    use std::fs;

    let junction_deleted = if junction::exists(junction_path).unwrap_or(false) {
        junction::delete(junction_path).is_ok()
    } else {
        true
    };
    let root_deleted = if probe_root.exists() {
        fs::remove_dir_all(probe_root).is_ok()
    } else {
        true
    };
    junction_deleted && root_deleted
}
