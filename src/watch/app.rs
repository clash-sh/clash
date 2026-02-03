//! Event loop and application orchestration

use super::{
    state::WatchState,
    ui,
    watcher::{self, EVENT_TYPE_FILE, EVENT_TYPE_GIT},
};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::Terminal;
use std::io;
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// Run the main application event loop
pub fn run_app<B>(terminal: &mut Terminal<B>, state: &mut WatchState) -> io::Result<()>
where
    B: ratatui::backend::Backend,
    B::Error: Into<io::Error>,
{
    // Set up file system watcher
    let (tx, rx) = mpsc::channel();
    let _watcher = watcher::setup_watcher(state, tx)?;

    // Debouncing: track when we last saw an event
    let mut last_event_time: Option<Instant> = None;
    let mut last_event_type: &str = "file"; // Track whether last event was git or file
    let debounce_duration = Duration::from_secs(1); // Wait 1 second after last event

    loop {
        // Draw UI - continue on error to prevent panic
        if let Err(e) = terminal.draw(|f| ui::ui(f, state)).map_err(Into::into) {
            // Log the error but don't crash
            state.add_event(format!("UI draw error: {} - attempting recovery", e));
            // Try to continue; the next draw might succeed
        }

        // Check for file system events (non-blocking)
        if let Ok(event_msg) = rx.try_recv() {
            // Track event type but don't show markers in event log
            if event_msg == EVENT_TYPE_GIT {
                last_event_type = "git";
            } else if event_msg == EVENT_TYPE_FILE {
                last_event_type = "file";
            } else if !event_msg.is_empty() {
                // Only add non-marker messages to event log
                state.add_event(event_msg);
            }
            // Record that we got an event but don't refresh yet
            last_event_time = Some(Instant::now());
        }

        // Check if we should refresh (debounce period elapsed)
        if let Some(last_time) = last_event_time
            && last_time.elapsed() > debounce_duration
        {
            // Enough time has passed, do the refresh
            if let Err(e) = state.refresh_conflicts() {
                state.add_event(format!("Refresh error: {}", e));
            } else {
                // Consolidated single-line message with event type prefix
                let prefix = if last_event_type == "git" {
                    "Git operation - "
                } else {
                    "Files changed - "
                };

                let msg = if state.conflicts.is_empty() {
                    format!(
                        "{}Refreshed: {} worktrees, no conflicts",
                        prefix,
                        state.worktrees.len()
                    )
                } else {
                    let unique_files = state.count_unique_conflict_files();
                    format!(
                        "{}Refreshed: {} worktrees, {} conflicts affecting {} files",
                        prefix,
                        state.worktrees.len(),
                        state.conflicts.len(),
                        unique_files
                    )
                };
                state.add_event(msg);
            }
            last_event_time = None; // Reset the timer
        }

        // Poll for keyboard events with timeout - handle errors gracefully
        match event::poll(Duration::from_millis(100)) {
            Ok(true) => {
                // Event available, try to read it
                match event::read() {
                    Ok(Event::Key(key)) => {
                        // Handle CTRL+C (crossterm captures it in raw mode)
                        if key.code == KeyCode::Char('c')
                            && key.modifiers.contains(KeyModifiers::CONTROL)
                        {
                            state.add_event("CTRL+C pressed, exiting...".to_string());
                            return Ok(());
                        }

                        match key.code {
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Char('r') => {
                                if let Err(e) = state.refresh_conflicts() {
                                    state.add_event(format!("Manual refresh error: {}", e));
                                } else {
                                    // Consolidated single-line message
                                    let msg = if state.conflicts.is_empty() {
                                        format!(
                                            "Manual refresh: {} worktrees, no conflicts",
                                            state.worktrees.len()
                                        )
                                    } else {
                                        let unique_files = state.count_unique_conflict_files();
                                        format!(
                                            "Manual refresh: {} worktrees, {} conflicts affecting {} files",
                                            state.worktrees.len(),
                                            state.conflicts.len(),
                                            unique_files
                                        )
                                    };
                                    state.add_event(msg);
                                }
                            }
                            // Scrolling controls for events window
                            KeyCode::Up => {
                                // Event window is 10 lines high, minus 2 for borders = 8 visible lines
                                let window_height = 8;

                                let current_scroll = match state.events_scroll {
                                    None => {
                                        // We're at bottom, calculate where we actually are
                                        state.events.len().saturating_sub(window_height)
                                    }
                                    Some(scroll) => scroll,
                                };

                                if current_scroll > 0 {
                                    // Move up by one, setting manual scroll position
                                    state.events_scroll = Some(current_scroll - 1);
                                }
                            }
                            KeyCode::Down => {
                                if let Some(scroll) = state.events_scroll {
                                    let window_height = 8;
                                    let max_scroll =
                                        state.events.len().saturating_sub(window_height);

                                    if scroll < max_scroll {
                                        // Move down by one
                                        state.events_scroll = Some(scroll + 1);
                                    } else {
                                        // We've reached the bottom, switch back to auto-scroll
                                        state.events_scroll = None;
                                    }
                                }
                                // If already at bottom (None), do nothing
                            }
                            _ => {}
                        }
                    }
                    Ok(_) => {
                        // Other event types (mouse, resize, etc.) - ignore
                    }
                    Err(e) => {
                        // Error reading event - log but continue
                        state.add_event(format!("Event read error: {} - continuing", e));
                    }
                }
            }
            Ok(false) => {
                // No events available, continue
            }
            Err(e) => {
                // Error polling - log but continue
                state.add_event(format!("Event poll error: {} - continuing", e));
            }
        }
    }
}
