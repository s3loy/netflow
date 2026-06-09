use aya::{Ebpf, include_bytes_aligned, programs::KProbe};
use aya_log::EbpfLogger;
use tracing::{info, warn};

use crate::config::Config;

pub struct EbpfLoader {
    pub bpf: Ebpf,
}

impl EbpfLoader {
    pub fn load(_config: &Config) -> anyhow::Result<Self> {
        let mut bpf = Ebpf::load(include_bytes_aligned!(concat!(env!("OUT_DIR"), "/netflow")))?;

        if let Err(e) = EbpfLogger::init(&mut bpf) {
            warn!("failed to initialize eBPF logger: {}", e);
        }

        let mut loader = Self { bpf };
        loader.attach_kprobes()?;

        info!("eBPF programs loaded and attached");
        Ok(loader)
    }

    fn attach_kprobes(&mut self) -> anyhow::Result<()> {
        let probes = [
            ("netflow_tcp_set_state", "tcp_set_state"),
            ("netflow_tcp_cleanup_rbuf", "tcp_cleanup_rbuf"),
            ("netflow_tcp_sendmsg", "tcp_sendmsg"),
            ("netflow_udp_sendmsg", "udp_sendmsg"),
            ("netflow_udp_rcv", "udp_rcv"),
        ];

        for (prog_name, fn_name) in probes {
            let program: &mut KProbe = self
                .bpf
                .program_mut(prog_name)
                .ok_or_else(|| anyhow::anyhow!("program {} not found", prog_name))?
                .try_into()?;
            program.load()?;
            program.attach(fn_name, 0)?;
            info!("attached kprobe: {} -> {}", prog_name, fn_name);
        }

        Ok(())
    }
}
