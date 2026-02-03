//! UI rendering for watch mode

use super::state::WatchState;
use clash_sh::WorktreeStatus;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

/// Render the TUI interface
pub fn ui(f: &mut Frame, state: &WatchState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(5),     // Main content
            Constraint::Length(10), // Event log
            Constraint::Length(3),  // Instructions
        ])
        .split(f.area());

    render_title(f, chunks[0], state);
    render_main_content(f, chunks[1], state);
    render_events(f, chunks[2], state);
    render_instructions(f, chunks[3]);
}

/// Render the title/status section
fn render_title(f: &mut Frame, area: Rect, state: &WatchState) {
    let title = Paragraph::new(vec![
        Line::from(vec![Span::styled(
            "Clash Watch Mode",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::raw(format!(
            "Monitoring {} worktrees",
            state.worktrees.len()
        ))]),
    ])
    .block(Block::default().borders(Borders::ALL).title("Status"));
    f.render_widget(title, area);
}

/// Render the main content area (worktrees + conflicts side-by-side)
fn render_main_content(f: &mut Frame, area: Rect, state: &WatchState) {
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    render_worktrees_list(f, main_chunks[0], state);
    render_conflicts(f, main_chunks[1], state);
}

/// Render the worktrees list
fn render_worktrees_list(f: &mut Frame, area: Rect, state: &WatchState) {
    let mut worktrees: Vec<ListItem> = state
        .worktrees
        .iter()
        .map(|wt| {
            let style = match wt.status {
                WorktreeStatus::Clean => Style::default().fg(Color::Green),
                WorktreeStatus::Dirty => Style::default().fg(Color::Yellow),
                _ => Style::default(),
            };
            ListItem::new(format!("{} [{}]", wt.branch, wt.status)).style(style)
        })
        .collect();

    // Add legend at the bottom
    if !worktrees.is_empty() {
        worktrees.push(ListItem::new(""));
        worktrees.push(ListItem::new(""));
        worktrees.push(ListItem::new(""));
        worktrees.push(ListItem::new(""));
        worktrees.push(ListItem::new(Line::from(vec![
            Span::styled(
                "clean",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::DIM),
            ),
            Span::styled(
                " = no uncommitted changes",
                Style::default().fg(Color::Rgb(128, 128, 128)),
            ),
        ])));
        worktrees.push(ListItem::new(Line::from(vec![
            Span::styled(
                "dirty",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::DIM),
            ),
            Span::styled(
                " = has uncommitted changes",
                Style::default().fg(Color::Rgb(128, 128, 128)),
            ),
        ])));
    }

    let worktrees_list = List::new(worktrees)
        .block(Block::default().borders(Borders::ALL).title("Worktrees"))
        .style(Style::default().fg(Color::White));
    f.render_widget(worktrees_list, area);
}

/// Render the conflicts display
fn render_conflicts(f: &mut Frame, area: Rect, state: &WatchState) {
    let mut conflict_text = if state.conflicts.is_empty() {
        vec![Line::from(Span::styled(
            "✓ No conflicts detected",
            Style::default().fg(Color::Green),
        ))]
    } else {
        let mut lines = vec![Line::from(Span::styled(
            format!("⚠ {} wt conflict(s) detected", state.conflicts.len()),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ))];

        for (wt1, wt2, files) in &state.conflicts {
            lines.push(Line::from(""));
            lines.push(Line::from(format!("{} ↔ {}", wt1, wt2)));
            for file in files {
                lines.push(Line::from(format!("  - {}", file)));
            }
        }
        lines
    };

    // Add legend explaining conflict detection basis
    conflict_text.push(Line::from(""));
    conflict_text.push(Line::from(""));
    conflict_text.push(Line::from(Span::styled(
        "Conflicts shown are based on",
        Style::default().fg(Color::Rgb(128, 128, 128)),
    )));
    conflict_text.push(Line::from(Span::styled(
        "committed changes, not",
        Style::default().fg(Color::Rgb(128, 128, 128)),
    )));
    conflict_text.push(Line::from(Span::styled(
        "uncommitted working directory edits",
        Style::default().fg(Color::Rgb(128, 128, 128)),
    )));

    let conflicts = Paragraph::new(conflict_text)
        .block(Block::default().borders(Borders::ALL).title("Conflicts"));
    f.render_widget(conflicts, area);
}

/// Render the scrollable event log
fn render_events(f: &mut Frame, area: Rect, state: &WatchState) {
    let available_height = area.height.saturating_sub(2) as usize; // -2 for borders

    // Calculate visible range of events
    let total_events = state.events.len();

    // Calculate start index based on scroll position
    let start_idx = match state.events_scroll {
        None => {
            // Stick to bottom - show the last available_height events
            total_events.saturating_sub(available_height)
        }
        Some(scroll) => {
            // Manual scroll position - clamp to valid range
            let max_scroll = total_events.saturating_sub(available_height);
            scroll.min(max_scroll)
        }
    };
    let end_idx = (start_idx + available_height).min(total_events);

    // Get visible events
    // Create visible events from the VecDeque by converting it to an iterator
    let visible_events: Vec<ListItem> = state
        .events
        .iter()
        .skip(start_idx)
        .take(end_idx - start_idx)
        .map(|e| ListItem::new(e.as_str()))
        .collect();

    // Create title with scroll indicator
    let title = if total_events > available_height {
        format!("Events [{}-{}/{}]", start_idx + 1, end_idx, total_events)
    } else if total_events == 0 {
        "Events [empty]".to_string()
    } else {
        format!("Events [{}]", total_events)
    };

    let events_list =
        List::new(visible_events).block(Block::default().borders(Borders::ALL).title(title));
    f.render_widget(events_list, area);
}

/// Render the keyboard instructions
fn render_instructions(f: &mut Frame, area: Rect) {
    let instructions = Paragraph::new(vec![Line::from(vec![
        Span::styled(
            "q",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" quit  "),
        Span::styled(
            "r",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" refresh  "),
        Span::styled(
            "↑↓",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" scroll events"),
    ])])
    .block(Block::default().borders(Borders::ALL).title("Keys"));
    f.render_widget(instructions, area);
}
