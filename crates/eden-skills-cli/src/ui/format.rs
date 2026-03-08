//! Path and URL formatting utilities, plus the [`UiSpinner`] type.
//!
//! Provides `~`-abbreviated paths, `owner/repo` GitHub URL shortening,
//! and the spinner wrapper that guards terminal input during animation.

use std::time::Duration;

#[cfg(unix)]
use std::fs::File;
#[cfg(unix)]
use std::fs::OpenOptions;
#[cfg(unix)]
use std::io::IsTerminal;
#[cfg(unix)]
use std::os::fd::{AsRawFd, RawFd};

#[cfg(windows)]
use std::iter::once;
#[cfg(windows)]
use windows_sys::Win32::Foundation::{
    CloseHandle, GENERIC_READ, GENERIC_WRITE, HANDLE, INVALID_HANDLE_VALUE,
};
#[cfg(windows)]
use windows_sys::Win32::Storage::FileSystem::{
    CreateFileW, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
};
#[cfg(windows)]
use windows_sys::Win32::System::Console::{
    FlushConsoleInputBuffer, GetConsoleMode, GetStdHandle, SetConsoleMode, CONSOLE_MODE,
    ENABLE_ECHO_INPUT, ENABLE_LINE_INPUT, STD_INPUT_HANDLE,
};

use indicatif::{ProgressBar, ProgressStyle};

use super::context::UiContext;
use super::table::StatusSymbol;

/// An in-flight spinner that can be resolved as success or failure.
pub struct UiSpinner {
    pub(crate) action: String,
    pub(crate) detail: String,
    pub(crate) _input_guard: SpinnerInputGuard,
    pub(crate) progress: Option<ProgressBar>,
}

impl UiSpinner {
    /// Stop the spinner and print a success line.
    pub fn finish_success(self, ui: &UiContext) {
        if let Some(progress) = self.progress {
            progress.finish_and_clear();
            println!(
                "{}  {} {} done",
                ui.action_prefix(&self.action),
                self.detail,
                ui.status_symbol(StatusSymbol::Success)
            );
        }
    }

    /// Stop the spinner and print a failure line with a summary.
    pub fn finish_failure(self, ui: &UiContext, summary: &str) {
        if let Some(progress) = self.progress {
            progress.finish_and_clear();
            println!(
                "{}  {} {} {}",
                ui.action_prefix(&self.action),
                self.detail,
                ui.status_symbol(StatusSymbol::Failure),
                summary
            );
        }
    }
}

/// Replace the `$HOME` prefix in a path with `~` for display.
///
/// Returns the original string unchanged if it does not start with the
/// home directory or if `$HOME`/`USERPROFILE` is unset.
pub fn abbreviate_home_path(path: &str) -> String {
    let Some(home_dir) = resolve_home_dir() else {
        return path.to_string();
    };
    let home_trimmed = home_dir.trim_end_matches(['/', '\\']);
    if home_trimmed.is_empty() {
        return path.to_string();
    }

    let normalized_home = home_trimmed.replace('\\', "/");
    let normalized_path = path.replace('\\', "/");

    if normalized_path == normalized_home {
        return "~".to_string();
    }

    if let Some(remainder) = normalized_path.strip_prefix(&normalized_home) {
        if remainder.starts_with('/') {
            return format!("~{remainder}");
        }
    }

    path.to_string()
}

/// Extract `owner/repo` from a GitHub URL for concise display.
///
/// Recognises `https://github.com/`, `http://github.com/`, and
/// `git@github.com:` prefixes. Non-GitHub URLs are returned verbatim.
pub fn abbreviate_repo_url(url: &str) -> String {
    let remainder = if let Some(rest) = url.strip_prefix("https://github.com/") {
        rest
    } else if let Some(rest) = url.strip_prefix("http://github.com/") {
        rest
    } else if let Some(rest) = url.strip_prefix("git@github.com:") {
        rest
    } else {
        return url.to_string();
    };

    let path = remainder
        .split(['?', '#'])
        .next()
        .unwrap_or(remainder)
        .trim_end_matches('/');
    let mut parts = path.split('/');
    let Some(owner) = parts.next() else {
        return url.to_string();
    };
    let Some(repo_raw) = parts.next() else {
        return url.to_string();
    };
    if owner.is_empty() || repo_raw.is_empty() || parts.next().is_some() {
        return url.to_string();
    }

    let repo = repo_raw.strip_suffix(".git").unwrap_or(repo_raw);
    if repo.is_empty() {
        return url.to_string();
    }
    format!("{owner}/{repo}")
}

pub(crate) fn resolve_home_dir() -> Option<String> {
    std::env::var("HOME")
        .ok()
        .filter(|value| !value.is_empty())
        .or_else(|| {
            std::env::var("USERPROFILE")
                .ok()
                .filter(|value| !value.is_empty())
        })
}

pub(crate) fn create_spinner(action: &str, detail: String, ui: &UiContext) -> UiSpinner {
    if !ui.spinner_enabled() {
        return UiSpinner {
            action: action.to_string(),
            detail,
            _input_guard: SpinnerInputGuard::new(),
            progress: None,
        };
    }

    let progress = ProgressBar::new_spinner();
    let style = ProgressStyle::with_template("{prefix}  {msg} {spinner}")
        .unwrap_or_else(|_| ProgressStyle::default_spinner())
        .tick_strings(&["-", "\\", "|", "/"]);
    progress.set_style(style);
    progress.set_prefix(ui.action_prefix(action));
    progress.set_message(detail.clone());
    progress.enable_steady_tick(Duration::from_millis(100));

    UiSpinner {
        action: action.to_string(),
        detail,
        _input_guard: SpinnerInputGuard::new(),
        progress: Some(progress),
    }
}

pub(crate) struct SpinnerInputGuard {
    #[cfg(unix)]
    _inner: Option<UnixSpinnerInputGuard>,
    #[cfg(windows)]
    _inner: Option<WindowsSpinnerInputGuard>,
}

impl SpinnerInputGuard {
    pub(crate) fn new() -> Self {
        Self {
            #[cfg(unix)]
            _inner: UnixSpinnerInputGuard::new(),
            #[cfg(windows)]
            _inner: WindowsSpinnerInputGuard::new(),
        }
    }
}

#[cfg(unix)]
struct UnixSpinnerInputGuard {
    fd: RawFd,
    tty_file: Option<File>,
    original_termios: libc::termios,
}

#[cfg(unix)]
impl UnixSpinnerInputGuard {
    fn new() -> Option<Self> {
        let stdin = std::io::stdin();
        let (fd, tty_file) = if stdin.is_terminal() {
            (stdin.as_raw_fd(), None)
        } else {
            let tty_file = OpenOptions::new()
                .read(true)
                .write(true)
                .open("/dev/tty")
                .ok()?;
            (tty_file.as_raw_fd(), Some(tty_file))
        };

        let mut original = std::mem::MaybeUninit::<libc::termios>::uninit();
        let get_result = unsafe { libc::tcgetattr(fd, original.as_mut_ptr()) };
        if get_result != 0 {
            return None;
        }
        let original = unsafe { original.assume_init() };
        let mut muted = original;
        muted.c_lflag &= !libc::ECHO;

        let set_result = unsafe { libc::tcsetattr(fd, libc::TCSADRAIN, &muted) };
        if set_result != 0 {
            return None;
        }

        Some(Self {
            fd,
            tty_file,
            original_termios: original,
        })
    }
}

#[cfg(unix)]
impl Drop for UnixSpinnerInputGuard {
    fn drop(&mut self) {
        let _ = &self.tty_file;
        unsafe {
            libc::tcflush(self.fd, libc::TCIFLUSH);
            libc::tcsetattr(self.fd, libc::TCSADRAIN, &self.original_termios);
        }
    }
}

#[cfg(windows)]
struct WindowsSpinnerInputGuard {
    handle: HANDLE,
    owns_handle: bool,
    original_mode: CONSOLE_MODE,
}

#[cfg(windows)]
impl WindowsSpinnerInputGuard {
    fn new() -> Option<Self> {
        let (handle, owns_handle) = open_windows_console_input_handle()?;
        let mut original_mode = 0;
        let get_result = unsafe { GetConsoleMode(handle, &mut original_mode) };
        if get_result == 0 {
            if owns_handle {
                unsafe {
                    CloseHandle(handle);
                }
            }
            return None;
        }

        let muted_mode = original_mode & !ENABLE_ECHO_INPUT & !ENABLE_LINE_INPUT;
        let set_result = unsafe { SetConsoleMode(handle, muted_mode) };
        if set_result == 0 {
            if owns_handle {
                unsafe {
                    CloseHandle(handle);
                }
            }
            return None;
        }

        Some(Self {
            handle,
            owns_handle,
            original_mode,
        })
    }
}

#[cfg(windows)]
impl Drop for WindowsSpinnerInputGuard {
    fn drop(&mut self) {
        unsafe {
            FlushConsoleInputBuffer(self.handle);
            SetConsoleMode(self.handle, self.original_mode);
            if self.owns_handle {
                CloseHandle(self.handle);
            }
        }
    }
}

#[cfg(windows)]
pub(crate) fn open_windows_console_input_handle() -> Option<(HANDLE, bool)> {
    let std_handle = unsafe { GetStdHandle(STD_INPUT_HANDLE) };
    if windows_console_handle_supports_mode(std_handle) {
        return Some((std_handle, false));
    }

    let conin = "CONIN$".encode_utf16().chain(once(0)).collect::<Vec<_>>();
    let handle = unsafe {
        CreateFileW(
            conin.as_ptr(),
            GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            std::ptr::null(),
            OPEN_EXISTING,
            0,
            std::ptr::null_mut(),
        )
    };
    if windows_console_handle_supports_mode(handle) {
        Some((handle, true))
    } else {
        if handle != INVALID_HANDLE_VALUE && !handle.is_null() {
            unsafe {
                CloseHandle(handle);
            }
        }
        None
    }
}

#[cfg(windows)]
fn windows_console_handle_supports_mode(handle: HANDLE) -> bool {
    if handle.is_null() || handle == INVALID_HANDLE_VALUE {
        return false;
    }
    let mut mode = 0;
    unsafe { GetConsoleMode(handle, &mut mode) != 0 }
}
