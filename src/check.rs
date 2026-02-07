use clash_sh::{Worktree, WorktreeManager};
use serde::Serialize;
use std::path::{Path, PathBuf};

// ============================================================================
// Output types
// ============================================================================

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

// ============================================================================
// Error type
// ============================================================================

/// Errors specific to the check command.
///
/// These map to exit code 1 (operational error) — distinct from
/// exit code 2 (conflicts found) and exit code 0 (clear).
#[derive(Debug)]
pub enum CheckError {
    /// Could not determine current directory
    CurrentDir(std::io::Error),
    /// The resolved file path is not inside any known worktree
    NotInWorktree(PathBuf),
    /// Could not strip worktree prefix from path
    PathResolution(PathBuf),
    /// Merge conflict detection failed for a worktree pair
    ConflictDetection { worktree: String, reason: String },
}

impl std::fmt::Display for CheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CurrentDir(e) => write!(f, "failed to get current directory: {}", e),
            Self::NotInWorktree(p) => {
                write!(f, "path '{}' is not inside any known worktree", p.display())
            }
            Self::PathResolution(p) => {
                write!(
                    f,
                    "could not resolve '{}' relative to worktree root",
                    p.display()
                )
            }
            Self::ConflictDetection { worktree, reason } => {
                write!(
                    f,
                    "conflict check failed for worktree '{}': {}",
                    worktree, reason
                )
            }
        }
    }
}

// ============================================================================
// Main entry point
// ============================================================================

/// Check a single file for conflicts across worktrees.
///
/// Prints JSON to stdout and returns whether conflicts were found.
/// - `Ok(false)` — no conflicts, exit 0
/// - `Ok(true)` — conflicts found, exit 2
/// - `Err(e)` — operational error, caller prints to stderr and exits 1
pub fn run_check(worktrees: &WorktreeManager, path: &str) -> Result<bool, CheckError> {
    let (current_wt, repo_relative) = resolve_file_path(path, worktrees)?;

    let mut conflicts = Vec::new();

    for other_wt in worktrees.iter() {
        if other_wt.id == current_wt.id {
            continue;
        }

        let merge_conflicts =
            current_wt
                .conflicts_with(other_wt)
                .map_err(|e| CheckError::ConflictDetection {
                    worktree: other_wt.id.clone(),
                    reason: e.to_string(),
                })?;

        let has_merge_conflict = merge_conflicts.iter().any(|f| f == &repo_relative);
        let has_active_changes = file_has_active_changes(&other_wt.path, &repo_relative);

        if has_merge_conflict || has_active_changes {
            conflicts.push(FileConflict {
                worktree: other_wt.id.clone(),
                branch: other_wt.branch.clone(),
                has_merge_conflict,
                has_active_changes,
            });
        }
    }

    let has_conflicts = !conflicts.is_empty();

    let output = CheckOutput {
        file: repo_relative,
        current_worktree: current_wt.id.clone(),
        current_branch: current_wt.branch.clone(),
        conflicts,
    };

    // Serialization of simple String/bool fields cannot fail in practice
    let json = serde_json::to_string_pretty(&output).expect("CheckOutput is always serializable");
    println!("{}", json);

    Ok(has_conflicts)
}

// ============================================================================
// Path resolution
// ============================================================================

/// Resolve a file path to its containing worktree and repo-relative path.
///
/// Handles both absolute paths (from hooks, e.g. `/repo/src/auth.rs`)
/// and relative paths (from CLI, e.g. `src/auth.rs` or `../lib/file.rs`).
///
/// Strategy:
/// 1. If relative, join with canonicalized cwd to make absolute
/// 2. Canonicalize if possible (resolves symlinks and `..` components)
/// 3. Walk up the path to find the containing worktree
/// 4. Strip the worktree prefix to get the repo-relative path
fn resolve_file_path<'a>(
    path: &str,
    worktrees: &'a WorktreeManager,
) -> Result<(&'a Worktree, String), CheckError> {
    let input = Path::new(path);

    let abs_path = if input.is_absolute() {
        PathBuf::from(path)
    } else {
        let cwd = std::env::current_dir()
            .and_then(|d| d.canonicalize().or(Ok(d)))
            .map_err(CheckError::CurrentDir)?;
        cwd.join(input)
    };

    // Canonicalize if possible (resolves symlinks and .. components).
    // Fall back to raw path — file might not exist yet (PreToolUse on Write).
    let abs_path = abs_path.canonicalize().unwrap_or(abs_path);

    let wt = worktrees
        .find_containing(&abs_path)
        .ok_or_else(|| CheckError::NotInWorktree(abs_path.clone()))?;

    let rel = abs_path
        .strip_prefix(&wt.path)
        .map_err(|_| CheckError::PathResolution(abs_path.clone()))?
        .to_string_lossy()
        .to_string();

    Ok((wt, rel))
}

// ============================================================================
// Active changes detection
// ============================================================================

/// Check if a file has uncommitted changes in a worktree.
///
/// Compares the file on disk against HEAD. Returns true if the file
/// differs from HEAD (modified, new, or deleted).
fn file_has_active_changes(worktree_path: &Path, file_path: &str) -> bool {
    let repo = match gix::open(worktree_path) {
        Ok(r) => r,
        Err(_) => return false,
    };

    let workdir = match repo.workdir() {
        Some(p) => p.to_path_buf(),
        None => return false,
    };

    let disk_path = workdir.join(file_path);
    let exists_on_disk = disk_path.exists();
    let head_blob = head_file_contents(&repo, file_path);

    match (head_blob, exists_on_disk) {
        (None, false) => false,   // Not tracked, not on disk
        (None, true) => true,     // New untracked file
        (Some(_), false) => true, // Deleted from disk
        (Some(head_data), true) => {
            match std::fs::read(&disk_path) {
                Ok(disk_data) => head_data != disk_data,
                // File exists but unreadable — conservatively assume changed
                Err(_) => true,
            }
        }
    }
}

/// Read a file's contents from HEAD in the given repository.
fn head_file_contents(repo: &gix::Repository, file_path: &str) -> Option<Vec<u8>> {
    let mut head = repo.head().ok()?;
    let head_id = head.try_peel_to_id().ok()??;
    let commit = repo.find_object(head_id).ok()?.try_into_commit().ok()?;
    let mut tree = commit.tree().ok()?;
    let entry = tree.peel_to_entry_by_path(file_path).ok()??;
    let blob = repo.find_object(entry.id()).ok()?;
    Some(blob.data.to_vec())
}
