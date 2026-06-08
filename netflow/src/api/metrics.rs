use std::sync::Arc;
use axum::{
    extract::State,
    response::IntoResponse,
};
use crate::flow_table::{FlowState, FlowTable};

pub async fn prometheus_metrics(State(table): State<Arc<FlowTable>>) -> impl IntoResponse {
    let flows = table.all_flows();
    let mut tcp_active = 0u64;
    let mut udp_active = 0u64;
    let mut total_bytes_sent = 0u64;
    let mut total_bytes_recv = 0u64;

    for flow in &flows {
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

    let output = format!(
        "# HELP netflow_flows_active_total Number of active flows\n\
         # TYPE netflow_flows_active_total gauge\n\
         netflow_flows_active_total{{protocol=\"tcp\"}} {tcp_active}\n\
         netflow_flows_active_total{{protocol=\"udp\"}} {udp_active}\n\
         # HELP netflow_bytes_total Total bytes\n\
         # TYPE netflow_bytes_total counter\n\
         netflow_bytes_total{{direction=\"sent\"}} {total_bytes_sent}\n\
         netflow_bytes_total{{direction=\"recv\"}} {total_bytes_recv}\n"
    );

    ([("content-type", "text/plain; version=0.0.4")], output)
}
