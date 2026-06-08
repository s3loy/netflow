use std::sync::Arc;
use std::time::Duration;
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use tokio::time::interval;
use tracing::info;
use crate::flow_table::FlowTable;

mod dashboard;
mod table;

pub async fn run(flow_table: Arc<FlowTable>) -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;
    terminal.clear()?;

    let mut tick = interval(Duration::from_millis(100));

    info!("TUI started");

    loop {
        tick.tick().await;

        if event::poll(Duration::from_millis(0))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }

        let flows = flow_table.active_flows();
        terminal.draw(|f| {
            let area = f.area();
            let chunks = ratatui::layout::Layout::default()
                .direction(ratatui::layout::Direction::Vertical)
                .constraints([
                    ratatui::layout::Constraint::Length(3),
                    ratatui::layout::Constraint::Min(0),
                ])
                .split(area);
            dashboard::render_dashboard(f, chunks[0], &flows);
            table::render_flow_table(f, chunks[1], &flows);
        })?;
    }

    disable_raw_mode()?;
    Ok(())
}
