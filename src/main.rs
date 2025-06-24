mod app;
mod events;
mod models;
mod modes;
mod services;
mod ui;

use anyhow::Result;
use app::App;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{
    io,
    time::{Duration, Instant},
};
use tokio::time::sleep;

/// Main entry point for the automation toolkit
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the app
    let mut app = App::new();

    // Set up the terminal
    let mut terminal = setup_terminal()?;

    // Run the main application loop
    let result = run_app(&mut terminal, &mut app).await;

    // Restore the terminal
    restore_terminal(&mut terminal)?;

    result
}

/// Set up the terminal for TUI mode
fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    Ok(terminal)
}

/// Restore the terminal to normal mode
fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

/// Main application loop with async event handling
async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<()> {
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(50); // 20 FPS for smooth UI updates

    loop {
        // Handle terminal events (keyboard input, etc.)
        if event::poll(Duration::from_millis(0))? {
            match event::read()? {
                Event::Key(key_event) => {
                    events::handle_key_event(app, key_event).await?;
                }
                Event::Resize(_, _) => {
                    // Terminal was resized, redraw will happen automatically
                }
                _ => {
                    // Ignore other events (mouse, focus, etc.)
                }
            }
        }

        // Process any pending messages from background tasks
        app.process_messages().await?;

        // Check if we should quit
        if app.should_quit {
            break;
        }

        // Render the UI
        terminal.draw(|f| ui::render_app(f, app))?;

        // Handle timing for smooth updates
        let now = Instant::now();
        let elapsed = now.duration_since(last_tick);
        if elapsed < tick_rate {
            sleep(tick_rate - elapsed).await;
        }
        last_tick = now;
    }

    Ok(())
}

/// Handle panics gracefully by restoring the terminal
#[allow(dead_code)]
fn setup_panic_hook() {
    let original_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic_info| {
        // Attempt to restore the terminal
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);

        // Call the original panic hook
        original_hook(panic_info);
    }));
}

/// Helper function to initialize logging (optional)
#[allow(dead_code)]
fn init_logging() -> Result<()> {
    // You could set up file logging here if needed
    // For now, we're using the in-app logging system
    Ok(())
}
