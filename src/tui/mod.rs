pub mod app;
pub mod event;
pub mod ui;

use crate::error::AppError;
use crate::store::Store;
use app::App;
use event::{Event, EventHandler};
use ratatui::{Terminal, backend::CrosstermBackend, crossterm};
use std::io;

pub fn run(store: Store) -> Result<(), AppError> {
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    // Setup terminal
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(
        io::stdout(),
        crossterm::terminal::EnterAlternateScreen
    )?;

    let events = EventHandler::new(100);
    let mut app = App::new(store);

    let res = run_loop(&mut terminal, &mut app, events);

    // Restore terminal
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;

    res
}

fn run_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    events: EventHandler,
) -> Result<(), AppError> {
    while !app.should_quit {
        terminal.draw(|f| ui::render(app, f))?;

        match events.next()? {
            Event::Tick => app.tick()?,
            Event::Key(key_event) => app.handle_key_event(key_event)?,
            _ => {}
        }
    }
    Ok(())
}
