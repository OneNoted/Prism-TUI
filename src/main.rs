mod actions;
mod app;
mod data;
mod error;
mod message;
mod theme;
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

    let data_dir = find_prism_data_dir()?;
    let config = PrismConfig::load(&data_dir)?;
    let mut app = App::new(config)?;
    let mut terminal = Terminal::new()?;
    let mut events = EventStream::new(Duration::from_millis(250));

    while app.running {
        terminal.draw(|frame| view::render(&mut app, frame))?;

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
