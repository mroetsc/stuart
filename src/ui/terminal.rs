use crossterm::event::{KeyCode, KeyModifiers, MouseEventKind};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

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
    app.viewport_height = inner.height as usize;

    draw_info_bar(app, frame, info_area);

    let all_lines: Vec<&str> = app
        .scrollback
        .iter()
        .flat_map(|l| l.split_inclusive('\n'))
        .flat_map(|l| l.strip_suffix('\n').or(Some(l)))
        .collect();
    let total = all_lines.len();
    let height = inner.height as usize;
    let max_offset = total.saturating_sub(height);
    let scrolling = app.scroll_offset > 0;

    let block = Block::new().borders(Borders::ALL);
    frame.render_widget(block, output_area);

    if scrolling {
        let end = total.saturating_sub(app.scroll_offset.min(max_offset));
        let start = end.saturating_sub(height);
        let visible: Vec<&str> = all_lines[start..end].to_vec();
        let text = visible.join("\n");
        frame.render_widget(Paragraph::new(text), inner);
    } else {
        let screen = app.parser.screen();
        let buf = frame.buffer_mut();
        for row in 0..inner.height {
            for col in 0..inner.width {
                if let Some(cell) = screen.cell(row, col) {
                    let ch = cell.contents();
                    if ch.is_empty() {
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
                    buf[(inner.x + col, inner.y + row)]
                        .set_symbol(ch)
                        .set_style(style);
                }
            }
        }
        let (crow, ccol) = screen.cursor_position();
        frame.set_cursor_position((
            (inner.x + ccol).min(inner.x + inner.width - 1),
            (inner.y + crow).min(inner.y + inner.height - 1),
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
        _ => {}
    }
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
                ("Esc", "bottom"),
                ("f", "flush"),
                ("c", "copy"),
                ("+/-", "baud"),
                ("s", "settings"),
                ("Del", "port select"),
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
        KeyCode::Esc => app.scroll_to_bottom(),
        KeyCode::Backspace | KeyCode::Delete => app.disconnect(),
        KeyCode::Char('f') => app.flush_screen(),
        KeyCode::Char('c') => app.copy_to_clipboard(),
        KeyCode::Char('+') => app.change_baud(1),
        KeyCode::Char('-') => app.change_baud(-1),
        KeyCode::Char('s') => app.show_settings = true,
        KeyCode::Char('q') => {
            app.disconnect();
            app.exit = true;
        }
        _ => {}
    }
}

fn vt100_color(color: vt100::Color) -> Color {
    match color {
        vt100::Color::Default => Color::Reset,
        vt100::Color::Idx(i) => Color::Indexed(i),
        vt100::Color::Rgb(r, g, b) => Color::Rgb(r, g, b),
    }
}
