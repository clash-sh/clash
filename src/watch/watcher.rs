//! File system watcher with gitignore filtering

use super::state::WatchState;
use ignore::gitignore::Gitignore;
use notify::event::ModifyKind;
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::io;
use std::path::PathBuf;
use std::sync::mpsc;

/// Event type marker for git operations
pub(super) const EVENT_TYPE_GIT: &str = "__GIT__";

/// Event type marker for file changes
pub(super) const EVENT_TYPE_FILE: &str = "__FILE__";

/// Setup file system watcher for all worktree directories
pub fn setup_watcher(
    state: &mut WatchState,
    tx: mpsc::Sender<String>,
) -> io::Result<RecommendedWatcher> {
    // Load .gitignore from repository root for filtering
    // All worktrees share the same repo, so we use the first worktree's path
    let (gitignore, repo_root) = if let Some(first_wt) = state.worktrees.main() {
        let repo_root = first_wt.path.clone();
        let gitignore_path = repo_root.join(".gitignore");
        let (gi, err) = Gitignore::new(&gitignore_path);
        if let Some(e) = err {
            state.add_event(format!("Warning loading .gitignore: {}", e));
        }
        (gi, repo_root)
    } else {
        (Gitignore::empty(), PathBuf::new())
    };

    // Clone for the watcher callback
    let gitignore = gitignore.clone();
    let repo_root = repo_root.clone();

    // Create watcher with event filtering callback
    let mut watcher = RecommendedWatcher::new(
        move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res
                && should_process_event(&event, &gitignore, &repo_root)
            {
                // Send markers to distinguish git vs file events (filtered in app.rs)
                let full_path_str = event
                    .paths
                    .first()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "no-path".to_string());

                if full_path_str.contains("/.git/") {
                    let _ = tx.send(EVENT_TYPE_GIT.to_string());
                } else {
                    let _ = tx.send(EVENT_TYPE_FILE.to_string());
                }
            }
        },
        Config::default(),
    )
    .map_err(io::Error::other)?;

    // Watch all worktree directories
    for worktree in state.worktrees.iter() {
        watcher
            .watch(&worktree.path, RecursiveMode::Recursive)
            .map_err(io::Error::other)?;
    }
    state.add_event(format!("Watching {} directories", state.worktrees.len()));

    Ok(watcher)
}

/// Determine if a file system event should trigger a conflict refresh
fn should_process_event(event: &notify::Event, gitignore: &Gitignore, repo_root: &PathBuf) -> bool {
    // Use .gitignore to determine if path should be watched
    let is_relevant = event.paths.iter().any(|path| {
        let path_str = path.to_string_lossy();

        // Skip temporary files from editors and build systems
        if path_str.ends_with(".tmp")
            || path_str.ends_with(".swp")
            || path_str.ends_with("~")
            || path_str.contains(".tmp.")
        {
            return false;
        }

        // Watch for important git operations that change worktree status
        if path_str.contains("/.git/") {
            // Watch these git files - they indicate commits, checkouts, etc.
            // Also watch .lock files to trigger debounce (filtered from display later)
            return path_str.contains("/.git/index") ||      // Staging/commits
                   path_str.ends_with("index.lock") ||      // Staging in progress
                   path_str.contains("/.git/HEAD") ||        // Branch switches
                   path_str.ends_with("HEAD.lock") ||        // Branch switch in progress
                   path_str.contains("/.git/refs/") ||       // Commits (branch updates)
                   path_str.contains("/refs/") && path_str.ends_with(".lock") ||  // Commit in progress
                   path_str.contains("/.git/MERGE_HEAD") ||  // Merge in progress
                   path_str.contains("/.git/REBASE_HEAD"); // Rebase in progress
        }

        // Make path relative to repo root for gitignore matching
        let relative_path = if let Ok(rel) = path.strip_prefix(repo_root) {
            rel
        } else {
            // If we can't make it relative, use the full path
            path.as_path()
        };

        // Check against .gitignore (including parent directories)
        let is_dir = path.is_dir();
        let match_result = gitignore.matched_path_or_any_parents(relative_path, is_dir);

        !match_result.is_ignore() // Relevant if NOT ignored
    });

    if !is_relevant {
        return false;
    }

    // Filter for only actual content changes, not metadata
    matches!(
        event.kind,
        EventKind::Modify(ModifyKind::Data(_)) |  // Actual content changes
        EventKind::Modify(ModifyKind::Name(_)) |  // Renames (git atomic operations)
        EventKind::Create(_) |                     // New files
        EventKind::Remove(_) // Deleted files
    )
}
