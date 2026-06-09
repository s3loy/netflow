use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::flow_table::FlowEntry;

pub fn render_dashboard(f: &mut Frame, area: Rect, flows: &[FlowEntry]) {
    let total = flows.len();
    let tcp = flows.iter().filter(|f| f.key.protocol == 6).count();
    let udp = flows.iter().filter(|f| f.key.protocol == 17).count();
    let other = total.saturating_sub(tcp + udp);
    let active = flows.iter().filter(|f| f.state == crate::flow_table::FlowState::Active).count();
    let closed = total.saturating_sub(active);

    let text = format!(
        "Total: {} | TCP: {} | UDP: {} | Other: {} | Active: {} | Closed: {}",
        total, tcp, udp, other, active, closed
    );

    let para = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title("Dashboard"),
        );

    f.render_widget(para, area);
}
