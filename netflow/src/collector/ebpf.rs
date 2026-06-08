use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::Mutex;
use tracing::info;

use crate::collector::Collector;
use crate::config::Config;
use crate::ebpf_loader::EbpfLoader;
use crate::flow_table::FlowTable;
use crate::ringbuf_poll;

/// Linux-only collector backed by eBPF kprobes.
///
/// Loads the compiled eBPF object, attaches kprobes to TCP/UDP kernel
/// functions, and forwards ring-buffer events into the `FlowTable`.
pub struct EbpfCollector {
    /// eBPF loader is wrapped in a mutex because `poll_ringbuf` needs `&mut Ebpf`.
    /// In practice this is uncontended: `run()` is called once and never returns.
    loader: Mutex<EbpfLoader>,
}

impl EbpfCollector {
    /// Load the eBPF programs and attach kprobes.
    ///
    /// Requires root privileges (for `CAP_BPF` or `CAP_SYS_ADMIN`).
    pub fn new(config: &Config) -> anyhow::Result<Self> {
        let loader = EbpfLoader::load(config)?;
        Ok(Self {
            loader: Mutex::new(loader),
        })
    }
}

#[async_trait]
impl Collector for EbpfCollector {
    async fn run(&self, flow_table: Arc<FlowTable>) -> anyhow::Result<()> {
        info!("eBPF collector starting");
        let mut loader = self.loader.lock().await;
        ringbuf_poll::poll_ringbuf(&mut loader.bpf, flow_table).await
    }
}
