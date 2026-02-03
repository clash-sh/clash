//! Real-time conflict monitoring with TUI

mod app;
mod state;
mod ui;
mod watcher;

pub use state::WatchState;

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

/// Entry point for watch mode - sets up terminal and runs the TUI
pub fn run_watch_mode(worktrees: clash_sh::WorktreeManager) -> Result<(), io::Error> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state with initial worktrees
    let mut state = WatchState::with_worktrees(worktrees);

    // Run the app (CTRL+C is handled as a keyboard event in raw mode)
    let res = app::run_app(&mut terminal, &mut state);

    // Restore terminal (always runs, even if interrupted)
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {}", err);
    }

    Ok(())
}
// Main watch updates
