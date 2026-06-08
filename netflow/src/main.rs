use std::sync::Arc;
use clap::Parser;
use tokio::signal;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod api;
mod collector;
mod config;
mod flow_table;
mod tui;

#[cfg(target_os = "linux")]
mod ebpf_loader;

use collector::Collector;
use config::{Cli, Config};
use flow_table::FlowTable;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let cli = Cli::parse();
    let config = Config::load(&cli.config)?;
    info!("loaded config from {:?}", cli.config);

    let flow_table = Arc::new(FlowTable::new());

    // Platform-aware backend selection.
    #[cfg(target_os = "linux")]
    let collector: Arc<dyn Collector> = {
        let c = collector::ebpf::EbpfCollector::new(&config)?;
        Arc::new(c)
    };
    #[cfg(not(target_os = "linux"))]
    let collector: Arc<dyn Collector> = {
        let c = collector::pcap::PcapCollector::new(&config)?;
        Arc::new(c)
    };

    let mut handles = vec![
        {
            let c = Arc::clone(&collector);
            let ft = Arc::clone(&flow_table);
            tokio::spawn(async move { c.run(ft).await })
        },
        tokio::spawn(api::serve(flow_table.clone(), config.clone())),
    ];

    if cli.tui {
        handles.push(tokio::spawn(tui::run(flow_table.clone())));
    } else {
        info!("running in API-only mode, press Ctrl+C to exit");
    }

    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("shutting down...");
        }
        _ = futures::future::join_all(handles) => {}
    }

    Ok(())
}
