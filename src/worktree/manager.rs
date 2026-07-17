//! WorktreeManager - manages collection of worktrees in a git repository

use super::error::{Result, WorktreeError};
use super::{
    DETACHED_HEAD_LABEL, INACCESSIBLE_PATH_LABEL, MAIN_WORKTREE_ID, Worktree, WorktreeStatus,
};
use std::path::{Path, PathBuf};

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

    /// Discover all worktrees from any path inside a repository.
    ///
    /// Accepts a file path, directory path, or `.` for cwd. Resolves
    /// relative paths against cwd, then uses `gix::discover` to walk
    /// up and find the containing git repository.
    pub fn discover_from(path: &str) -> Result<Self> {
        let input = PathBuf::from(path);
        let abs_path = if input.is_absolute() {
            input
        } else {
            std::env::current_dir()
                .and_then(|d| d.canonicalize().or(Ok(d)))
                .unwrap_or_else(|_| input.clone())
                .join(&input)
        };

        // gix::discover expects a directory; if the path isn't a directory
        // (file or non-existent path for new files), use its parent
        let discover_path = if abs_path.is_dir() {
            abs_path.clone()
        } else {
            abs_path.parent().unwrap_or(&abs_path).to_path_buf()
        };

        let repo = gix::discover(&discover_path).map_err(|_| WorktreeError::NotARepository {
            path: abs_path.clone(),
        })?;

        let mut items = Vec::new();

        // Find the real main worktree via the common .git directory.
        // repo.workdir() returns the cwd worktree (wrong if discovered from
        // a linked worktree). The common_dir is the shared .git/ directory
        // whose parent is always the main worktree root.
        //
        // gix may return common_dir with relative components
        // (e.g. `.git/worktrees/name/../..`), so canonicalize first.
        let common_dir = repo
            .common_dir()
            .canonicalize()
            .unwrap_or_else(|_| repo.common_dir().to_path_buf());
        let main_path = common_dir
            .parent()
            .and_then(|p| p.canonicalize().ok())
            .ok_or(WorktreeError::BareRepository)?;

        // Open the main worktree explicitly to get its branch and status,
        // since `repo` may point to a linked worktree.
        let main_repo = gix::open(&main_path).map_err(|_| WorktreeError::NotARepository {
            path: main_path.clone(),
        })?;

        let main_branch = main_repo
            .head()
            .ok()
            .and_then(|head| head.referent_name().map(|n| n.shorten().to_string()))
            .unwrap_or_else(|| DETACHED_HEAD_LABEL.to_string());

        let main_status = match main_repo.is_dirty() {
            Ok(true) => WorktreeStatus::Dirty,
            Ok(false) => WorktreeStatus::Clean,
            Err(_) => WorktreeStatus::Clean,
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
    pub fn find_containing(&self, dir: &Path) -> Option<&Worktree> {
        let mut current = Some(dir);
        while let Some(d) = current {
            if let Some(wt) = self.items.iter().find(|wt| paths_equivalent(&wt.path, d)) {
                return Some(wt);
            }
            current = d.parent();
        }
        None
    }

    /// Return `path` relative to `worktree`, using the same normalization as
    /// worktree lookup.
    pub fn repo_relative_path(&self, path: &Path, worktree: &Worktree) -> Option<String> {
        path_relative_to(path, &worktree.path)
    }
}

// WorktreeManager methods are extended in other modules:
// - conflict.rs: adds check_all_conflicts(), check_conflicts_between(), etc.

fn paths_equivalent(left: &Path, right: &Path) -> bool {
    comparable_path(left) == comparable_path(right)
}

fn path_relative_to(path: &Path, root: &Path) -> Option<String> {
    if let Ok(relative) = path.strip_prefix(root) {
        return Some(path_to_string(relative));
    }

    let path_normalized = normalized_path(path);
    let root_normalized = normalized_path(root);
    let path_key = comparable_path_from_normalized(&path_normalized);
    let root_key = comparable_path_from_normalized(&root_normalized);

    if path_key == root_key {
        return Some(String::new());
    }

    let root_with_separator = if root_key.ends_with('/') {
        root_key
    } else {
        format!("{}/", root_key)
    };

    path_key.strip_prefix(&root_with_separator)?;

    let relative_start = if root_normalized.ends_with('/') {
        root_normalized.len()
    } else {
        root_normalized.len() + 1
    };

    path_normalized
        .get(relative_start..)
        .map(ToString::to_string)
}

fn comparable_path(path: &Path) -> String {
    comparable_path_from_normalized(&normalized_path(path))
}

fn normalized_path(path: &Path) -> String {
    let mut path = path_to_string(path);

    if let Some(stripped) = path.strip_prefix("//?/UNC/") {
        path = format!("//{}", stripped);
    } else if let Some(stripped) = path.strip_prefix("//?/") {
        path = stripped.to_string();
    }

    trim_trailing_separators(&mut path);
    path
}

fn comparable_path_from_normalized(path: &str) -> String {
    let mut path = path.to_string();
    if is_windows_path(&path) {
        path.make_ascii_lowercase();
    }

    path
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn trim_trailing_separators(path: &mut String) {
    while path.len() > 1 && path.ends_with('/') && !is_windows_drive_root(path) {
        path.pop();
    }
}

fn is_windows_drive_root(path: &str) -> bool {
    let bytes = path.as_bytes();
    bytes.len() == 3 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' && bytes[2] == b'/'
}

fn is_windows_path(path: &str) -> bool {
    let bytes = path.as_bytes();
    (bytes.len() >= 3 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' && bytes[2] == b'/')
        || path.starts_with("//")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_reported_path_shapes_do_not_match_without_normalization() {
        assert_ne!(
            Path::new(r"\\?\C:\Repo\Linked-A"),
            Path::new("C:/repo/linked-a"),
        );
    }

    #[test]
    fn windows_extended_prefix_and_slashes_match() {
        assert!(paths_equivalent(
            Path::new(r"\\?\C:\Repo\Linked-A"),
            Path::new("C:/repo/linked-a"),
        ));
    }

    #[test]
    fn normalized_relative_path_handles_windows_prefix_mismatch() {
        let relative = path_relative_to(
            Path::new(r"\\?\C:\Repo\Linked-A\Src\Lib.rs"),
            Path::new("C:/repo/linked-a"),
        );

        assert_eq!(relative.as_deref(), Some("Src/Lib.rs"));
    }

    #[test]
    fn normalized_relative_path_handles_windows_prefix_mismatch_reversed() {
        let relative = path_relative_to(
            Path::new("C:/Repo/Linked-A/src/lib.rs"),
            Path::new(r"\\?\C:\repo\linked-a"),
        );

        assert_eq!(relative.as_deref(), Some("src/lib.rs"));
    }

    #[test]
    fn unix_paths_remain_case_sensitive() {
        assert!(!paths_equivalent(
            Path::new("/tmp/Repo"),
            Path::new("/tmp/repo"),
        ));
    }
}
