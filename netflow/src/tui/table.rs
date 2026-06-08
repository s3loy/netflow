use ratatui::{
    layout::Rect,
    prelude::Stylize,
    widgets::{Block, Borders, Row, Table},
    Frame,
};
use crate::flow_table::FlowEntry;

pub fn render_flow_table(f: &mut Frame, area: Rect, flows: &[FlowEntry]) {
    let rows: Vec<Row> = flows
        .iter()
        .map(|flow| {
            let src_ip = flow.key.src_ip.to_be_bytes();
            let dst_ip = flow.key.dst_ip.to_be_bytes();
            Row::new(vec![
                format!("{}", if flow.key.protocol == 6 { "TCP" } else { "UDP" }),
                format!("{}.{}.{}.{}", src_ip[0], src_ip[1], src_ip[2], src_ip[3]),
                format!("{}", flow.key.src_port),
                format!("{}.{}.{}.{}", dst_ip[0], dst_ip[1], dst_ip[2], dst_ip[3]),
                format!("{}", u16::from_be(flow.key.dst_port)),
                format!("{}", flow.stats.bytes_sent),
                format!("{}", flow.stats.bytes_recv),
                format!("{}", if flow.state == crate::flow_table::FlowState::Active { "Active" } else { "Closed" }),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            ratatui::layout::Constraint::Length(6),
            ratatui::layout::Constraint::Length(18),
            ratatui::layout::Constraint::Length(8),
            ratatui::layout::Constraint::Length(18),
            ratatui::layout::Constraint::Length(8),
            ratatui::layout::Constraint::Length(12),
            ratatui::layout::Constraint::Length(12),
            ratatui::layout::Constraint::Length(8),
        ],
    )
    .header(Row::new(vec!["Proto", "Src IP", "Port", "Dst IP", "Port", "↑ Bytes", "↓ Bytes", "State"]).bold())
    .block(Block::default().borders(Borders::ALL).title("Flows"));

    f.render_widget(table, area);
}
