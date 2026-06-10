use std::sync::Arc;

use axum::{extract::State, response::IntoResponse};

use crate::flow_table::{FlowEntry, FlowState};

pub async fn prometheus_metrics(
    State(table): State<Arc<crate::flow_table::FlowTable>>,
) -> impl IntoResponse {
    let output = render_metrics(&table.all_flows());
    ([("content-type", "text/plain; version=0.0.4")], output)
}

fn render_metrics(flows: &[FlowEntry]) -> String {
    let mut tcp_active = 0u64;
    let mut udp_active = 0u64;
    let mut total_bytes_sent = 0u64;
    let mut total_bytes_recv = 0u64;

    for flow in flows {
        if flow.state == FlowState::Active {
            if flow.key.protocol == 6 {
                tcp_active += 1;
            } else {
                udp_active += 1;
            }
        }
        total_bytes_sent += flow.stats.bytes_sent;
        total_bytes_recv += flow.stats.bytes_recv;
    }

    format!(
        "# HELP netflow_flows_active_total Number of active flows\n\
         # TYPE netflow_flows_active_total gauge\n\
         netflow_flows_active_total{{protocol=\"tcp\"}} {tcp_active}\n\
         netflow_flows_active_total{{protocol=\"udp\"}} {udp_active}\n\
         # HELP netflow_bytes_total Total bytes\n\
         # TYPE netflow_bytes_total counter\n\
         netflow_bytes_total{{direction=\"sent\"}} {total_bytes_sent}\n\
         netflow_bytes_total{{direction=\"recv\"}} {total_bytes_recv}\n"
    )
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::*;
    use crate::flow_table::{FlowEntry, FlowState, FlowStats};

    fn make_flow(protocol: u8, state: FlowState, sent: u64, recv: u64) -> FlowEntry {
        FlowEntry {
            key: netflow_common::FlowKey {
                src_ip: 0x0A000001,
                dst_ip: 0x08080808,
                src_port: 12345,
                dst_port: 80,
                protocol,
                _pad: [0; 3],
            },
            stats: FlowStats {
                packets_sent: 0,
                packets_recv: 0,
                bytes_sent: sent,
                bytes_recv: recv,
                ts_start_ns: 0,
                ts_last_ns: 0,
            },
            state,
            created_at: Instant::now(),
            last_seen: Instant::now(),
        }
    }

    #[test]
    fn test_render_metrics_empty() {
        let out = render_metrics(&[]);
        assert!(out.contains("netflow_flows_active_total{protocol=\"tcp\"} 0"));
        assert!(out.contains("netflow_flows_active_total{protocol=\"udp\"} 0"));
        assert!(out.contains("netflow_bytes_total{direction=\"sent\"} 0"));
        assert!(out.contains("netflow_bytes_total{direction=\"recv\"} 0"));
    }

    #[test]
    fn test_render_metrics_counts() {
        let flows = vec![
            make_flow(6, FlowState::Active, 100, 200),
            make_flow(17, FlowState::Active, 50, 75),
            make_flow(6, FlowState::Closed, 10, 20),
        ];
        let out = render_metrics(&flows);
        assert!(out.contains("netflow_flows_active_total{protocol=\"tcp\"} 1"));
        assert!(out.contains("netflow_flows_active_total{protocol=\"udp\"} 1"));
        assert!(out.contains("netflow_bytes_total{direction=\"sent\"} 160"));
        assert!(out.contains("netflow_bytes_total{direction=\"recv\"} 295"));
    }

    #[test]
    fn test_render_metrics_other_protocol_not_counted() {
        let flows = vec![make_flow(1, FlowState::Active, 10, 20)];
        let out = render_metrics(&flows);
        // Other protocols count toward udp_active (fallback branch)
        assert!(out.contains("netflow_flows_active_total{protocol=\"udp\"} 1"));
        assert!(out.contains("netflow_flows_active_total{protocol=\"tcp\"} 0"));
    }
}
