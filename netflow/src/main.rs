use std::sync::Arc;
use clap::Parser;
use tokio::signal;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod api;
mod config;
mod ebpf_loader;
mod flow_table;
mod iterator_poll;
mod ringbuf_poll;
mod tui;

use config::{Cli, Config};
use ebpf_loader::EbpfLoader;
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

    let mut loader = EbpfLoader::load(&config)?;
    let flow_table = Arc::new(FlowTable::new());

    let mut handles = vec![
        tokio::spawn(ringbuf_poll::poll_ringbuf(
            &mut loader.bpf,
            flow_table.clone(),
        )),
        tokio::spawn(api::serve(flow_table.clone(), &config)),
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
