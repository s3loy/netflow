use std::sync::Arc;
use async_trait::async_trait;
use crate::flow_table::FlowTable;

#[cfg(target_os = "linux")]
pub mod ebpf;
pub mod pcap;

/// Platform-agnostic flow collector trait.
///
/// Implementations may use eBPF (Linux, high performance) or libpcap
/// (cross-platform, suitable for development on macOS).
#[async_trait]
pub trait Collector: Send + Sync {
    /// Start collecting and feed parsed flows into `flow_table`.
    ///
    /// This method should run indefinitely until an unrecoverable error
    /// occurs or the task is cancelled. It is spawned onto its own
    /// Tokio task by the caller.
    async fn run(&self, flow_table: Arc<FlowTable>) -> anyhow::Result<()>;
}
