//! Application state for watch mode

use clash_sh::WorktreeManager;
use std::collections::VecDeque;

/// Maximum number of events to keep in memory
const MAX_EVENTS: usize = 1000;

/// Watch mode application state
pub struct WatchState {
    pub worktrees: WorktreeManager,
    pub conflicts: Vec<(String, String, Vec<String>)>, // (wt1, wt2, files)
    pub events: VecDeque<String>,                      // Changed to VecDeque for efficient removal
    pub events_scroll: Option<usize>, // None = stick to bottom, Some(n) = show from event n
}

impl WatchState {
    /// Create WatchState with initial worktrees
    pub fn with_worktrees(worktrees: WorktreeManager) -> Self {
        let mut state = Self {
            worktrees,
            conflicts: Vec::new(),
            events: VecDeque::with_capacity(MAX_EVENTS),
            events_scroll: None, // None = stick to bottom
        };

        state.add_event("Watch mode started".to_string());
        state.add_event(format!("Found {} worktrees", state.worktrees.len()));
        state.check_conflicts();

        state
    }

    pub fn add_event(&mut self, event: String) {
        // Use actual system time for better debugging
        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
        let formatted_event = format!("[{}] {}", timestamp, event);

        // Add new event
        self.events.push_back(formatted_event);

        // Remove oldest events if we exceed the limit
        while self.events.len() > MAX_EVENTS {
            self.events.pop_front();
            // Adjust scroll position if needed
            if let Some(scroll) = self.events_scroll
                && scroll > 0
            {
                self.events_scroll = Some(scroll.saturating_sub(1));
            }
        }

        // If events_scroll is None, we stay at bottom (UI will handle it)
        // If it's Some(n), we preserve the manual scroll position
    }

    /// Re-discover worktrees and check for conflicts (called by file watcher)
    pub fn refresh_conflicts(&mut self) -> Result<(), String> {
        // Re-discover worktrees
        self.worktrees.refresh().map_err(|e| e.to_string())?;

        // Check for conflicts
        self.check_conflicts();

        Ok(())
    }

    /// Check for conflicts between current worktrees
    fn check_conflicts(&mut self) {
        // Clear old conflicts
        self.conflicts.clear();

        // Collect errors to add to events later
        let mut errors = Vec::new();

        // Check all pairs of worktrees for conflicts
        {
            let all = self.worktrees.all();
            for i in 0..all.len() {
                for j in (i + 1)..all.len() {
                    let wt1 = &all[i];
                    let wt2 = &all[j];

                    match wt1.conflicts_with(wt2) {
                        Ok(conflicting_files) => {
                            if !conflicting_files.is_empty() {
                                self.conflicts.push((
                                    wt1.branch.clone(),
                                    wt2.branch.clone(),
                                    conflicting_files,
                                ));
                            }
                        }
                        Err(e) => {
                            errors.push(format!(
                                "Error checking {}/{}: {}",
                                wt1.branch, wt2.branch, e
                            ));
                        }
                    }
                }
            }
        }

        // Now add the error events
        for error in errors {
            self.add_event(error);
        }
    }

    /// Count unique files affected by conflicts
    pub fn count_unique_conflict_files(&self) -> usize {
        let mut unique_files = std::collections::HashSet::new();
        for (_, _, files) in &self.conflicts {
            for file in files {
                unique_files.insert(file.clone());
            }
        }
        unique_files.len()
    }
}
