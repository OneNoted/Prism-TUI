mod actions;
mod app;
mod data;
mod error;
mod message;
pub mod theme;
mod tui;
mod update;
mod view;

use app::App;
use color_eyre::Result;
use data::{PrismConfig, find_prism_data_dir};
use message::Message;
use std::time::Duration;
use tui::{Event, EventStream, Terminal};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    // Find PrismLauncher data directory
    let data_dir = find_prism_data_dir().map_err(|e| {
        eprintln!("Error: {}", e);
        eprintln!("Make sure PrismLauncher is installed and has been run at least once.");
        e
    })?;

    // Load configuration
    let config = PrismConfig::load(&data_dir)?;

    // Initialize app state
    let mut app = App::new(config)?;

    // Initialize terminal
    let mut terminal = Terminal::new()?;

    // Event stream
    let mut events = EventStream::new(Duration::from_millis(250));

    // Main loop
    while app.running {
        // Render
        terminal.draw(|frame| view::render(&mut app, frame))?;

        // Handle events
        if let Some(event) = events.next().await {
            let msg = match event {
                Event::Key(key) => Message::Key(key),
                Event::Mouse(mouse) => Message::Mouse(mouse),
                Event::Tick => Message::Tick,
                Event::Resize(_, _) => Message::Tick, // Trigger redraw
            };
            update::update(&mut app, msg);
        }
    }

    Ok(())
}
