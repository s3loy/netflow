use std::sync::Arc;
use aya::maps::RingBuf;
use aya::Ebpf;
use netflow_common::FlowEvent;
use tracing::{debug, info};
use crate::flow_table::FlowTable;

pub async fn poll_ringbuf(bpf: &mut Ebpf, flow_table: Arc<FlowTable>) -> anyhow::Result<()> {
    let flow_events = bpf
        .map_mut("FLOW_EVENTS")
        .ok_or_else(|| anyhow::anyhow!("FLOW_EVENTS map not found"))?;
    let mut ringbuf = RingBuf::try_from(flow_events)?;

    info!("started ringbuf polling");

    loop {
        while let Some(item) = ringbuf.next() {
            let ptr = item.as_ptr() as *const FlowEvent;
            let event = unsafe { ptr.read_unaligned() };
            debug!("ringbuf event: {:?}", event.ty as u8);
            flow_table.handle_event(event);
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }
}
