use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
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
            protocol: match entry.key.protocol {
                6 => "tcp".into(),
                17 => "udp".into(),
                _ => "other".into(),
            },
            packets_sent: entry.stats.packets_sent,
            packets_recv: entry.stats.packets_recv,
            bytes_sent: entry.stats.bytes_sent,
            bytes_recv: entry.stats.bytes_recv,
            state: if entry.state == FlowState::Active {
                "active".into()
            } else {
                "closed".into()
            },
        }
    }
}

pub async fn list_flows(State(table): State<Arc<FlowTable>>) -> Json<Vec<FlowResponse>> {
    let flows = table.active_flows();
    Json(flows.into_iter().map(Into::into).collect())
}

pub async fn get_flow(
    State(table): State<Arc<FlowTable>>,
    Path(id): Path<String>,
) -> Result<Json<FlowResponse>, axum::http::StatusCode> {
    let key = decode_flow_id(&id).ok_or(axum::http::StatusCode::BAD_REQUEST)?;
    match table.get_flow(&key) {
        Some(entry) => Ok(Json(entry.into())),
        None => Err(axum::http::StatusCode::NOT_FOUND),
    }
}

fn decode_flow_id(id: &str) -> Option<netflow_common::FlowKey> {
    let decoded = hex::decode(id).ok()?;
    if decoded.len() != 13 {
        return None;
    }
    Some(netflow_common::FlowKey {
        src_ip: u32::from_be_bytes([decoded[0], decoded[1], decoded[2], decoded[3]]),
        dst_ip: u32::from_be_bytes([decoded[4], decoded[5], decoded[6], decoded[7]]),
        src_port: u16::from_be_bytes([decoded[8], decoded[9]]),
        dst_port: u16::from_be_bytes([decoded[10], decoded[11]]),
        protocol: decoded[12],
    })
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::*;
    use crate::flow_table::{FlowEntry, FlowState, FlowStats};

    fn sample_entry(protocol: u8, state: FlowState) -> FlowEntry {
        FlowEntry {
            key: netflow_common::FlowKey {
                src_ip: 0x0A000001,
                dst_ip: 0x08080808,
                src_port: 54321,
                dst_port: u16::to_be(443),
                protocol,
            },
            stats: FlowStats {
                packets_sent: 10,
                packets_recv: 20,
                bytes_sent: 1000,
                bytes_recv: 2000,
                ts_start_ns: 0,
                ts_last_ns: 0,
            },
            state,
            created_at: Instant::now(),
            last_seen: Instant::now(),
        }
    }

    #[test]
    fn test_flow_response_from_tcp_active() {
        let entry = sample_entry(6, FlowState::Active);
        let resp: FlowResponse = entry.into();
        assert_eq!(resp.src_ip, "10.0.0.1");
        assert_eq!(resp.dst_ip, "8.8.8.8");
        assert_eq!(resp.src_port, 54321);
        assert_eq!(resp.dst_port, 443);
        assert_eq!(resp.protocol, "tcp");
        assert_eq!(resp.packets_sent, 10);
        assert_eq!(resp.packets_recv, 20);
        assert_eq!(resp.bytes_sent, 1000);
        assert_eq!(resp.bytes_recv, 2000);
        assert_eq!(resp.state, "active");
    }

    #[test]
    fn test_flow_response_from_udp_closed() {
        let entry = sample_entry(17, FlowState::Closed);
        let resp: FlowResponse = entry.into();
        assert_eq!(resp.protocol, "udp");
        assert_eq!(resp.state, "closed");
    }

    #[test]
    fn test_flow_response_from_other_protocol() {
        let entry = sample_entry(1, FlowState::Active);
        let resp: FlowResponse = entry.into();
        assert_eq!(resp.protocol, "other");
    }

    #[test]
    fn test_decode_flow_id_valid() {
        // 10.0.0.1, 8.8.8.8, port 54321, port 443, protocol 6
        let hex_id = "0a00000108080808d43101bb06";
        let key = decode_flow_id(hex_id).unwrap();
        assert_eq!(key.src_ip, 0x0A000001);
        assert_eq!(key.dst_ip, 0x08080808);
        assert_eq!(key.src_port, 0xD431); // 54321 in BE
        assert_eq!(key.dst_port, 0x01BB); // 443 in BE
        assert_eq!(key.protocol, 6);
    }

    #[test]
    fn test_decode_flow_id_invalid_hex() {
        assert!(decode_flow_id("not-hex").is_none());
    }

    #[test]
    fn test_decode_flow_id_wrong_length() {
        // 12 bytes instead of 13
        assert!(decode_flow_id("0a00000108080808d43101bb").is_none());
        // 14 bytes instead of 13
        assert!(decode_flow_id("0a00000108080808d43101bb0600").is_none());
    }
}
