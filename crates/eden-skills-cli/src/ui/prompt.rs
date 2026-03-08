//! Interactive multi-select prompts for skill installation and removal.
//!
//! Provides [`SkillSelectTheme`] for rendering checkbox-style prompts and
//! [`prompt_skill_multi_select`] for driving the interactive selection loop.

use std::collections::HashMap;

use dialoguer::console::{measure_text_width, Key, Style, Term};
use eden_skills_core::error::EdenError;

#[cfg(windows)]
use std::iter::once;
#[cfg(windows)]
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
#[cfg(windows)]
use windows_sys::Win32::System::Console::{
    FlushConsoleInputBuffer, WriteConsoleInputW, INPUT_RECORD, INPUT_RECORD_0, KEY_EVENT,
    KEY_EVENT_RECORD, KEY_EVENT_RECORD_0,
};

use super::context::UiContext;

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

/// Best-effort wake-up for a blocked interactive key read.
///
/// On Windows, dialoguer's `Term::read_key()` may remain blocked after
/// `Ctrl+C` until another key event arrives. We inject a synthetic
/// `Escape` keypress into the console input buffer so the shared
/// selection prompt can observe the pending prompt interrupt
/// immediately and exit without waiting for an extra user keystroke.
#[cfg(windows)]
pub fn wake_interactive_prompt_input() {
    let Some((handle, owns_handle)) = super::format::open_windows_console_input_handle() else {
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
