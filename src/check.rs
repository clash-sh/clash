use clash_sh::WorktreeManager;
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize)]
struct CheckOutput {
    file: String,
    current_worktree: String,
    current_branch: String,
    conflicts: Vec<FileConflict>,
}

#[derive(Debug, Serialize)]
struct FileConflict {
    worktree: String,
    branch: String,
    has_merge_conflict: bool,
    has_active_changes: bool,
}

/// Run the check command - checks a single file for conflicts across worktrees.
///
/// Returns true if conflicts were found (caller should exit with code 2).
pub fn run_check(worktrees: &WorktreeManager, path: &str) -> bool {
    let current_dir = match std::env::current_dir() {
        Ok(dir) => dir.canonicalize().unwrap_or(dir),
        Err(e) => {
            eprintln!("Error: failed to get current directory: {}", e);
            return true;
        }
    };

    // Find which worktree we're running from
    let current_wt = match worktrees.find_containing(&current_dir) {
        Some(wt) => wt,
        None => {
            eprintln!("Error: not running from within a known worktree");
            return true;
        }
    };

    // Make path relative to worktree root
    let normalized_path = to_repo_relative(path, &current_wt.path);

    let mut conflicts = Vec::new();

    for other_wt in worktrees.iter() {
        if other_wt.id == current_wt.id {
            continue;
        }

        // Check merge conflicts between current and other worktree
        let conflicting_files = current_wt.conflicts_with(other_wt).unwrap_or_default();
        let has_merge_conflict = conflicting_files.iter().any(|f| f == &normalized_path);

        // Check if file has uncommitted changes in other worktree
        let has_active_changes = check_file_active(&other_wt.path, &normalized_path);

        if has_merge_conflict || has_active_changes {
            conflicts.push(FileConflict {
                worktree: other_wt.id.clone(),
                branch: other_wt.branch.clone(),
                has_merge_conflict,
                has_active_changes,
            });
        }
    }

    let is_conflicted = !conflicts.is_empty();

    let output = CheckOutput {
        file: normalized_path,
        current_worktree: current_wt.id.clone(),
        current_branch: current_wt.branch.clone(),
        conflicts,
    };

    match serde_json::to_string_pretty(&output) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("Error serializing output: {}", e),
    }

    is_conflicted
}

/// Convert a file path to be relative to the worktree root.
///
/// Relative paths are returned as-is (assumed repo-relative).
/// Absolute paths are stripped of the worktree root prefix.
fn to_repo_relative(path: &str, worktree_root: &Path) -> String {
    let path_buf = PathBuf::from(path);

    if path_buf.is_absolute() {
        path_buf
            .strip_prefix(worktree_root)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| path.to_string())
    } else {
        path.to_string()
    }
}

/// Check if a file has uncommitted changes in a worktree.
///
/// Compares the file on disk against its blob in HEAD using gix.
/// Returns true if the file differs from HEAD (modified/new/deleted).
fn check_file_active(worktree_path: &Path, file_path: &str) -> bool {
    let repo = match gix::open(worktree_path) {
        Ok(r) => r,
        Err(_) => return false,
    };

    let workdir = match repo.workdir() {
        Some(p) => p.to_path_buf(),
        None => return false,
    };

    let disk_path = workdir.join(file_path);
    let file_on_disk = disk_path.exists();

    // Get the file's blob from HEAD
    let head_blob = head_file_contents(&repo, file_path);

    match (head_blob, file_on_disk) {
        (None, false) => false,   // Not in HEAD, not on disk: no change
        (None, true) => true,     // New file (not tracked in HEAD)
        (Some(_), false) => true, // Deleted from disk
        (Some(head_data), true) => {
            // Compare HEAD blob with file on disk
            match std::fs::read(&disk_path) {
                Ok(disk_data) => head_data != disk_data,
                Err(_) => false,
            }
        }
    }
}

/// Get the contents of a file from HEAD in the given repository.
fn head_file_contents(repo: &gix::Repository, file_path: &str) -> Option<Vec<u8>> {
    let mut head = repo.head().ok()?;
    let head_id = head.try_peel_to_id().ok()??;
    let commit = repo.find_object(head_id).ok()?.try_into_commit().ok()?;
    let mut tree = commit.tree().ok()?;
    let entry = tree.peel_to_entry_by_path(file_path).ok()??;
    let blob = repo.find_object(entry.id()).ok()?;
    Some(blob.data.to_vec())
}
