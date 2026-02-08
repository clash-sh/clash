//! WorktreeManager - manages collection of worktrees in a git repository

use super::error::{Result, WorktreeError};
use super::{
    DETACHED_HEAD_LABEL, INACCESSIBLE_PATH_LABEL, MAIN_WORKTREE_ID, Worktree, WorktreeStatus,
};
use std::path::PathBuf;

/// Manager for all worktrees in a git repository
#[derive(Debug, Clone)]
pub struct WorktreeManager {
    items: Vec<Worktree>,
    repo_path: PathBuf,
}

impl WorktreeManager {
    /// Discover all worktrees in the current repository
    pub fn discover() -> Result<Self> {
        Self::discover_from(".")
    }

    /// Discover all worktrees in the repository at the given path
    pub fn discover_from(path: &str) -> Result<Self> {
        let path_buf = PathBuf::from(path);
        // Canonicalize path for better error messages
        let canonical_path = path_buf.canonicalize().unwrap_or_else(|_| path_buf.clone());

        let repo = gix::discover(path).map_err(|_| WorktreeError::NotARepository {
            path: canonical_path.clone(),
        })?;

        let mut items = Vec::new();

        // Add main worktree
        let main_path = repo
            .workdir()
            .and_then(|p| p.canonicalize().ok())
            .ok_or(WorktreeError::BareRepository)?;

        let main_branch = repo
            .head()
            .ok()
            .and_then(|head| head.referent_name().map(|n| n.shorten().to_string()))
            .unwrap_or_else(|| DETACHED_HEAD_LABEL.to_string());

        // Check dirty status with explicit error handling
        let main_status = match repo.is_dirty() {
            Ok(true) => WorktreeStatus::Dirty,
            Ok(false) => WorktreeStatus::Clean,
            Err(_) => {
                // If we can't determine dirty status (permissions, I/O error, etc),
                // default to Clean to avoid false warnings. User can manually check.
                WorktreeStatus::Clean
            }
        };

        items.push(Worktree {
            id: MAIN_WORKTREE_ID.to_string(),
            path: main_path.clone(),
            branch: main_branch,
            status: main_status,
        });

        // Add linked worktrees
        if let Ok(linked) = repo.worktrees() {
            for proxy in linked.iter() {
                let id = proxy.id().to_string();
                let path = proxy
                    .base()
                    .ok()
                    .unwrap_or_else(|| PathBuf::from(INACCESSIBLE_PATH_LABEL));

                let (branch, status) = proxy
                    .clone()
                    .into_repo_with_possibly_inaccessible_worktree()
                    .ok()
                    .map(|wt_repo| {
                        let branch = wt_repo
                            .head()
                            .ok()
                            .and_then(|head| head.referent_name().map(|n| n.shorten().to_string()))
                            .unwrap_or_else(|| DETACHED_HEAD_LABEL.to_string());

                        // Check dirty status with explicit error handling
                        let status = match wt_repo.is_dirty() {
                            Ok(true) => WorktreeStatus::Dirty,
                            Ok(false) => WorktreeStatus::Clean,
                            Err(_) => {
                                // If we can't determine dirty status, default to Clean
                                WorktreeStatus::Clean
                            }
                        };

                        (branch, status)
                    })
                    .unwrap_or_else(|| (DETACHED_HEAD_LABEL.to_string(), WorktreeStatus::Clean));

                items.push(Worktree {
                    id,
                    path,
                    branch,
                    status,
                });
            }
        }

        Ok(Self {
            items,
            repo_path: PathBuf::from(path),
        })
    }

    /// Refresh worktree information by re-discovering
    pub fn refresh(&mut self) -> Result<()> {
        // Convert path to string, handling non-UTF-8 paths
        let path_str = self
            .repo_path
            .to_str()
            .ok_or_else(|| WorktreeError::InvalidPath {
                path: self.repo_path.clone(),
            })?;

        *self = Self::discover_from(path_str)?;
        Ok(())
    }

    /// Get all worktrees
    pub fn all(&self) -> &[Worktree] {
        &self.items
    }

    /// Get the main worktree
    pub fn main(&self) -> Option<&Worktree> {
        self.items.iter().find(|w| w.id == MAIN_WORKTREE_ID)
    }

    /// Get the number of worktrees
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if there are no worktrees
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Iterate over all worktrees
    pub fn iter(&self) -> std::slice::Iter<'_, Worktree> {
        self.items.iter()
    }

    /// Find the worktree containing the given directory.
    ///
    /// Walks up from `dir` checking each directory against known worktree paths.
    /// This handles subdirectories and avoids ambiguity with nested worktrees.
    pub fn find_containing(&self, dir: &std::path::Path) -> Option<&Worktree> {
        let mut current = Some(dir);
        while let Some(d) = current {
            if let Some(wt) = self.items.iter().find(|wt| wt.path == d) {
                return Some(wt);
            }
            current = d.parent();
        }
        None
    }
}

// WorktreeManager methods are extended in other modules:
// - conflict.rs: adds check_all_conflicts(), check_conflicts_between(), etc.
