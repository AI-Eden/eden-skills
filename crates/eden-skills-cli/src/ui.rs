//! Terminal UI primitives for the eden-skills CLI.
//!
//! Provides [`UiContext`] — the central entry point for color-aware output,
//! status symbols, action prefixes, spinners, and table construction.
//! All human-mode rendering flows through this module so that JSON mode,
//! non-TTY pipes, `NO_COLOR`/`FORCE_COLOR`, and `--color` flags are
//! handled consistently in one place.

use std::collections::HashMap;
#[cfg(unix)]
use std::fs::File;
#[cfg(unix)]
use std::fs::OpenOptions;
use std::io::IsTerminal;
#[cfg(windows)]
use std::iter::once;
#[cfg(unix)]
use std::os::fd::{AsRawFd, RawFd};
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::time::Duration;

use clap::ValueEnum;
use comfy_table::{presets, Cell, ContentArrangement, Table};
use dialoguer::console::{
    measure_text_width, set_colors_enabled, set_colors_enabled_stderr, Key, Style, Term,
};
use eden_skills_core::error::EdenError;
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
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
    FlushConsoleInputBuffer, GetConsoleMode, GetStdHandle, SetConsoleMode, WriteConsoleInputW,
    CONSOLE_MODE, ENABLE_ECHO_INPUT, ENABLE_LINE_INPUT, INPUT_RECORD, INPUT_RECORD_0, KEY_EVENT,
    KEY_EVENT_RECORD, KEY_EVENT_RECORD_0, STD_INPUT_HANDLE,
};

/// When to emit ANSI color sequences.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ColorWhen {
    Auto,
    Always,
    Never,
}

impl ColorWhen {
    const fn as_u8(self) -> u8 {
        match self {
            Self::Auto => 0,
            Self::Always => 1,
            Self::Never => 2,
        }
    }

    const fn from_u8(value: u8) -> Self {
        match value {
            1 => Self::Always,
            2 => Self::Never,
            _ => Self::Auto,
        }
    }
}

static COLOR_WHEN_OVERRIDE: AtomicU8 = AtomicU8::new(ColorWhen::Auto.as_u8());
static COLOR_ENABLED_OVERRIDE: AtomicBool = AtomicBool::new(true);

/// Initialize the global color override from the `--color` flag and JSON mode.
///
/// Must be called once during CLI startup before any output is produced.
pub fn configure_color_output(color_when: ColorWhen, json_mode: bool) {
    #[cfg(windows)]
    {
        enable_ansi_support::enable_ansi_support().ok();
    }

    COLOR_WHEN_OVERRIDE.store(color_when.as_u8(), Ordering::Relaxed);
    let enabled = resolve_colors_enabled(color_when, json_mode, stdout_is_tty());
    COLOR_ENABLED_OVERRIDE.store(enabled, Ordering::Relaxed);
    set_colors_enabled(enabled);
    set_colors_enabled_stderr(enabled);
    owo_colors::set_override(enabled);
}

/// Query whether color output is globally enabled.
pub fn color_output_enabled() -> bool {
    COLOR_ENABLED_OVERRIDE.load(Ordering::Relaxed)
}

/// Semantic symbols rendered in human-mode output (e.g. `✓`, `✗`, `!`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusSymbol {
    Success,
    Failure,
    Skipped,
    Warning,
}

/// A single skill entry rendered in interactive checkbox prompts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkillSelectItem<'a> {
    pub name: &'a str,
    pub description: &'a str,
}

/// Result of an interactive multi-select prompt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkillSelectOutcome {
    Selected(Vec<usize>),
    Cancelled,
    Interrupted,
}

/// Shared renderer for checkbox-based skill selection.
pub struct SkillSelectTheme {
    descriptions: HashMap<String, String>,
    colors_enabled: bool,
    terminal_width: usize,
}

impl SkillSelectTheme {
    /// Build a theme from a list of selectable skills.
    pub fn new(items: &[SkillSelectItem<'_>], colors_enabled: bool) -> Self {
        let descriptions = items
            .iter()
            .filter_map(|item| {
                let trimmed = item.description.trim();
                (!trimmed.is_empty()).then(|| (item.name.to_string(), trimmed.to_string()))
            })
            .collect::<HashMap<_, _>>();
        Self {
            descriptions,
            colors_enabled,
            terminal_width: resolve_terminal_width(),
        }
    }

    /// Render a single item preview for tests and prompt formatting.
    pub fn format_item_preview(&self, text: &str, checked: bool, active: bool) -> String {
        let display_name = truncate_to_width(
            text,
            self.terminal_width
                .saturating_sub(SKILL_SELECT_PREFIX_WIDTH),
        );
        let checkbox = self.render_checkbox(checked, active);
        let label = self.render_label(&display_name, checked, active);

        let mut rendered = format!("   {checkbox} {label}");
        if let Some(description) =
            self.format_inline_description(text, &display_name, checked, active)
        {
            rendered.push_str(&description);
        }
        rendered
    }

    /// Render the prompt title line.
    pub fn format_prompt_line(&self, title: &str) -> String {
        let icon = self.apply_style("◆", Style::new().cyan());
        let hint = self.apply_style("(space to toggle)", Style::new().color256(245));
        format!("{icon}  {title} {hint}")
    }

    /// Render an optional discovery summary line.
    pub fn format_header_line(&self, header: &str) -> String {
        let icon = self.apply_style("◇", Style::new().green());
        format!("{icon}  {header}")
    }

    /// Render an overflow indicator row.
    pub fn format_overflow_line(&self) -> String {
        format!("  {}", self.apply_style("...", Style::new().color256(245)))
    }

    /// Render a full prompt frame for tests and runtime drawing.
    pub fn render_frame(
        &self,
        header: Option<&str>,
        title: &str,
        items: &[SkillSelectItem<'_>],
        selected: &[bool],
        cursor: usize,
        viewport_slots: usize,
    ) -> Vec<String> {
        let mut lines = Vec::new();
        if let Some(header) = header {
            lines.push(self.format_header_line(header));
            lines.push(String::new());
        }
        lines.push(self.format_prompt_line(title));

        if items.is_empty() {
            return lines;
        }

        let (start, end, show_top, show_bottom) =
            compute_skill_select_viewport(items.len(), cursor, viewport_slots.max(1));
        if show_top {
            lines.push(self.format_overflow_line());
        }
        for (index, item) in items.iter().enumerate().take(end).skip(start) {
            lines.push(self.format_item_preview(
                item.name,
                selected.get(index).copied().unwrap_or(false),
                index == cursor,
            ));
        }
        if show_bottom {
            lines.push(self.format_overflow_line());
        }
        lines
    }

    fn format_inline_description(
        &self,
        text: &str,
        display_name: &str,
        checked: bool,
        active: bool,
    ) -> Option<String> {
        if !active && !checked {
            return None;
        }
        let description = self.descriptions.get(text)?.trim();
        if description.is_empty() {
            return None;
        }

        let capped = truncate_to_char_limit(description, SKILL_SELECT_DESCRIPTION_CHAR_LIMIT);
        let available_width = self.terminal_width.saturating_sub(
            SKILL_SELECT_PREFIX_WIDTH
                + measure_text_width(display_name)
                + SKILL_SELECT_DESCRIPTION_PADDING,
        );
        if available_width < 4 {
            return None;
        }

        let truncated = truncate_to_width(&capped, available_width);
        let rendered = format!(" ({truncated})");
        Some(self.apply_style(&rendered, Style::new().color256(245)))
    }

    fn render_checkbox(&self, checked: bool, active: bool) -> String {
        match (checked, active) {
            (true, _) => self.apply_style("◼", Style::new().green()),
            (false, true) => self.apply_style("◻", Style::new().cyan()),
            (false, false) => self.apply_style("◻", Style::new().color256(245)),
        }
    }

    fn render_label(&self, text: &str, checked: bool, active: bool) -> String {
        match (checked, active) {
            (_, true) => text.to_string(),
            (true, false) => self.apply_style(text, Style::new().color256(250)),
            (false, false) => self.apply_style(text, Style::new().color256(245)),
        }
    }

    fn apply_style(&self, text: &str, style_obj: Style) -> String {
        if self.colors_enabled {
            style_obj.apply_to(text).to_string()
        } else {
            text.to_string()
        }
    }
}

/// Central context for all human-mode output decisions.
///
/// Captures TTY state, color/symbol policy, and JSON mode at construction
/// time so that every rendering call produces output consistent with the
/// user's terminal capabilities and CLI flags.
#[derive(Debug, Clone)]
pub struct UiContext {
    json_mode: bool,
    stdout_is_tty: bool,
    no_color: bool,
    force_color: bool,
    ci: bool,
    color_when: ColorWhen,
}

impl UiContext {
    /// Construct a context by snapshotting the current environment.
    pub fn from_env(json_mode: bool) -> Self {
        let stdout_is_tty = stdout_is_tty();
        Self {
            json_mode,
            stdout_is_tty,
            no_color: env_var_present("NO_COLOR"),
            force_color: env_var_present("FORCE_COLOR"),
            ci: env_var_present("CI"),
            color_when: configured_color_when(),
        }
    }

    /// Whether JSON output mode is active.
    pub fn json_mode(&self) -> bool {
        self.json_mode
    }

    /// Whether ANSI colors should be emitted in the current context.
    ///
    /// Precedence: `--json` → `--color` flag → `NO_COLOR` → `FORCE_COLOR` → `CI` → TTY.
    pub fn colors_enabled(&self) -> bool {
        if self.json_mode {
            return false;
        }
        match self.color_when {
            ColorWhen::Never => false,
            ColorWhen::Always => true,
            ColorWhen::Auto => {
                if self.no_color {
                    return false;
                }
                if self.force_color {
                    return true;
                }
                if self.ci {
                    return false;
                }
                self.stdout_is_tty
            }
        }
    }

    /// Whether Unicode status symbols (✓, ✗, etc.) should be emitted.
    pub fn symbols_enabled(&self) -> bool {
        let force_symbols = matches!(self.color_when, ColorWhen::Always) || self.force_color;
        !self.json_mode && (self.stdout_is_tty || force_symbols) && !self.ci
    }

    /// Whether a progress spinner should be displayed.
    pub fn spinner_enabled(&self) -> bool {
        !self.json_mode && self.stdout_is_tty && !self.ci
    }

    /// Whether interactive prompts (confirm, input) are allowed.
    pub fn interactive_enabled(&self) -> bool {
        !self.json_mode && self.stdout_is_tty && !self.ci
    }

    /// Render a colored status symbol string for the given semantic value.
    pub fn status_symbol(&self, symbol: StatusSymbol) -> String {
        let raw = match symbol {
            StatusSymbol::Success => "✓",
            StatusSymbol::Failure => "✗",
            StatusSymbol::Skipped => "·",
            StatusSymbol::Warning => "!",
        };
        if !self.colors_enabled() {
            return raw.to_string();
        }
        match symbol {
            StatusSymbol::Success => raw.green().to_string(),
            StatusSymbol::Failure => raw.red().to_string(),
            StatusSymbol::Skipped => raw.dimmed().to_string(),
            StatusSymbol::Warning => raw.yellow().to_string(),
        }
    }

    /// Render a right-padded, bold-cyan action label (e.g. `" Install"`).
    pub fn action_prefix(&self, action: &str) -> String {
        let padded = format!("{action:>8}");
        if self.colors_enabled() {
            padded.cyan().bold().to_string()
        } else {
            padded
        }
    }

    /// Abbreviate a path with `~` and colorize it for human output.
    ///
    /// Paths are rendered in cyan when colors are enabled. JSON mode and
    /// `--color never` keep the abbreviated path as plain text.
    pub fn styled_path(&self, path: &str) -> String {
        let abbreviated = abbreviate_home_path(path);
        if self.colors_enabled() {
            abbreviated.cyan().to_string()
        } else {
            abbreviated
        }
    }

    /// Style a skill identifier for table cells and other human-facing output.
    pub fn styled_skill_id(&self, skill_id: &str) -> String {
        if self.colors_enabled() {
            skill_id.bold().magenta().to_string()
        } else {
            skill_id.to_string()
        }
    }

    /// Style an agent name for table cells and other human-facing output.
    pub fn styled_agent_name(&self, agent_name: &str) -> String {
        if self.colors_enabled() {
            agent_name.magenta().to_string()
        } else {
            agent_name.to_string()
        }
    }

    /// Style a version string for table cells and other human-facing output.
    pub fn styled_version(&self, version: &str) -> String {
        if self.colors_enabled() {
            version.yellow().to_string()
        } else {
            version.to_string()
        }
    }

    /// Style a semantic status label for table output.
    pub fn styled_status(&self, status: &str) -> String {
        if !self.colors_enabled() {
            return status.to_string();
        }
        match status {
            "up-to-date" | "ok" | "noop" => status.green().to_string(),
            "failed" | "error" => status.red().to_string(),
            "warning" | "conflict" => status.yellow().to_string(),
            "skipped" | "missing" => status.dimmed().to_string(),
            "cloned" | "updated" | "new commit" => status.cyan().to_string(),
            _ => status.to_string(),
        }
    }

    /// Style secondary detail text such as modes or explanatory suffixes.
    pub fn styled_secondary(&self, text: &str) -> String {
        if self.colors_enabled() {
            text.dimmed().to_string()
        } else {
            text.to_string()
        }
    }

    /// Style generic cyan content such as source labels.
    pub fn styled_cyan(&self, text: &str) -> String {
        if self.colors_enabled() {
            text.cyan().to_string()
        } else {
            text.to_string()
        }
    }

    /// Style warning-emphasis text such as list truncation markers.
    pub fn styled_warning_text(&self, text: &str) -> String {
        if self.colors_enabled() {
            text.yellow().to_string()
        } else {
            text.to_string()
        }
    }

    /// Render the canonical hint prefix used across CLI guidance lines.
    pub fn hint_prefix(&self) -> String {
        if self.colors_enabled() {
            "~>".magenta().to_string()
        } else {
            "~>".to_string()
        }
    }

    /// Render a signal-driven cancellation line for interactive prompts.
    pub fn signal_cancelled_line(&self, action: &str) -> String {
        let content = format!("◆  {action} canceled");
        if self.colors_enabled() {
            content.red().to_string()
        } else {
            content
        }
    }

    /// Style a table header label.
    pub fn style_table_header(&self, header: &str) -> String {
        if self.colors_enabled() {
            header.bold().to_string()
        } else {
            header.to_string()
        }
    }

    /// Create a [`Table`] pre-configured for the current terminal context.
    ///
    /// TTY output uses content-driven column widths with bold headers when
    /// colors are enabled; non-TTY output keeps ASCII borders capped at
    /// 80 columns with dynamic wrapping.
    pub fn table(&self, headers: &[&str]) -> Table {
        let mut table = Table::new();
        let human_tty = self.stdout_is_tty && !self.ci;
        if human_tty {
            table.load_preset(presets::UTF8_FULL_CONDENSED);
            table.set_content_arrangement(ContentArrangement::Disabled);
        } else {
            table.load_preset(presets::ASCII_FULL_CONDENSED);
            table.set_width(80);
            table.set_content_arrangement(ContentArrangement::Dynamic);
        }

        let header_cells = headers
            .iter()
            .map(|header| Cell::new(self.style_table_header(header)))
            .collect::<Vec<_>>();
        table.set_header(header_cells);
        table
    }

    /// Start a terminal spinner with an action label and detail message.
    ///
    /// Returns a no-op spinner when the terminal does not support animation.
    pub fn spinner(&self, action: &str, detail: String) -> UiSpinner {
        if !self.spinner_enabled() {
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
        progress.set_prefix(self.action_prefix(action));
        progress.set_message(detail.clone());
        progress.enable_steady_tick(Duration::from_millis(100));

        UiSpinner {
            action: action.to_string(),
            detail,
            _input_guard: SpinnerInputGuard::new(),
            progress: Some(progress),
        }
    }
}

/// Prompt the user to select skills via the shared checkbox selector.
pub fn prompt_skill_multi_select(
    ui: &UiContext,
    title: &str,
    items: &[SkillSelectItem<'_>],
    test_env_var: &str,
    header: Option<String>,
) -> Result<SkillSelectOutcome, EdenError> {
    if let Some(selection) = parse_test_multi_select_env(test_env_var, items.len())? {
        return Ok(selection);
    }

    let term = interactive_term();
    let theme = SkillSelectTheme::new(items, ui.colors_enabled());
    let _interrupt_guard = crate::signal::PromptInterruptGuard::new();

    if !term.features().is_attended() {
        return Err(EdenError::Runtime(
            "interactive prompt failed: no attended terminal available".to_string(),
        ));
    }

    let _cursor_guard = CursorGuard::new(&term);
    term.hide_cursor().map_err(EdenError::Io)?;

    let mut selected = vec![false; items.len()];
    let mut cursor = 0usize;
    let mut last_render_height = 0usize;
    let preserved_prefix_lines = skill_select_static_prefix_line_count(header.is_some());

    loop {
        let viewport_slots = resolve_skill_select_viewport_slots(&term, header.is_some());
        let lines = theme.render_frame(
            header.as_deref(),
            title,
            items,
            &selected,
            cursor,
            viewport_slots,
        );
        redraw_skill_select_frame(&term, &lines, &mut last_render_height).map_err(EdenError::Io)?;

        if crate::signal::take_prompt_interrupt() {
            preserve_skill_select_prefix(&term, &mut last_render_height, preserved_prefix_lines)
                .map_err(EdenError::Io)?;
            return Ok(SkillSelectOutcome::Interrupted);
        }

        let key = match term.read_key() {
            Ok(key) => key,
            Err(err) if err.kind() == std::io::ErrorKind::Interrupted => {
                let _ = crate::signal::take_prompt_interrupt();
                preserve_skill_select_prefix(
                    &term,
                    &mut last_render_height,
                    preserved_prefix_lines,
                )
                .map_err(EdenError::Io)?;
                return Ok(SkillSelectOutcome::Interrupted);
            }
            Err(err) => return Err(EdenError::Io(err)),
        };

        if crate::signal::take_prompt_interrupt() {
            preserve_skill_select_prefix(&term, &mut last_render_height, preserved_prefix_lines)
                .map_err(EdenError::Io)?;
            return Ok(SkillSelectOutcome::Interrupted);
        }

        match key {
            Key::ArrowUp | Key::Char('k') => {
                cursor = cursor.saturating_sub(1);
            }
            Key::ArrowDown | Key::Char('j') => {
                if cursor + 1 < items.len() {
                    cursor += 1;
                }
            }
            Key::Char(' ') => {
                if let Some(current) = selected.get_mut(cursor) {
                    *current = !*current;
                }
            }
            Key::Enter => {
                let indices = selected
                    .iter()
                    .enumerate()
                    .filter_map(|(index, is_selected)| is_selected.then_some(index))
                    .collect::<Vec<_>>();
                clear_skill_select_frame(&term, &mut last_render_height).map_err(EdenError::Io)?;
                return Ok(SkillSelectOutcome::Selected(indices));
            }
            Key::Escape | Key::CtrlC | Key::Char('q') => {
                preserve_skill_select_prefix(
                    &term,
                    &mut last_render_height,
                    preserved_prefix_lines,
                )
                .map_err(EdenError::Io)?;
                return Ok(SkillSelectOutcome::Interrupted);
            }
            _ => {}
        }
    }
}

struct CursorGuard<'a> {
    term: &'a Term,
}

impl<'a> CursorGuard<'a> {
    fn new(term: &'a Term) -> Self {
        Self { term }
    }
}

impl Drop for CursorGuard<'_> {
    fn drop(&mut self) {
        let _ = self.term.show_cursor();
    }
}

struct SpinnerInputGuard {
    #[cfg(unix)]
    _inner: Option<UnixSpinnerInputGuard>,
    #[cfg(windows)]
    _inner: Option<WindowsSpinnerInputGuard>,
}

impl SpinnerInputGuard {
    fn new() -> Self {
        Self {
            #[cfg(unix)]
            _inner: UnixSpinnerInputGuard::new(),
            #[cfg(windows)]
            _inner: WindowsSpinnerInputGuard::new(),
        }
    }
}

/// Best-effort wake-up for a blocked interactive key read.
///
/// On Windows, dialoguer's `Term::read_key()` may remain blocked after
/// `Ctrl+C` until another key event arrives. We inject a synthetic
/// `Escape` keypress into the console input buffer so the shared
/// selection prompt can observe the pending prompt interrupt
/// immediately and exit without waiting for an extra user keystroke.
#[cfg(windows)]
pub fn wake_interactive_prompt_input() {
    let Some((handle, owns_handle)) = open_windows_console_input_handle() else {
        return;
    };

    unsafe {
        let _ = FlushConsoleInputBuffer(handle);
        let records = windows_prompt_wake_input_records();
        let mut events_written = 0;
        let _ = WriteConsoleInputW(
            handle,
            records.as_ptr(),
            records.len() as u32,
            &mut events_written,
        );
        if owns_handle {
            CloseHandle(handle);
        }
    }
}

/// Non-Windows platforms do not need prompt wake-up injection.
#[cfg(not(windows))]
pub fn wake_interactive_prompt_input() {}

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
fn windows_prompt_wake_input_records() -> [INPUT_RECORD; 2] {
    [
        windows_prompt_wake_input_record(true),
        windows_prompt_wake_input_record(false),
    ]
}

#[cfg(windows)]
fn windows_prompt_wake_input_record(key_down: bool) -> INPUT_RECORD {
    INPUT_RECORD {
        EventType: KEY_EVENT as u16,
        Event: INPUT_RECORD_0 {
            KeyEvent: KEY_EVENT_RECORD {
                bKeyDown: i32::from(key_down),
                wRepeatCount: 1,
                wVirtualKeyCode: 27,
                wVirtualScanCode: 0,
                uChar: KEY_EVENT_RECORD_0 { UnicodeChar: 27 },
                dwControlKeyState: 0,
            },
        },
    }
}

/// An in-flight spinner that can be resolved as success or failure.
pub struct UiSpinner {
    action: String,
    detail: String,
    _input_guard: SpinnerInputGuard,
    progress: Option<ProgressBar>,
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

fn env_var_present(name: &str) -> bool {
    std::env::var(name)
        .ok()
        .is_some_and(|value| !value.is_empty())
}

fn configured_color_when() -> ColorWhen {
    ColorWhen::from_u8(COLOR_WHEN_OVERRIDE.load(Ordering::Relaxed))
}

fn resolve_home_dir() -> Option<String> {
    std::env::var("HOME")
        .ok()
        .filter(|value| !value.is_empty())
        .or_else(|| {
            std::env::var("USERPROFILE")
                .ok()
                .filter(|value| !value.is_empty())
        })
}

fn resolve_colors_enabled(color_when: ColorWhen, json_mode: bool, stdout_is_tty: bool) -> bool {
    if json_mode {
        return false;
    }
    match color_when {
        ColorWhen::Never => false,
        ColorWhen::Always => true,
        ColorWhen::Auto => {
            let no_color = env_var_present("NO_COLOR");
            if no_color {
                return false;
            }
            let force_color = env_var_present("FORCE_COLOR");
            if force_color {
                return true;
            }
            let ci = env_var_present("CI");
            if ci {
                return false;
            }
            stdout_is_tty
        }
    }
}

fn stdout_is_tty() -> bool {
    forced_tty_for_tests() || std::io::stdout().is_terminal()
}

fn forced_tty_for_tests() -> bool {
    std::env::var("EDEN_SKILLS_FORCE_TTY")
        .ok()
        .is_some_and(|value| value == "1")
}

const SKILL_SELECT_PREFIX_WIDTH: usize = 5;
const SKILL_SELECT_DESCRIPTION_PADDING: usize = 3;
const SKILL_SELECT_DESCRIPTION_CHAR_LIMIT: usize = 57;

fn parse_test_multi_select_env(
    name: &str,
    item_count: usize,
) -> Result<Option<SkillSelectOutcome>, EdenError> {
    let Ok(raw) = std::env::var(name) else {
        return Ok(None);
    };
    if raw.trim().eq_ignore_ascii_case("interrupt") {
        return Ok(Some(SkillSelectOutcome::Interrupted));
    }

    let mut indices = Vec::new();
    for token in raw
        .split(',')
        .map(str::trim)
        .filter(|token| !token.is_empty())
    {
        let index = token.parse::<usize>().map_err(|_| {
            EdenError::InvalidArguments(format!("invalid interactive selection index: '{token}'"))
        })?;
        if index >= item_count {
            return Err(EdenError::InvalidArguments(format!(
                "interactive selection index out of range: {index}"
            )));
        }
        if !indices.contains(&index) {
            indices.push(index);
        }
    }

    Ok(Some(SkillSelectOutcome::Selected(indices)))
}

fn resolve_terminal_width() -> usize {
    std::env::var("COLUMNS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|width| *width > 0)
        .or_else(|| {
            interactive_term()
                .size_checked()
                .map(|(_, width)| width as usize)
        })
        .unwrap_or(80)
}

fn interactive_term() -> Term {
    let stderr = Term::stderr();
    if stderr.features().is_attended() {
        stderr
    } else {
        Term::stdout()
    }
}

fn resolve_skill_select_viewport_slots(term: &Term, has_header: bool) -> usize {
    let (rows, _) = term.size();
    let reserved_lines = skill_select_static_prefix_line_count(has_header);
    (rows as usize).saturating_sub(reserved_lines).max(1)
}

fn skill_select_static_prefix_line_count(has_header: bool) -> usize {
    1 + usize::from(has_header) * 2
}

fn compute_skill_select_viewport(
    item_count: usize,
    cursor: usize,
    viewport_slots: usize,
) -> (usize, usize, bool, bool) {
    if item_count <= viewport_slots {
        return (0, item_count, false, false);
    }

    let mut visible_count = item_count.min(viewport_slots);
    loop {
        let max_start = item_count.saturating_sub(visible_count);
        let mut start = cursor.saturating_sub(visible_count / 2);
        start = start.min(max_start);
        let end = start + visible_count;
        let show_top = start > 0;
        let show_bottom = end < item_count;
        let total_lines = visible_count + usize::from(show_top) + usize::from(show_bottom);

        if total_lines <= viewport_slots {
            return (start, end, show_top, show_bottom);
        }

        if visible_count == 1 {
            return (
                cursor.min(item_count - 1),
                cursor.min(item_count - 1) + 1,
                false,
                false,
            );
        }

        visible_count -= 1;
    }
}

fn redraw_skill_select_frame(
    term: &Term,
    lines: &[String],
    last_render_height: &mut usize,
) -> std::io::Result<()> {
    clear_skill_select_frame(term, last_render_height)?;
    let frame = render_skill_select_frame_text(lines);
    if !frame.is_empty() {
        term.write_str(&frame)?;
    }
    term.flush()?;
    *last_render_height = lines.len();
    Ok(())
}

fn clear_skill_select_frame(term: &Term, last_render_height: &mut usize) -> std::io::Result<()> {
    if *last_render_height > 0 {
        term.clear_line()?;
        for _ in 1..*last_render_height {
            term.move_cursor_up(1)?;
            term.clear_line()?;
        }
        *last_render_height = 0;
    }
    Ok(())
}

fn preserve_skill_select_prefix(
    term: &Term,
    last_render_height: &mut usize,
    preserved_prefix_lines: usize,
) -> std::io::Result<()> {
    let body_height = last_render_height.saturating_sub(preserved_prefix_lines);
    if body_height == 0 {
        return Ok(());
    }

    term.clear_line()?;
    for _ in 1..body_height {
        term.move_cursor_up(1)?;
        term.clear_line()?;
    }
    *last_render_height = preserved_prefix_lines;
    Ok(())
}

fn render_skill_select_frame_text(lines: &[String]) -> String {
    lines.join("\n")
}

#[cfg(windows)]
fn open_windows_console_input_handle() -> Option<(HANDLE, bool)> {
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

fn truncate_to_char_limit(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }

    let mut rendered = text.chars().take(max_chars).collect::<String>();
    rendered.push_str("...");
    rendered
}

fn truncate_to_width(text: &str, max_width: usize) -> String {
    if measure_text_width(text) <= max_width {
        return text.to_string();
    }

    let keep_width = max_width.saturating_sub(3);
    let mut rendered = String::new();
    let mut used_width = 0;
    for ch in text.chars() {
        let ch_width = measure_text_width(&ch.to_string());
        if used_width + ch_width > keep_width {
            break;
        }
        rendered.push(ch);
        used_width += ch_width;
    }
    rendered.push_str("...");
    rendered
}

#[cfg(all(test, windows))]
mod tests {
    use super::{windows_prompt_wake_input_records, KEY_EVENT};

    #[test]
    fn windows_prompt_wake_records_emit_escape_keypress() {
        let records = windows_prompt_wake_input_records();

        assert_eq!(records[0].EventType, KEY_EVENT as u16);
        assert_eq!(records[1].EventType, KEY_EVENT as u16);

        unsafe {
            let key_down = records[0].Event.KeyEvent;
            assert_ne!(key_down.bKeyDown, 0);
            assert_eq!(key_down.wVirtualKeyCode, 27);
            assert_eq!(key_down.uChar.UnicodeChar, 27);

            let key_up = records[1].Event.KeyEvent;
            assert_eq!(key_up.bKeyDown, 0);
            assert_eq!(key_up.wVirtualKeyCode, 27);
            assert_eq!(key_up.uChar.UnicodeChar, 27);
        }
    }
}
