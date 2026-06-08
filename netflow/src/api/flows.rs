use std::sync::Arc;
use axum::{
    extract::{Path, State},
    Json,
};
use serde::Serialize;
use crate::flow_table::{FlowEntry, FlowState, FlowTable};

#[derive(Serialize)]
pub struct FlowResponse {
    pub src_ip: String,
    pub dst_ip: String,
    pub src_port: u16,
    pub dst_port: u16,
    pub protocol: String,
    pub packets_sent: u64,
    pub packets_recv: u64,
    pub bytes_sent: u64,
    pub bytes_recv: u64,
    pub state: String,
}

impl From<FlowEntry> for FlowResponse {
    fn from(entry: FlowEntry) -> Self {
        let src_ip = entry.key.src_ip.to_be_bytes();
        let dst_ip = entry.key.dst_ip.to_be_bytes();
        Self {
            src_ip: format!("{}.{}.{}.{}", src_ip[0], src_ip[1], src_ip[2], src_ip[3]),
            dst_ip: format!("{}.{}.{}.{}", dst_ip[0], dst_ip[1], dst_ip[2], dst_ip[3]),
            src_port: entry.key.src_port,
            dst_port: u16::from_be(entry.key.dst_port),
            protocol: if entry.key.protocol == 6 { "tcp".into() } else { "udp".into() },
            packets_sent: entry.stats.packets_sent,
            packets_recv: entry.stats.packets_recv,
            bytes_sent: entry.stats.bytes_sent,
            bytes_recv: entry.stats.bytes_recv,
            state: if entry.state == FlowState::Active { "active".into() } else { "closed".into() },
        }
    }
}

pub async fn list_flows(
    State(table): State<Arc<FlowTable>>,
) -> Json<Vec<FlowResponse>> {
    let flows = table.active_flows();
    Json(flows.into_iter().map(Into::into).collect())
}

pub async fn get_flow(
    State(table): State<Arc<FlowTable>>,
    Path(id): Path<String>,
) -> Result<Json<FlowResponse>, axum::http::StatusCode> {
    let decoded = hex::decode(&id).map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
    if decoded.len() != 13 {
        return Err(axum::http::StatusCode::BAD_REQUEST);
    }

    let key = netflow_common::FlowKey {
        src_ip: u32::from_be_bytes([decoded[0], decoded[1], decoded[2], decoded[3]]),
        dst_ip: u32::from_be_bytes([decoded[4], decoded[5], decoded[6], decoded[7]]),
        src_port: u16::from_be_bytes([decoded[8], decoded[9]]),
        dst_port: u16::from_be_bytes([decoded[10], decoded[11]]),
        protocol: decoded[12],
    };

    match table.get_flow(&key) {
        Some(entry) => Ok(Json(entry.into())),
        None => Err(axum::http::StatusCode::NOT_FOUND),
    }
}
