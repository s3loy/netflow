use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use crate::flow_table::FlowEntry;
use super::app::AppState;
use super::format;

/// Render the detail modal. Returns the number of content lines for scroll limit tracking.
pub fn render(f: &mut Frame, area: Rect, state: &AppState) -> usize {
    let Some(flow) = state.selected_flow() else {
        return 0;
    };

    // Modal size: 60% width, 70% height, max 60x22
    let modal_width = (area.width as f32 * 0.6).clamp(40.0, 60.0) as u16;
    let modal_height = (area.height as f32 * 0.7).clamp(12.0, 22.0) as u16;

    let modal_area = centered_rect(modal_width, modal_height, area);

    // Clear background behind modal
    f.render_widget(Clear, modal_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" Flow Details ")
        .title_alignment(Alignment::Center);

    let inner = block.inner(modal_area);
    f.render_widget(block, modal_area);

    let lines = flow_detail_lines(flow);
    let total_lines = lines.len();
    let scroll = state.modal_scroll.min(total_lines.saturating_sub(1));

    let visible_lines: Vec<Line> = lines
        .into_iter()
        .skip(scroll)
        .take(inner.height as usize)
        .collect();

    let para = Paragraph::new(visible_lines)
        .wrap(Wrap { trim: true });

    f.render_widget(para, inner);

    // Scroll hint
    if total_lines > inner.height as usize {
        let hint = format!(" {}/{} ", scroll + 1, total_lines);
        let hint_area = Rect {
            x: modal_area.x + modal_area.width.saturating_sub(hint.len() as u16 + 2),
            y: modal_area.y + modal_area.height - 1,
            width: hint.len() as u16,
            height: 1,
        };
        let hint_widget = Paragraph::new(hint)
            .style(Style::default().fg(Color::Gray));
        f.render_widget(hint_widget, hint_area);
    }

    total_lines
}

fn flow_detail_lines(flow: &FlowEntry) -> Vec<Line<'static>> {
    let proto = format::proto_str(flow.key.protocol);
    let state = format::state_str(flow);
    let src_ip = format::ip_str(flow.key.src_ip);
    let dst_ip = format::ip_str(flow.key.dst_ip);
    let src_port = flow.key.src_port;
    let dst_port = u16::from_be(flow.key.dst_port);

    let duration_secs = flow
        .last_seen
        .saturating_duration_since(flow.created_at)
        .as_secs();
    let duration = format::format_duration(duration_secs);

    let total_pkts = flow.stats.packets_sent + flow.stats.packets_recv;
    let total_bytes = flow.stats.bytes_sent + flow.stats.bytes_recv;

    let src_port_str = format_port(src_port);
    let dst_port_str = format_port(dst_port);

    let mut lines = vec![
        detail_line("Protocol", proto),
        detail_line("State", state),
        Line::from(""),
        detail_line("Source", &format!("{} {}", src_ip, src_port_str)),
        detail_line("Destination", &format!("{} {}", dst_ip, dst_port_str)),
        Line::from(""),
        detail_line("Duration", &duration),
        detail_line("Age", &format_duration_since(flow.created_at)),
        Line::from(""),
        detail_line("Total Packets", &format!("{}", total_pkts)),
        detail_line("  └─ Sent", &format!("{} (avg {})",
            flow.stats.packets_sent,
            format::avg_pkt_size(flow.stats.bytes_sent, flow.stats.packets_sent))),
        detail_line("  └─ Recv", &format!("{} (avg {})",
            flow.stats.packets_recv,
            format::avg_pkt_size(flow.stats.bytes_recv, flow.stats.packets_recv))),
        detail_line("Total Bytes", &format::format_bytes(total_bytes)),
        detail_line("  └─ Sent", &format::format_bytes(flow.stats.bytes_sent)),
        detail_line("  └─ Recv", &format::format_bytes(flow.stats.bytes_recv)),
    ];

    // Add rate info only if flow has been alive long enough
    if duration_secs > 0 {
        lines.push(Line::from(""));
        lines.push(detail_line("Send Rate", &format!("{}  {}",
            format::format_pps(flow.stats.packets_sent, duration_secs),
            format::format_bps(flow.stats.bytes_sent, duration_secs))));
        lines.push(detail_line("Recv Rate", &format!("{}  {}",
            format::format_pps(flow.stats.packets_recv, duration_secs),
            format::format_bps(flow.stats.bytes_recv, duration_secs))));
    }

    // Traffic ratio bar
    if total_bytes > 0 {
        lines.push(Line::from(""));
        let sent_pct = (flow.stats.bytes_sent as f64 / total_bytes as f64 * 100.0) as u8;
        lines.push(detail_line("Traffic Split", &format!("{}% sent / {}% recv", sent_pct, 100 - sent_pct)));
        lines.push(Line::from(format!("  [{}]", traffic_bar(sent_pct))));
    }

    lines
}

fn format_port(port: u16) -> String {
    match format::port_name(port) {
        Some(name) => format!(":{} ({})", port, name),
        None => format!(":{}", port),
    }
}

fn format_duration_since(instant: std::time::Instant) -> String {
    let secs = instant.elapsed().as_secs();
    format::format_duration(secs)
}

fn traffic_bar(sent_pct: u8) -> String {
    let width = 20usize;
    let sent_blocks = (sent_pct as usize * width / 100).min(width);
    let recv_blocks = width - sent_blocks;
    format!("{}{}", "█".repeat(sent_blocks), "░".repeat(recv_blocks))
}

fn detail_line(label: &str, value: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("{:<14}", label),
            Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan),
        ),
        Span::raw(value.to_string()),
    ])
}

fn centered_rect(width: u16, height: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length((r.height.saturating_sub(height)) / 2),
            Constraint::Length(height),
            Constraint::Length(r.height.saturating_sub(height).div_ceil(2)),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length((r.width.saturating_sub(width)) / 2),
            Constraint::Length(width),
            Constraint::Length(r.width.saturating_sub(width).div_ceil(2)),
        ])
        .split(popup_layout[1])[1]
}
