use crossterm::event::{KeyCode, KeyModifiers, MouseEventKind};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::serial::InputMode;
use crate::state::{App, TerminalMode};

use super::common::{
    draw_error_popup, draw_info_bar, help_bar_height, help_spans, info_bar_spans, sep_span,
};

pub fn draw(app: &mut App, frame: &mut Frame) {
    let info_height = help_bar_height(info_bar_spans(app), frame.area().width).0;
    let help_height = help_bar_height(
        help_spans_for_mode(&app.terminal_mode, app.keyboard_enhanced),
        frame.area().width,
    )
    .0;

    let [info_area, output_area, help_area] = Layout::vertical([
        Constraint::Length(info_height),
        Constraint::Min(0),
        Constraint::Length(help_height),
    ])
    .areas(frame.area());

    let inner = Rect {
        x: output_area.x + 1,
        y: output_area.y + 1,
        width: output_area.width.saturating_sub(2),
        height: output_area.height.saturating_sub(2),
    };

    app.resize_parser(inner.height, inner.width);
    app.view.viewport_height = inner.height as usize;
    app.view.output_rect = inner;

    draw_info_bar(app, frame, info_area);

    let lines_source = app
        .view
        .frozen_lines
        .as_ref()
        .unwrap_or(&app.view.scrollback);
    let all_lines: Vec<&str> = lines_source
        .iter()
        .flat_map(|l| l.split_inclusive('\n'))
        .flat_map(|l| l.strip_suffix('\n').or(Some(l)))
        .collect();
    let total = all_lines.len();
    let height = inner.height as usize;
    let max_offset = total.saturating_sub(height);
    let scrolling = app.view.scroll_offset > 0;

    let block = Block::new().borders(Borders::ALL);
    frame.render_widget(block, output_area);

    let selection = app.selection.range();

    if scrolling {
        let end = total.saturating_sub(app.view.scroll_offset.min(max_offset));
        let start = end.saturating_sub(height);
        let visible: Vec<&str> = all_lines[start..end].to_vec();
        app.view.visible_lines = visible.iter().map(|l| l.to_string()).collect();

        let lines: Vec<ratatui::text::Line<'static>> = visible
            .iter()
            .enumerate()
            .map(|(row, line)| styled_line(line, row, selection))
            .collect();
        frame.render_widget(Paragraph::new(Text::from(lines)), inner);
    } else {
        let screen = app.view.parser.screen();
        let buf = frame.buffer_mut();
        app.view.visible_lines = vec![String::new(); inner.height as usize];
        for row in 0..inner.height {
            for col in 0..inner.width {
                if let Some(cell) = screen.cell(row, col) {
                    let ch = cell.contents();
                    if let Some(line) = app.view.visible_lines.get_mut(row as usize) {
                        line.push_str(if ch.is_empty() { " " } else { ch });
                    }
                    let selected = is_selected(selection, row as usize, col as usize);
                    if ch.is_empty() && !selected {
                        continue;
                    }
                    let mut style = Style::default();
                    style = style.fg(vt100_color(cell.fgcolor()));
                    style = style.bg(vt100_color(cell.bgcolor()));
                    if cell.bold() {
                        style = style.add_modifier(Modifier::BOLD);
                    }
                    if cell.italic() {
                        style = style.add_modifier(Modifier::ITALIC);
                    }
                    if cell.underline() {
                        style = style.add_modifier(Modifier::UNDERLINED);
                    }
                    if cell.inverse() {
                        style = style.add_modifier(Modifier::REVERSED);
                    }
                    if selected {
                        style = style.add_modifier(Modifier::REVERSED);
                    }
                    let symbol = if ch.is_empty() { " " } else { ch };
                    buf[(inner.x + col, inner.y + row)]
                        .set_symbol(symbol)
                        .set_style(style);
                }
            }
        }
        let (crow, ccol) = screen.cursor_position();
        let (mut col, mut row) = (ccol, crow);
        if app.input_mode == InputMode::Line {
            let style = Style::default().fg(Color::Cyan);
            let mut utf8 = [0u8; 4];
            for ch in app.line.buffer.chars() {
                if col >= inner.width {
                    col = 0;
                    row = row.saturating_add(1);
                }
                if row >= inner.height {
                    break;
                }
                let symbol = ch.encode_utf8(&mut utf8);
                buf[(inner.x + col, inner.y + row)]
                    .set_symbol(symbol)
                    .set_style(style);
                col += 1;
            }
        }
        frame.set_cursor_position((
            (inner.x + col).min(inner.x + inner.width - 1),
            (inner.y + row).min(inner.y + inner.height - 1),
        ));
    }

    let (_, help_lines) = help_bar_height(
        help_spans_for_mode(&app.terminal_mode, app.keyboard_enhanced),
        help_area.width,
    );
    let help = Paragraph::new(Text::from(help_lines)).block(Block::new().borders(Borders::ALL));
    frame.render_widget(help, help_area);
    draw_error_popup(app, frame);
}

pub fn handle_key(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
    match app.terminal_mode {
        TerminalMode::Insert => handle_insert_mode(app, code, modifiers),
        TerminalMode::Control => handle_control_mode(app, code),
    }
}

pub fn handle_mouse(app: &mut App, mouse: crossterm::event::MouseEvent) {
    match mouse.kind {
        MouseEventKind::ScrollUp => app.scroll(3),
        MouseEventKind::ScrollDown => app.scroll(-3),
        MouseEventKind::Down(_) => {
            if let Some((row, col)) = cell_at(app, mouse.column, mouse.row) {
                app.selection.start(row, col);
            }
        }
        MouseEventKind::Drag(_) | MouseEventKind::Moved if app.selection.active => {
            let (row, col) = clamp_cell(app, mouse.column, mouse.row);
            app.selection.update(row, col);
        }
        MouseEventKind::Up(_) if app.selection.active => app.finish_selection(),
        _ => {}
    }
}

fn cell_at(app: &App, col: u16, row: u16) -> Option<(usize, usize)> {
    let rect = app.view.output_rect;
    if col < rect.x || col >= rect.x + rect.width || row < rect.y || row >= rect.y + rect.height {
        return None;
    }
    Some(((row - rect.y) as usize, (col - rect.x) as usize))
}

fn clamp_cell(app: &App, col: u16, row: u16) -> (usize, usize) {
    let rect = app.view.output_rect;
    let col = col.clamp(rect.x, rect.x + rect.width.saturating_sub(1));
    let row = row.clamp(rect.y, rect.y + rect.height.saturating_sub(1));
    ((row - rect.y) as usize, (col - rect.x) as usize)
}

fn help_spans_for_mode(mode: &TerminalMode, keyboard_enhanced: bool) -> Vec<Span<'static>> {
    match mode {
        TerminalMode::Insert => {
            let mut spans = vec![
                Span::styled(
                    " INSERT ",
                    Style::default().fg(Color::Black).bg(Color::Green).bold(),
                ),
                sep_span(),
            ];
            let escape_hint: &'static str = if keyboard_enhanced {
                "Ctrl+Esc"
            } else {
                "Ctrl+Space"
            };
            spans.extend(help_spans(&[(escape_hint, "control mode")]));
            spans
        }
        TerminalMode::Control => {
            let mut spans = vec![
                Span::styled(
                    " CONTROL ",
                    Style::default().fg(Color::Black).bg(Color::Cyan).bold(),
                ),
                sep_span(),
            ];
            spans.extend(help_spans(&[
                ("a", "insert"),
                ("↑↓", "scroll"),
                ("esc", "bottom"),
                ("f", "flush"),
                ("c", "copy"),
                ("+/-", "baud"),
                ("p", "pause"),
                ("s", "settings"),
                ("del", "port select"),
                ("q", "quit"),
            ]));
            spans
        }
    }
}

fn handle_insert_mode(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
    let is_escape =
        (app.keyboard_enhanced && code == KeyCode::Esc && modifiers == KeyModifiers::CONTROL)
            || (code == KeyCode::Char(' ') && modifiers == KeyModifiers::CONTROL);
    if is_escape {
        app.terminal_mode = TerminalMode::Control;
        return;
    }

    if code == KeyCode::Esc {
        let had_selection = app.selection.anchor.is_some();
        app.selection.clear();
        if app.view.scroll_offset > 0 {
            app.scroll_to_bottom();
            return;
        }
        if had_selection {
            return;
        }
    }

    if app.input_mode == InputMode::Line && !modifiers.contains(KeyModifiers::CONTROL) {
        match code {
            KeyCode::Char(c) => app.line.buffer.push(c),
            KeyCode::Backspace => {
                app.line.buffer.pop();
            }
            KeyCode::Enter => app.send_line(),
            KeyCode::Up => app.history_prev(),
            KeyCode::Down => app.history_next(),
            _ => {}
        }
        return;
    }

    let key_event = crossterm::event::Event::Key(crossterm::event::KeyEvent::new(code, modifiers));
    if let Ok(t_event) = terminput_crossterm::to_terminput(key_event) {
        let mut buf = [0u8; 16];
        if let Ok(n) = t_event.encode(&mut buf, terminput::Encoding::Xterm) {
            let mut bytes = buf[..n].to_vec();
            if app.local_echo {
                app.echo_local(&bytes);
            }
            if bytes == [0x0d] {
                use crate::serial::NewlineEncoding;
                match app.outgoing_newline {
                    NewlineEncoding::CR => {}
                    NewlineEncoding::LF => bytes[0] = 0x0a,
                    NewlineEncoding::CRLF => bytes.push(0x0a),
                }
            }
            app.send_bytes(bytes);
        }
    }
}

fn handle_control_mode(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('a') | KeyCode::Char('i') => {
            app.scroll_to_bottom();
            app.terminal_mode = TerminalMode::Insert;
        }
        KeyCode::Up | KeyCode::Char('k') => app.scroll(3),
        KeyCode::Down | KeyCode::Char('j') => app.scroll(-3),
        KeyCode::Esc => {
            app.selection.clear();
            app.scroll_to_bottom();
        }
        KeyCode::Backspace | KeyCode::Delete => app.disconnect(),
        KeyCode::Char('f') => app.flush_screen(),
        KeyCode::Char('c') => app.copy_to_clipboard(),
        KeyCode::Char('+') => app.change_baud(1),
        KeyCode::Char('-') => app.change_baud(-1),
        KeyCode::Char('p') => app.toggle_pause(),
        KeyCode::Char('s') => app.settings.show = true,
        KeyCode::Char('q') => {
            app.disconnect();
            app.exit = true;
        }
        _ => {}
    }
}

fn is_selected(
    selection: Option<((usize, usize), (usize, usize))>,
    row: usize,
    col: usize,
) -> bool {
    let Some((start, end)) = selection else {
        return false;
    };
    if row < start.0 || row > end.0 {
        return false;
    }
    let from = if row == start.0 { start.1 } else { 0 };
    let to = if row == end.0 { end.1 } else { usize::MAX };
    col >= from && col <= to
}

fn styled_line(
    line: &str,
    row: usize,
    selection: Option<((usize, usize), (usize, usize))>,
) -> ratatui::text::Line<'static> {
    let Some((start, end)) = selection else {
        return ratatui::text::Line::from(line.to_string());
    };
    if row < start.0 || row > end.0 {
        return ratatui::text::Line::from(line.to_string());
    }

    let chars: Vec<char> = line.chars().collect();
    let from = (if row == start.0 { start.1 } else { 0 }).min(chars.len());
    let to = (if row == end.0 { end.1 + 1 } else { chars.len() })
        .max(from)
        .min(chars.len());

    let before: String = chars[..from].iter().collect();
    let selected: String = chars[from..to].iter().collect();
    let after: String = chars[to..].iter().collect();

    let mut spans = Vec::new();
    if !before.is_empty() {
        spans.push(Span::raw(before));
    }
    spans.push(Span::styled(
        selected,
        Style::default().add_modifier(Modifier::REVERSED),
    ));
    if !after.is_empty() {
        spans.push(Span::raw(after));
    }
    ratatui::text::Line::from(spans)
}

fn vt100_color(color: vt100::Color) -> Color {
    match color {
        vt100::Color::Default => Color::Reset,
        vt100::Color::Idx(i) => Color::Indexed(i),
        vt100::Color::Rgb(r, g, b) => Color::Rgb(r, g, b),
    }
}
