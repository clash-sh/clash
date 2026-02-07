use clash_sh::{WorktreeManager, WorktreePairConflict, WorktreeStatus};
use colored::Colorize;
use serde::Serialize;
use std::collections::HashMap;

/// JSON output structure for status command
#[derive(Debug, Serialize)]
struct StatusOutput {
    worktrees: Vec<WorktreeInfo>,
    conflicts: Vec<ConflictInfo>,
}

/// Worktree information for JSON output (simplified from full Worktree struct)
#[derive(Debug, Serialize)]
struct WorktreeInfo {
    id: String,
    path: String,
    branch: String,
    status: WorktreeStatus,
}

/// Conflict information for JSON output (references worktrees by ID)
#[derive(Debug, Serialize)]
struct ConflictInfo {
    wt1_id: String,
    wt2_id: String,
    conflicting_files: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

/// Handles the display of status information for worktrees and conflicts
pub struct StatusDisplay<'a> {
    worktrees: &'a WorktreeManager,
}

impl<'a> StatusDisplay<'a> {
    /// Create a new StatusDisplay for the given worktrees
    pub fn new(worktrees: &'a WorktreeManager) -> Self {
        Self { worktrees }
    }

    /// Run the full status display (worktrees + conflicts)
    pub fn show(&self) {
        self.show_worktrees();
        self.show_conflicts();
    }

    /// Display all worktrees in a formatted list
    pub fn show_worktrees(&self) {
        println!("{}", "Worktrees:".bright_cyan().bold());
        for wt in self.worktrees.iter() {
            let status_colored = match wt.status {
                WorktreeStatus::Clean => "clean".green(),
                WorktreeStatus::Dirty => "dirty".yellow().bold(),
                WorktreeStatus::Conflicted => "conflicted".red().bold(),
                WorktreeStatus::Detached => "detached".bright_yellow(),
                WorktreeStatus::Locked => "locked".bright_red(),
            };
            let branch_colored = if wt.branch == "main" || wt.branch == "master" {
                wt.branch.bright_white().bold()
            } else {
                wt.branch.bright_magenta()
            };
            println!(
                "  {}: {} [{}] ({})",
                wt.id.bright_blue(),
                wt.path.display().to_string().white(),
                branch_colored,
                status_colored
            );
        }
    }

    /// Check for conflicts and display results
    pub fn show_conflicts(&self) {
        if self.worktrees.len() <= 1 {
            println!(
                "\nNo potential conflicts (only {} worktree)",
                self.worktrees.len()
            );
            return;
        }

        println!(
            "\n{}",
            format!(
                "Checking for conflicts between {} worktrees...",
                self.worktrees.len()
            )
            .bright_yellow()
            .italic()
        );

        let pair_results = self.worktrees.check_all_conflicts();
        let conflict_matrix = self.build_conflict_matrix(&pair_results);

        // Display as table
        self.display_conflict_table(&conflict_matrix);

        // Display detailed view
        self.display_detailed_conflicts(&pair_results, &conflict_matrix);

        // Display summary
        self.display_summary(&pair_results);
    }

    /// Build a conflict matrix from pair results
    fn build_conflict_matrix(
        &self,
        pair_results: &[WorktreePairConflict],
    ) -> Vec<Vec<Option<Vec<String>>>> {
        let mut matrix: Vec<Vec<Option<Vec<String>>>> =
            vec![vec![None; self.worktrees.len()]; self.worktrees.len()];

        // Map worktree IDs to indices (IDs are unique, branches might not be)
        let id_to_index: HashMap<String, usize> = self
            .worktrees
            .iter()
            .enumerate()
            .map(|(i, wt)| (wt.id.clone(), i))
            .collect();

        for result in pair_results {
            // Skip errored pairs — they stay as None ("?" in display)
            if result.error.is_some() {
                continue;
            }
            let i = id_to_index[&result.wt1.id];
            let j = id_to_index[&result.wt2.id];
            matrix[i][j] = Some(result.conflicting_files.clone());
            matrix[j][i] = Some(result.conflicting_files.clone());
        }

        matrix
    }

    /// Display conflicts as a table/matrix
    fn display_conflict_table(&self, conflict_matrix: &[Vec<Option<Vec<String>>>]) {
        // Calculate column widths dynamically
        let branch_width = self
            .worktrees
            .iter()
            .map(|w| w.branch.len())
            .max()
            .unwrap_or(10)
            .max(20);

        // Calculate column width based on abbreviated branch names
        // Add 2 chars padding to prevent truncation
        let col_width = self
            .worktrees
            .iter()
            .map(|w| {
                if w.branch.starts_with("feature/") {
                    // "f/" + rest of name + padding
                    2 + w.branch.len() - 8 + 2
                } else {
                    w.branch.len() + 2
                }
            })
            .max()
            .unwrap_or(12)
            .clamp(12, 25); // Minimum 12, maximum 25 for readability
        let total_width = branch_width + 2 + (col_width + 1) * self.worktrees.len();

        // Table header with bright cyan color
        println!(
            "\n{}",
            format!("╔{:═^width$}╗", " Conflict Matrix ", width = total_width)
                .bright_cyan()
                .bold()
        );

        // Column headers
        print!("{}", "║".bright_cyan());
        print!("{:width$} {}", "", "│".bright_cyan(), width = branch_width);
        for wt in self.worktrees.iter() {
            let truncated = Self::truncate_branch(&wt.branch, col_width);
            print!(
                " {:^width$}",
                truncated.bright_magenta().bold(),
                width = col_width
            );
        }
        println!("{}", "║".bright_cyan());

        // Separator
        print!(
            "{}",
            format!("╟{:─<width$}─┼", "", width = branch_width).bright_cyan()
        );
        for _ in 0..self.worktrees.len() {
            print!(
                "{}",
                format!("─{:─<width$}", "", width = col_width).bright_cyan()
            );
        }
        println!("{}", "╢".bright_cyan());

        // Rows
        for (i, wt) in self.worktrees.iter().enumerate() {
            print!("{}", "║".bright_cyan());
            let truncated_branch = Self::truncate_branch(&wt.branch, branch_width);
            print!(
                "{:width$} {}",
                truncated_branch.bright_magenta().bold(),
                "│".bright_cyan(),
                width = branch_width
            );
            for (j, cell) in conflict_matrix[i].iter().enumerate() {
                if i == j {
                    print!(" {:^width$}", "-".bright_black(), width = col_width);
                } else {
                    match cell {
                        Some(files) => {
                            let count = files.len();
                            if count == 0 {
                                print!(
                                    " {:^width$}",
                                    "OK".bright_green().bold(),
                                    width = col_width
                                );
                            } else {
                                let conflict_display = if count == 1 {
                                    count.to_string().bright_yellow().bold()
                                } else {
                                    count.to_string().bright_red().bold()
                                };
                                print!(" {:^width$}", conflict_display, width = col_width);
                            }
                        }
                        None => print!(" {:^width$}", "?".bright_black(), width = col_width),
                    }
                }
            }
            println!("{}", "║".bright_cyan());
        }

        // Bottom border
        println!(
            "{}",
            format!("╚{:═<width$}╝", "", width = total_width)
                .bright_cyan()
                .bold()
        );
    }

    /// Display detailed conflict information
    fn display_detailed_conflicts(
        &self,
        _pair_results: &[WorktreePairConflict],
        conflict_matrix: &[Vec<Option<Vec<String>>>],
    ) {
        println!("\n{}", "Detailed conflicts:".bright_cyan().bold());
        let worktree_list: Vec<_> = self.worktrees.iter().collect();

        for i in 0..self.worktrees.len() {
            for j in (i + 1)..self.worktrees.len() {
                let wt1 = worktree_list[i];
                let wt2 = worktree_list[j];

                print!(
                    "  {} {} {}: ",
                    wt1.branch.bright_magenta(),
                    "vs".white(),
                    wt2.branch.bright_magenta()
                );

                match &conflict_matrix[i][j] {
                    Some(files) if files.is_empty() => {
                        println!("{} {}", "✓".bright_green().bold(), "No conflicts".green());
                    }
                    Some(files) => {
                        let warn_text = format!(
                            "⚠ {} conflict{}",
                            files.len(),
                            if files.len() == 1 { "" } else { "s" }
                        );
                        if files.len() == 1 {
                            println!("{}", warn_text.bright_yellow().bold());
                        } else {
                            println!("{}", warn_text.bright_red().bold());
                        }
                        for file in files {
                            println!("    {} {}", "→".bright_red(), file.yellow());
                        }
                    }
                    None => {
                        println!("{}", "Error checking conflicts".red());
                    }
                }
            }
        }
    }

    /// Display summary statistics
    fn display_summary(&self, pair_results: &[WorktreePairConflict]) {
        if pair_results.is_empty() {
            return;
        }

        let error_count = pair_results.iter().filter(|r| r.error.is_some()).count();
        let checked_pairs = pair_results.len() - error_count;
        let total_conflicts: usize = pair_results
            .iter()
            .filter(|r| r.error.is_none())
            .map(|r| r.conflicting_files.len())
            .sum();

        print!("\n{}: ", "Summary".bright_cyan().bold());
        print!(
            "Checked {} pair{}, ",
            checked_pairs.to_string().bright_blue().bold(),
            if checked_pairs == 1 { "" } else { "s" }
        );

        if total_conflicts == 0 {
            println!("found {} conflicts ✅", "no".bright_green().bold());
        } else {
            let conflict_color = if total_conflicts == 1 {
                total_conflicts.to_string().bright_yellow().bold()
            } else {
                total_conflicts.to_string().bright_red().bold()
            };
            println!(
                "found {} total conflict{}",
                conflict_color,
                if total_conflicts == 1 { "" } else { "s" }
            );
        }

        if error_count > 0 {
            println!(
                "  {} {} failed to check",
                error_count.to_string().bright_red().bold(),
                if error_count == 1 { "pair" } else { "pairs" }
            );
        }
    }

    /// Truncate branch name to fit in column
    fn truncate_branch(branch: &str, max_len: usize) -> String {
        if branch.len() <= max_len {
            return branch.to_string();
        }

        if max_len <= 2 {
            return "...".to_string();
        }

        // Try smart truncation for feature branches
        if branch.starts_with("feature/") && max_len > 4 {
            let suffix = &branch[8..];
            let abbreviated = format!("f/{}", suffix);
            if abbreviated.len() <= max_len {
                return abbreviated;
            }
            let suffix_max = max_len.saturating_sub(3);
            if suffix_max > 0 && suffix.len() > suffix_max {
                return format!("f/{}...", &suffix[..suffix_max.min(suffix.len() - 1)]);
            }
        }

        // For other branches, show the end
        if branch.contains('/')
            && max_len > 6
            && let Some(pos) = branch.rfind('/')
        {
            let suffix = &branch[pos + 1..];
            if suffix.len() < max_len - 1 {
                return format!(".../{}", suffix);
            }
        }

        // Default truncation
        format!("{}...", &branch[..max_len - 3])
    }
}

/// Run the status command - displays worktrees and checks for conflicts
pub fn run_status(worktrees: &WorktreeManager, json: bool) {
    if json {
        // Build JSON output
        let worktree_infos: Vec<WorktreeInfo> = worktrees
            .iter()
            .map(|wt| WorktreeInfo {
                id: wt.id.clone(),
                path: wt.path.display().to_string(),
                branch: wt.branch.clone(),
                status: wt.status,
            })
            .collect();

        // Check conflicts and convert to minimal format
        let conflicts: Vec<ConflictInfo> = worktrees
            .check_all_conflicts()
            .into_iter()
            .filter(|c| !c.conflicting_files.is_empty() || c.error.is_some())
            .map(|c| ConflictInfo {
                wt1_id: c.wt1.id,
                wt2_id: c.wt2.id,
                conflicting_files: c.conflicting_files,
                error: c.error,
            })
            .collect();

        let output = StatusOutput {
            worktrees: worktree_infos,
            conflicts,
        };

        // Output JSON
        match serde_json::to_string_pretty(&output) {
            Ok(json_str) => println!("{}", json_str),
            Err(e) => eprintln!("Error serializing to JSON: {}", e),
        }
    } else {
        // Human-readable output
        let display = StatusDisplay::new(worktrees);
        display.show();
    }
}
