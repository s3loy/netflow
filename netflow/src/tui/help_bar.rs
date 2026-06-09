use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

/// Render a nano-style bottom help bar with context-aware shortcuts.
pub fn render(f: &mut Frame, area: Rect, modal_open: bool) {
    let spans = if modal_open {
        vec![
            key_span("↑↓/jk", "scroll"),
            sep(),
            key_span("PgUp/PgDown", "page"),
            sep(),
            key_span("Enter/Esc", "close"),
            sep(),
            key_span("q/Ctrl+C", "quit"),
        ]
    } else {
        vec![
            key_span("↑↓/jk", "move"),
            sep(),
            key_span("Enter", "detail"),
            sep(),
            key_span("PgUp/PgDown", "page"),
            sep(),
            key_span("g/G", "top/bottom"),
            sep(),
            key_span("q/Ctrl+C", "quit"),
        ]
    };

    let line = Line::from(spans);
    let para = Paragraph::new(line)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White));
    f.render_widget(para, area);
}

fn key_span(key: &str, desc: &str) -> Span<'static> {
    Span::styled(
        format!(" {}: {} ", key, desc),
        Style::default().bg(Color::DarkGray).fg(Color::White),
    )
}

fn sep() -> Span<'static> {
    Span::styled("│", Style::default().bg(Color::DarkGray).fg(Color::Gray))
}
