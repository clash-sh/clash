//! Clash - Predict merge conflicts across git worktrees
//!
//! This library provides core functionality for detecting potential merge conflicts
//! between git worktrees before actually merging them. It's designed to support
//! parallel development workflows where multiple AI agents or developers work on
//! different branches simultaneously.
//!
//! # Architecture
//!
//! The library is organized into focused modules:
//!
//! - **worktree** - Worktree discovery and status tracking
//! - **conflict** - Conflict detection using git merge-tree analysis
//!
//! # Usage
//!
//! This library can be used in multiple ways:
//!
//! 1. **CLI tool** (`src/main.rs`) - Command-line interface for humans
//! 2. **MCP server** (planned) - JSON-RPC interface for AI agents
//! 3. **Library** - Rust code can import and use these functions directly
//!
//! # Example
//!
//! ```rust,no_run
//! use clash_sh::WorktreeManager;
//!
//! // Discover all worktrees in the current repository
//! let manager = WorktreeManager::discover().expect("Failed to discover worktrees");
//!
//! // Check all pairs for conflicts
//! let conflicts = manager.check_all_conflicts();
//!
//! // Process results
//! for conflict in conflicts {
//!     if !conflict.conflicting_files.is_empty() {
//!         println!("{} vs {}: {} conflicts",
//!             conflict.wt1.branch,
//!             conflict.wt2.branch,
//!             conflict.conflicting_files.len()
//!         );
//!     }
//! }
//! ```

pub mod worktree;

pub use worktree::*;
// Main lib updates
