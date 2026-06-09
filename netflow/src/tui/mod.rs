use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use crossterm::{
    event::Event,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use tokio::time::interval;
use tracing::{info, warn};

use crate::flow_table::FlowTable;

mod app;
mod dashboard;
mod detail_modal;
mod event;
mod format;
mod help_bar;
mod list_view;

pub use app::AppState;

/// Ensures raw mode is disabled when the TUI task is cancelled or panics.
struct RawModeGuard;

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
    }
}

/// Spawn a blocking thread that reads crossterm events and forwards them
/// over an async channel. Uses `std::thread` instead of `tokio::spawn_blocking`
/// so the thread is not tracked by the tokio runtime — the process can exit
/// cleanly without waiting for the thread to unblock from `read()`.
fn spawn_event_reader(
    tx: tokio::sync::mpsc::Sender<std::io::Result<Event>>,
    running: Arc<AtomicBool>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        while running.load(Ordering::Relaxed) {
            match crossterm::event::read() {
                Ok(event) => {
                    if tx.blocking_send(Ok(event)).is_err() {
                        break;
                    }
                }
                Err(e) => {
                    let _ = tx.blocking_send(Err(e));
                    break;
                }
            }
        }
    })
}

pub async fn run(flow_table: Arc<FlowTable>) -> anyhow::Result<()> {
    enable_raw_mode()?;
    let _raw_guard = RawModeGuard;
    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;
    terminal.clear()?;

    let mut state = AppState::new();
    let mut tick = interval(Duration::from_millis(100));

    let running = Arc::new(AtomicBool::new(true));
    let (event_tx, mut event_rx) = tokio::sync::mpsc::channel::<std::io::Result<Event>>(32);
    let _reader_handle = spawn_event_reader(event_tx, Arc::clone(&running));

    info!("TUI started");

    loop {
        // Fetch latest flows before each render
        let flows = flow_table.all_flows();
        state.update_flows(flows);

        // Draw — capture list area height for navigation calculations
        let mut list_area_height = 0u16;
        terminal.draw(|f| {
            let area = f.area();
            let chunks = ratatui::layout::Layout::default()
                .direction(ratatui::layout::Direction::Vertical)
                .constraints([
                    ratatui::layout::Constraint::Length(3),
                    ratatui::layout::Constraint::Min(0),
                    ratatui::layout::Constraint::Length(1),
                ])
                .split(area);
            list_area_height = chunks[1].height;
            dashboard::render_dashboard(f, chunks[0], &state.flows);
            list_view::render(f, chunks[1], &state);
            help_bar::render(f, chunks[2], state.modal_open);
            if state.modal_open {
                let modal_lines = detail_modal::render(f, area, &state);
                state.set_modal_max_scroll(modal_lines);
            }
        })?;

        let view_height = list_view::visible_row_count_from_height(list_area_height);

        // Event loop
        let action = tokio::select! {
            _ = tick.tick() => app::Action::Tick,
            maybe_event = event_rx.recv() => {
                match maybe_event {
                    Some(Ok(event)) => {
                        let action = event::handle_event(event);
                        if state.modal_open {
                            event::modal_action(action)
                        } else {
                            action
                        }
                    }
                    Some(Err(e)) => {
                        warn!("TUI event error: {}", e);
                        app::Action::None
                    }
                    None => app::Action::Quit,
                }
            }
        };

        match action {
            app::Action::Quit => {
                running.store(false, Ordering::Relaxed);
                break;
            }
            app::Action::Tick => {}
            app::Action::None => {}
            app::Action::Up => state.cursor_up(view_height),
            app::Action::Down => state.cursor_down(view_height),
            app::Action::PageUp => state.page_up(view_height),
            app::Action::PageDown => state.page_down(view_height),
            app::Action::Top => state.cursor_top(),
            app::Action::Bottom => state.cursor_bottom(view_height),
            app::Action::ToggleModal => {
                if state.modal_open {
                    state.close_modal();
                } else {
                    state.open_modal();
                }
            }
            app::Action::ModalUp => state.modal_up(),
            app::Action::ModalDown => state.modal_down(),
            app::Action::ModalPageUp => state.modal_page_up(view_height),
            app::Action::ModalPageDown => state.modal_page_down(view_height),
            app::Action::ModalTop => state.modal_top(),
            app::Action::ModalBottom => state.modal_bottom(),
        }
    }

    terminal.clear()?;
    disable_raw_mode()?;
    info!("TUI stopped");
    Ok(())
}
