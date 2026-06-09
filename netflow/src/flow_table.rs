use std::time::{Duration, Instant};

use dashmap::DashMap;
pub use netflow_common::FlowStats;
use netflow_common::{FlowEvent, FlowEventType, FlowKey};
use tracing::debug;

#[derive(Debug, Clone)]
pub struct FlowEntry {
    pub key: FlowKey,
    pub stats: FlowStats,
    pub state: FlowState,
    pub created_at: Instant,
    pub last_seen: Instant,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FlowState {
    Active,
    Closed,
}

pub struct FlowTable {
    flows: DashMap<FlowKey, FlowEntry>,
}

impl Default for FlowTable {
    fn default() -> Self {
        Self {
            flows: DashMap::new(),
        }
    }
}

impl FlowTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn handle_event(&self, event: FlowEvent) {
        match event.ty {
            FlowEventType::New => {
                let entry = FlowEntry {
                    key: event.key,
                    stats: event.stats,
                    state: FlowState::Active,
                    created_at: Instant::now(),
                    last_seen: Instant::now(),
                };
                self.flows.insert(event.key, entry);
                debug!("new flow: {:?}", event.key);
            }
            FlowEventType::Close | FlowEventType::Timeout => {
                if let Some(mut entry) = self.flows.get_mut(&event.key) {
                    entry.stats = event.stats;
                    entry.state = FlowState::Closed;
                    entry.last_seen = Instant::now();
                    debug!("closed flow: {:?}", event.key);
                }
            }
        }
    }

    pub fn active_flows(&self) -> Vec<FlowEntry> {
        self.flows
            .iter()
            .filter(|e| e.state == FlowState::Active)
            .map(|e| e.clone())
            .collect()
    }

    pub fn all_flows(&self) -> Vec<FlowEntry> {
        self.flows.iter().map(|e| e.clone()).collect()
    }

    pub fn get_flow(&self, key: &FlowKey) -> Option<FlowEntry> {
        self.flows.get(key).map(|e| e.clone())
    }

    pub fn gc_closed(&self, retention: Duration) {
        let now = Instant::now();
        let to_remove: Vec<FlowKey> = self
            .flows
            .iter()
            .filter(|e| {
                e.state == FlowState::Closed && now.duration_since(e.last_seen) >= retention
            })
            .map(|e| e.key)
            .collect();

        for key in to_remove {
            self.flows.remove(&key);
        }
    }

    /// Incrementally update flow stats from a captured packet.
    ///
    /// If the flow already exists, accumulates `bytes` and increments the
    /// packet counter in the appropriate direction (`sent` or `recv`).
    /// If the flow does not exist, creates a new `Active` entry with the
    /// initial counters set from this packet.
    pub fn upsert_packet(&self, key: FlowKey, bytes: u64, is_sent: bool) {
        use dashmap::mapref::entry::Entry;
        match self.flows.entry(key) {
            Entry::Occupied(mut e) => {
                let entry = e.get_mut();
                if is_sent {
                    entry.stats.packets_sent += 1;
                    entry.stats.bytes_sent += bytes;
                } else {
                    entry.stats.packets_recv += 1;
                    entry.stats.bytes_recv += bytes;
                }
                entry.last_seen = Instant::now();
            }
            Entry::Vacant(e) => {
                let (packets_sent, packets_recv, bytes_sent, bytes_recv) = if is_sent {
                    (1, 0, bytes, 0)
                } else {
                    (0, 1, 0, bytes)
                };
                let entry = FlowEntry {
                    key,
                    stats: FlowStats {
                        packets_sent,
                        packets_recv,
                        bytes_sent,
                        bytes_recv,
                        ts_start_ns: 0,
                        ts_last_ns: 0,
                    },
                    state: FlowState::Active,
                    created_at: Instant::now(),
                    last_seen: Instant::now(),
                };
                e.insert(entry);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> FlowKey {
        FlowKey {
            src_ip: 0x0a000001,
            dst_ip: 0x08080808,
            src_port: 54321,
            dst_port: 443,
            protocol: 6,
        }
    }

    fn test_stats() -> FlowStats {
        FlowStats {
            packets_sent: 10,
            packets_recv: 20,
            bytes_sent: 1000,
            bytes_recv: 2000,
            ts_start_ns: 0,
            ts_last_ns: 0,
        }
    }

    #[test]
    fn test_new_flow() {
        let table = FlowTable::new();
        let key = test_key();
        table.handle_event(FlowEvent {
            ty: FlowEventType::New,
            key,
            stats: test_stats(),
        });
        assert_eq!(table.active_flows().len(), 1);
    }

    #[test]
    fn test_close_flow() {
        let table = FlowTable::new();
        let key = test_key();
        table.handle_event(FlowEvent {
            ty: FlowEventType::New,
            key,
            stats: test_stats(),
        });
        table.handle_event(FlowEvent {
            ty: FlowEventType::Close,
            key,
            stats: FlowStats {
                bytes_sent: 5000,
                ..test_stats()
            },
        });
        let flow = table.get_flow(&key).unwrap();
        assert_eq!(flow.state, FlowState::Closed);
        assert_eq!(flow.stats.bytes_sent, 5000);
    }

    #[test]
    fn test_gc_closed_flows() {
        let table = FlowTable::new();
        let key = test_key();
        table.handle_event(FlowEvent {
            ty: FlowEventType::New,
            key,
            stats: test_stats(),
        });
        table.handle_event(FlowEvent {
            ty: FlowEventType::Close,
            key,
            stats: test_stats(),
        });
        table.gc_closed(Duration::from_secs(0));
        assert!(table.get_flow(&key).is_none());
    }

    #[test]
    fn test_upsert_packet_creates_new_flow() {
        let table = FlowTable::new();
        let key = test_key();
        table.upsert_packet(key, 100, true);
        let flow = table.get_flow(&key).unwrap();
        assert_eq!(flow.state, FlowState::Active);
        assert_eq!(flow.stats.packets_sent, 1);
        assert_eq!(flow.stats.bytes_sent, 100);
        assert_eq!(flow.stats.packets_recv, 0);
        assert_eq!(flow.stats.bytes_recv, 0);
    }

    #[test]
    fn test_upsert_packet_updates_existing_flow() {
        let table = FlowTable::new();
        let key = test_key();
        table.upsert_packet(key, 100, true);
        table.upsert_packet(key, 200, false);
        let flow = table.get_flow(&key).unwrap();
        assert_eq!(flow.stats.packets_sent, 1);
        assert_eq!(flow.stats.bytes_sent, 100);
        assert_eq!(flow.stats.packets_recv, 1);
        assert_eq!(flow.stats.bytes_recv, 200);
    }

    #[test]
    fn test_upsert_packet_creates_new_recv_flow() {
        let table = FlowTable::new();
        let key = test_key();
        table.upsert_packet(key, 300, false);
        let flow = table.get_flow(&key).unwrap();
        assert_eq!(flow.state, FlowState::Active);
        assert_eq!(flow.stats.packets_sent, 0);
        assert_eq!(flow.stats.bytes_sent, 0);
        assert_eq!(flow.stats.packets_recv, 1);
        assert_eq!(flow.stats.bytes_recv, 300);
    }
}
