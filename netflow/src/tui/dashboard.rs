use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::flow_table::{FlowEntry, FlowState};

pub fn render_dashboard(f: &mut Frame, area: Rect, flows: &[FlowEntry]) {
    let total = flows.len();
    let tcp = flows.iter().filter(|f| f.key.protocol == 6).count();
    let udp = flows.iter().filter(|f| f.key.protocol == 17).count();
    let active = flows.iter().filter(|f| f.state == FlowState::Active).count();

    let text = format!(
        "Total: {} | TCP: {} | UDP: {} | Active: {}",
        total, tcp, udp, active
    );

    let para = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Dashboard"));

    f.render_widget(para, area);
}
