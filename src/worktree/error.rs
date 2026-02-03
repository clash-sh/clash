//! Error types for worktree operations
//!
//! This module defines custom error types using thiserror for better
//! error handling and propagation in the worktree module.

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during worktree operations
#[derive(Debug, Error)]
pub enum WorktreeError {
    /// Generic git error that we convert to string
    /// This handles various gix error types that don't have direct conversions
    #[error("Git operation failed: {0}")]
    GitOperation(String),

    /// Repository not found or invalid
    #[error("Not a git repository: {path}")]
    NotARepository { path: PathBuf },

    /// Failed to resolve HEAD reference
    #[error("Failed to resolve HEAD for branch '{branch}': {reason}")]
    HeadResolution { branch: String, reason: String },

    /// Failed to get tree from commit
    #[error("Failed to get tree for {label}: {reason}")]
    TreeResolution { label: String, reason: String },

    /// No commit found for branch
    #[error("No commit found for branch '{branch}'")]
    NoCommit { branch: String },

    /// Branch not found in worktrees
    #[error("Branch '{branch}' not found in worktrees")]
    BranchNotFound { branch: String },

    /// Invalid path provided
    #[error("Invalid path: {path}")]
    InvalidPath { path: PathBuf },

    /// Bare repository not supported
    #[error("Bare repositories are not supported. Clash requires a working directory to check for conflicts.")]
    BareRepository,

    /// Object is not a commit
    #[error("{label} object is not a commit")]
    NotACommit { label: String },

    /// Merge operation failed
    #[error("Merge operation failed: {0}")]
    MergeFailed(String),
}

/// Result type alias using WorktreeError
pub type Result<T> = std::result::Result<T, WorktreeError>;

// Implement conversions for gix error types that we don't handle explicitly
impl From<gix::repository::merge_base::Error> for WorktreeError {
    fn from(err: gix::repository::merge_base::Error) -> Self {
        WorktreeError::GitOperation(err.to_string())
    }
}

impl From<gix::repository::tree_merge_options::Error> for WorktreeError {
    fn from(err: gix::repository::tree_merge_options::Error) -> Self {
        WorktreeError::GitOperation(err.to_string())
    }
}

impl From<gix::object::find::existing::Error> for WorktreeError {
    fn from(err: gix::object::find::existing::Error) -> Self {
        WorktreeError::GitOperation(err.to_string())
    }
}
