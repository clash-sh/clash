//! Conflict detection methods for Worktree and WorktreeManager
//!
//! This module extends Worktree and WorktreeManager with conflict detection
//! capabilities using git merge-tree analysis, following the pattern of
//! splitting impl blocks across files by functionality.

use super::error::{Result, WorktreeError};
use super::{Worktree, WorktreeManager};
use serde::{Deserialize, Serialize};

/// Result of checking a pair of worktrees for conflicts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreePairConflict {
    pub wt1: Worktree,
    pub wt2: Worktree,
    pub conflicting_files: Vec<String>,
    /// Error message if conflict detection failed for this pair.
    /// When set, `conflicting_files` is empty and should not be trusted.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub error: Option<String>,
}

// ============================================================================
// Worktree conflict methods
// ============================================================================

impl Worktree {
    /// Check for conflicts between this worktree and another
    pub fn conflicts_with(&self, other: &Worktree) -> Result<Vec<String>> {
        // Open repository for the first worktree
        let repo1 = gix::open(&self.path).map_err(|_| WorktreeError::NotARepository {
            path: self.path.clone(),
        })?;

        // Open repository for the second worktree (or use the same if both are in same repo)
        let repo2 = gix::open(&other.path).map_err(|_| WorktreeError::NotARepository {
            path: other.path.clone(),
        })?;

        // Get the HEAD commits for each worktree
        let head1 = get_head_commit(&repo1, &self.branch)?;
        let head2 = get_head_commit(&repo2, &other.branch)?;

        // Find merge base between the two commits
        let base_id = repo1.merge_base(head1, head2)?;

        // Get tree IDs for merge (not the tree objects themselves)
        let base_tree_id = get_tree_id(&repo1, base_id, "base")?;
        let tree1_id = get_tree_id(&repo1, head1, &self.branch)?;
        let tree2_id = get_tree_id(&repo2, head2, &other.branch)?;

        // Create labels for the merge
        let labels = gix::merge::blob::builtin_driver::text::Labels {
            ancestor: Some("base".into()),
            current: Some(self.branch.as_str().into()),
            other: Some(other.branch.as_str().into()),
        };

        // Get merge options
        let options = repo1.tree_merge_options()?;

        // Perform the merge to detect conflicts
        let merge_outcome = repo1
            .merge_trees(base_tree_id, tree1_id, tree2_id, labels, options)
            .map_err(|e| WorktreeError::MergeFailed(e.to_string()))?;

        // Extract conflicting file paths
        let conflicting_files: Vec<String> = merge_outcome
            .conflicts
            .into_iter()
            .map(|conflict| conflict.ours.location().to_string())
            .collect();

        Ok(conflicting_files)
    }
}

// ============================================================================
// WorktreeManager conflict methods
// ============================================================================

impl WorktreeManager {
    /// Check for conflicts between all worktree pairs
    pub fn check_all_conflicts(&self) -> Vec<WorktreePairConflict> {
        let mut results = Vec::new();
        let all = self.all();

        for i in 0..all.len() {
            for j in (i + 1)..all.len() {
                let (files, error) = match all[i].conflicts_with(&all[j]) {
                    Ok(files) => (files, None),
                    Err(e) => (Vec::new(), Some(e.to_string())),
                };

                results.push(WorktreePairConflict {
                    wt1: all[i].clone(),
                    wt2: all[j].clone(),
                    conflicting_files: files,
                    error,
                });
            }
        }
        results
    }
}

// ============================================================================
// Helper functions (private to this module)
// ============================================================================

/// Get HEAD commit ID for a worktree
fn get_head_commit<'a>(repo: &'a gix::Repository, branch_name: &str) -> Result<gix::Id<'a>> {
    let mut head = repo.head().map_err(|e| WorktreeError::HeadResolution {
        branch: branch_name.to_string(),
        reason: e.to_string(),
    })?;

    let id = head
        .try_peel_to_id()
        .map_err(|e| WorktreeError::HeadResolution {
            branch: branch_name.to_string(),
            reason: format!("Failed to peel HEAD: {}", e),
        })?;

    id.ok_or_else(|| WorktreeError::NoCommit {
        branch: branch_name.to_string(),
    })
}

/// Get tree ID from a commit ID
fn get_tree_id<'a>(
    repo: &'a gix::Repository,
    commit_id: gix::Id<'a>,
    label: &str,
) -> Result<gix::Id<'a>> {
    let object = repo.find_object(commit_id)?;

    let commit = object
        .try_into_commit()
        .map_err(|_| WorktreeError::NotACommit {
            label: label.to_string(),
        })?;

    commit.tree_id().map_err(|e| WorktreeError::TreeResolution {
        label: label.to_string(),
        reason: e.to_string(),
    })
}
