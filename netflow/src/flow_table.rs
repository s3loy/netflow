use std::time::{Duration, Instant};
use dashmap::DashMap;
use netflow_common::{FlowEvent, FlowEventType, FlowKey, FlowStats};
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

impl FlowTable {
    pub fn new() -> Self {
        Self {
            flows: DashMap::new(),
        }
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
                e.state == FlowState::Closed && now.duration_since(e.last_seen) > retention
            })
            .map(|e| e.key)
            .collect();

        for key in to_remove {
            self.flows.remove(&key);
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
}
