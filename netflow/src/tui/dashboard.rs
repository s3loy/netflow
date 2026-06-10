use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};

use crate::flow_table::{FlowEntry, FlowState};

#[derive(Debug, PartialEq)]
pub struct DashboardStats {
    pub total: usize,
    pub tcp: usize,
    pub udp: usize,
    pub other: usize,
    pub active: usize,
    pub closed: usize,
}

pub fn compute_stats(flows: &[FlowEntry]) -> DashboardStats {
    let total = flows.len();
    let tcp = flows.iter().filter(|f| f.key.protocol == 6).count();
    let udp = flows.iter().filter(|f| f.key.protocol == 17).count();
    let other = total.saturating_sub(tcp + udp);
    let active = flows
        .iter()
        .filter(|f| f.state == FlowState::Active)
        .count();
    let closed = total.saturating_sub(active);
    DashboardStats {
        total,
        tcp,
        udp,
        other,
        active,
        closed,
    }
}

pub fn render_dashboard(f: &mut Frame, area: Rect, flows: &[FlowEntry]) {
    let stats = compute_stats(flows);
    let text = format!(
        "Total: {} | TCP: {} | UDP: {} | Other: {} | Active: {} | Closed: {}",
        stats.total, stats.tcp, stats.udp, stats.other, stats.active, stats.closed
    );

    let para = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title("Dashboard"),
    );
    f.render_widget(para, area);
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::*;
    use crate::flow_table::{FlowEntry, FlowStats};

    fn flow(protocol: u8, state: FlowState) -> FlowEntry {
        FlowEntry {
            key: netflow_common::FlowKey {
                src_ip: 0x0A000001,
                dst_ip: 0x08080808,
                src_port: 12345,
                dst_port: 80,
                protocol,
                _pad: [0; 3],
            },
            stats: FlowStats::default(),
            state,
            created_at: Instant::now(),
            last_seen: Instant::now(),
        }
    }

    #[test]
    fn test_compute_stats_empty() {
        let stats = compute_stats(&[]);
        assert_eq!(
            stats,
            DashboardStats {
                total: 0,
                tcp: 0,
                udp: 0,
                other: 0,
                active: 0,
                closed: 0
            }
        );
    }

    #[test]
    fn test_compute_stats_mixed() {
        let flows = vec![
            flow(6, FlowState::Active),
            flow(6, FlowState::Closed),
            flow(17, FlowState::Active),
            flow(1, FlowState::Active),
        ];
        let stats = compute_stats(&flows);
        assert_eq!(stats.total, 4);
        assert_eq!(stats.tcp, 2);
        assert_eq!(stats.udp, 1);
        assert_eq!(stats.other, 1);
        assert_eq!(stats.active, 3);
        assert_eq!(stats.closed, 1);
    }

    #[test]
    fn test_compute_stats_all_closed() {
        let flows = vec![flow(6, FlowState::Closed), flow(17, FlowState::Closed)];
        let stats = compute_stats(&flows);
        assert_eq!(stats.active, 0);
        assert_eq!(stats.closed, 2);
    }
}
