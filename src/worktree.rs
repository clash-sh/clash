//! Worktree discovery and status tracking
//!
//! This module provides functionality for discovering git worktrees (both main and linked)
//! and tracking their status (clean, dirty, conflicted, etc.). It also includes
//! conflict detection using git merge-tree analysis.

mod conflict;
mod error;
mod manager;

pub use conflict::WorktreePairConflict;
pub use error::{Result as WorktreeResult, WorktreeError};
pub use manager::WorktreeManager;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Identifier for the main (primary) worktree
pub(crate) const MAIN_WORKTREE_ID: &str = "main";

/// Label for worktrees with detached HEAD (no branch)
pub(crate) const DETACHED_HEAD_LABEL: &str = "(detached HEAD)";

/// Label for worktrees with inaccessible paths
pub(crate) const INACCESSIBLE_PATH_LABEL: &str = "(inaccessible)";

/// A git worktree with its current state (simplified for MVP)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Worktree {
    /// Unique identifier (MAIN_WORKTREE_ID for main worktree, branch name for linked worktrees)
    pub id: String,

    /// Absolute filesystem path
    pub path: PathBuf,

    /// Branch name (e.g., "main", "feature/auth")
    pub branch: String,

    /// Working directory status
    pub status: WorktreeStatus,
}

// Worktree methods are extended in submodules:
// - conflict.rs: adds conflicts_with() and other conflict detection methods

/// Status of a git worktree
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorktreeStatus {
    /// No uncommitted changes
    Clean,

    /// Has uncommitted changes
    Dirty,

    /// Has unresolved merge conflicts
    Conflicted,

    /// Detached HEAD state
    Detached,

    /// Locked by another process
    Locked,
}

impl std::fmt::Display for WorktreeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            WorktreeStatus::Clean => write!(f, "clean"),
            WorktreeStatus::Dirty => write!(f, "dirty"),
            WorktreeStatus::Conflicted => write!(f, "conflicted"),
            WorktreeStatus::Detached => write!(f, "detached"),
            WorktreeStatus::Locked => write!(f, "locked"),
        }
    }
}
