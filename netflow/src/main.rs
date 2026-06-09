use std::sync::Arc;
use clap::Parser;
use tokio::signal;
use tokio::task::JoinHandle;
use tracing::{error, info, Level};
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
    let cli = Cli::parse();

    // TUI draws on stdout; keep logs on stderr and quiet them down so they
    // don't corrupt the terminal UI.
    let log_level = if cli.tui { Level::WARN } else { Level::INFO };
    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .with_writer(std::io::stderr)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

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

    let collector_handle: JoinHandle<anyhow::Result<()>> = {
        let c = Arc::clone(&collector);
        let ft = Arc::clone(&flow_table);
        tokio::spawn(async move { c.run(ft).await })
    };
    let api_handle = tokio::spawn(api::serve(flow_table.clone(), config.clone()));
    let mut tui_handle: Option<JoinHandle<anyhow::Result<()>>> = if cli.tui {
        Some(tokio::spawn(tui::run(flow_table.clone())))
    } else {
        info!("running in API-only mode, press Ctrl+C to exit");
        None
    };

    let collector_abort = collector_handle.abort_handle();
    let api_abort = api_handle.abort_handle();
    let tui_abort = tui_handle.as_ref().map(|h| h.abort_handle());

    tokio::select! {
        biased;
        _ = signal::ctrl_c() => {
            info!("shutting down...");
        }
        res = collector_handle => {
            match res {
                Ok(Ok(())) => info!("collector finished"),
                Ok(Err(e)) => error!("collector failed: {e}"),
                Err(e) => error!("collector panicked: {e}"),
            }
        }
        res = api_handle => {
            match res {
                Ok(Ok(())) => info!("api server finished"),
                Ok(Err(e)) => error!("api server failed: {e}"),
                Err(e) => error!("api server panicked: {e}"),
            }
        }
        Some(res) = async { match tui_handle.as_mut() { Some(h) => Some(h.await), None => None } } => {
            match res {
                Ok(Ok(())) => {
                    info!("tui finished, shutting down...");
                    // User exited TUI (pressed 'q'). Collector uses spawn_blocking
                    // which hangs on pcap::next_packet(); force exit to avoid
                    // waiting indefinitely for network traffic.
                    std::process::exit(0);
                }
                Ok(Err(e)) => error!("tui failed: {e}"),
                Err(e) => error!("tui panicked: {e}"),
            }
        }
    }

    // Abort any remaining tasks so the process exits cleanly.
    collector_abort.abort();
    api_abort.abort();
    if let Some(h) = tui_abort {
        h.abort();
    }

    // Give tasks a moment to clean up (e.g. TUI restoring terminal state).
    // Avoid re-awaiting a handle that was already consumed by the select branch.
    if let Some(h) = tui_handle {
        if !h.is_finished() {
            let _ = tokio::time::timeout(std::time::Duration::from_millis(500), h).await;
        }
    }

    // Final safety net: ensure raw mode is disabled even if TUI task was
    // aborted before its RawModeGuard Drop ran.
    let _ = crossterm::terminal::disable_raw_mode();

    Ok(())
}
