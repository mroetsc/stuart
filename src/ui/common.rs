use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::state::{App, Screen};

pub fn key_span(label: &'static str) -> Span<'static> {
    Span::styled(format!(" {} ", label), Style::default().reversed().bold())
}

pub fn action_span(label: &'static str) -> Span<'static> {
    Span::styled(label, Style::default().fg(Color::DarkGray))
}

pub fn sep_span() -> Span<'static> {
    Span::styled(" │ ", Style::default().fg(Color::DarkGray))
}

pub fn help_entry(key: &'static str, action: &'static str) -> [Span<'static>; 4] {
    [
        key_span(key),
        Span::raw(" "),
        action_span(action),
        sep_span(),
    ]
}

pub fn help_spans(entries: &[(&'static str, &'static str)]) -> Vec<Span<'static>> {
    let mut spans: Vec<Span> = entries.iter().flat_map(|(k, a)| help_entry(k, a)).collect();
    spans.pop();
    spans
}

pub fn wrap_spans_to_lines(spans: Vec<Span<'static>>, width: u16) -> Vec<Line<'static>> {
    let inner_width = width.saturating_sub(2) as usize;
    let mut lines: Vec<Line> = Vec::new();
    let mut current: Vec<Span> = Vec::new();
    let mut current_width: usize = 0;

    for span in spans {
        let span_width = span.width();
        if current_width + span_width > inner_width && !current.is_empty() {
            lines.push(Line::from(std::mem::take(&mut current)));
            current_width = 0;
        }
        current_width += span_width;
        current.push(span);
    }
    if !current.is_empty() {
        lines.push(Line::from(current));
    }
    if lines.is_empty() {
        lines.push(Line::default());
    }
    lines
}

pub fn help_bar_height(spans: Vec<Span<'static>>, width: u16) -> (u16, Vec<Line<'static>>) {
    let lines = wrap_spans_to_lines(spans, width);
    let height = lines.len() as u16 + 2;
    (height, lines)
}

fn stuart_span() -> Span<'static> {
    Span::styled(
        " stuart ",
        Style::default()
            .bold()
            .bg(Color::Rgb(211, 69, 21))
            .fg(Color::Gray),
    )
}

pub fn info_bar_left_spans(app: &App) -> Vec<Span<'static>> {
    if app.active_port.is_empty() || app.screen == Screen::PortSelect {
        vec![stuart_span()]
    } else {
        vec![
            stuart_span(),
            sep_span(),
            Span::styled(" on", Style::default().fg(Color::DarkGray)),
            Span::styled(format!(" {} ", app.active_port), Style::default().bold()),
            sep_span(),
            Span::styled(
                format!(" {} ", app.port_config.baud),
                Style::default().bold(),
            ),
            Span::styled("baud rate", Style::default().fg(Color::DarkGray)),
        ]
    }
}

pub fn info_bar_right_spans(app: &App) -> Vec<Span<'static>> {
    let mut spans: Vec<Span<'static>> = Vec::new();

    let line_count: usize = app
        .scrollback
        .iter()
        .flat_map(|l| l.split_inclusive('\n'))
        .flat_map(|l| l.strip_suffix('\n').or(Some(l)))
        .count();
    let max_offset = line_count.saturating_sub(app.viewport_height);
    let at_top = app.scroll_offset >= max_offset && max_offset > 0;

    if at_top {
        spans.push(Span::styled(
            " scrollback TOP ",
            Style::default().fg(Color::DarkGray),
        ));
    } else if app.scroll_offset > 0 {
        spans.push(Span::styled(
            format!(" scrollback +{} ", app.scroll_offset),
            Style::default().fg(Color::DarkGray),
        ));
    }

    if app.connection.is_none() && app.hold && app.screen == Screen::Terminal {
        if !spans.is_empty() {
            spans.push(sep_span());
        }
        spans.push(Span::styled(
            " reconnecting… ",
            Style::default().fg(Color::Yellow).bold(),
        ));
    }

    spans
}

pub fn info_bar_spans(app: &App) -> Vec<Span<'static>> {
    info_bar_left_spans(app)
}

pub fn draw_info_bar(app: &App, frame: &mut Frame, area: Rect) {
    let block = Block::new().borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let left = Paragraph::new(Line::from(info_bar_left_spans(app)));
    frame.render_widget(left, inner);

    let right_spans = info_bar_right_spans(app);
    if !right_spans.is_empty() {
        let right = Paragraph::new(Line::from(right_spans)).alignment(Alignment::Right);
        frame.render_widget(right, inner);
    }
}

pub fn draw_error_popup(app: &App, frame: &mut Frame) {
    let Some(entry) = app.errors.last() else {
        return;
    };

    let label = if entry.count > 1 {
        format!("{} (x{})", entry.message, entry.count)
    } else {
        entry.message.clone()
    };

    let area = frame.area();
    let width = (label.len() as u16 + 4).max(24).min(area.width);
    let popup_area = Rect {
        x: (area.width.saturating_sub(width)) / 2,
        y: 0,
        width,
        height: 3,
    };

    frame.render_widget(Clear, popup_area);
    let paragraph = Paragraph::new(Span::styled(label, Style::default().fg(Color::Red))).block(
        Block::new()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red))
            .title(Span::styled(
                " error ",
                Style::default().fg(Color::Red).bold(),
            )),
    );
    frame.render_widget(paragraph, popup_area);
}
