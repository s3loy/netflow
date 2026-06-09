use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Row, Table, TableState},
};

use super::{app::AppState, format};
use crate::flow_table::{FlowEntry, FlowState};

/// Render the scrollable flow list.
pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
    let header = Row::new(vec![
        "Proto", "Src IP", "SPort", "Dst IP", "DPort", "↑Pkts", "↓Pkts", "↑Bytes", "↓Bytes",
        "State",
    ])
    .style(Style::default().add_modifier(Modifier::BOLD));

    let visible = area.height.saturating_sub(3) as usize;

    // Empty state: show a friendly hint instead of a blank table
    if state.flows.is_empty() {
        let title = "Flows (0 total)";
        let empty_text = "No flows captured yet. Waiting for traffic...";
        let para = ratatui::widgets::Paragraph::new(empty_text)
            .alignment(ratatui::layout::Alignment::Center)
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::ALL).title(title));
        f.render_widget(para, area);
        return;
    }

    let rows: Vec<Row> = state
        .flows
        .iter()
        .skip(state.offset)
        .take(visible)
        .enumerate()
        .map(|(idx, flow)| {
            let actual_idx = state.offset + idx;
            let style = if actual_idx == state.selected {
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else if flow.state == FlowState::Closed {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default()
            };
            row_from_flow(flow).style(style)
        })
        .collect();

    let widths = [
        Constraint::Length(6),
        Constraint::Length(16),
        Constraint::Length(7),
        Constraint::Length(16),
        Constraint::Length(7),
        Constraint::Length(8),
        Constraint::Length(8),
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Length(8),
    ];

    let mut table_state = TableState::default();
    if !state.flows.is_empty() {
        table_state.select(Some(state.selected));
    }

    let title = format!("Flows ({} total)", state.flows.len());
    let rows_len = rows.len();
    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(title))
        .row_highlight_style(Style::default().bg(Color::Blue).fg(Color::White));

    f.render_stateful_widget(table, area, &mut table_state);

    // Scrollbar hint
    if state.flows.len() > visible {
        let hint = format!(
            "{}-{}/{}",
            state.offset + 1,
            (state.offset + rows_len).min(state.flows.len()),
            state.flows.len()
        );
        let hint_width = hint.len() as u16 + 2;
        let hint_area = Rect {
            x: area.x + area.width.saturating_sub(hint_width + 2),
            y: area.y,
            width: hint_width.min(area.width.saturating_sub(2)),
            height: 1,
        };
        let hint_widget =
            ratatui::widgets::Paragraph::new(hint).style(Style::default().fg(Color::Gray));
        f.render_widget(hint_widget, hint_area);
    }
}

fn row_from_flow(flow: &FlowEntry) -> Row<'_> {
    let state_str = if flow.state == FlowState::Active {
        "Active"
    } else {
        "Closed"
    };

    Row::new(vec![
        format::proto_str(flow.key.protocol).to_string(),
        format::ip_str(flow.key.src_ip),
        format!("{}", flow.key.src_port),
        format::ip_str(flow.key.dst_ip),
        format!("{}", u16::from_be(flow.key.dst_port)),
        format!("{}", flow.stats.packets_sent),
        format!("{}", flow.stats.packets_recv),
        format::format_bytes_compact(flow.stats.bytes_sent),
        format::format_bytes_compact(flow.stats.bytes_recv),
        state_str.to_string(),
    ])
}

/// Compute how many rows fit in the given area (accounting for borders).
pub fn visible_row_count_from_height(height: u16) -> usize {
    height.saturating_sub(3) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visible_row_count_from_height() {
        assert_eq!(visible_row_count_from_height(0), 0);
        assert_eq!(visible_row_count_from_height(3), 0);
        assert_eq!(visible_row_count_from_height(10), 7);
        assert_eq!(visible_row_count_from_height(23), 20);
    }
}
