use clap::{Parser, Subcommand};
use clash_sh::WorktreeManager;
use colored::control;

mod check;
mod status;
mod watch;

#[derive(Parser)]
#[command(name = "clash")]
#[command(version)]
#[command(about = "Manage merge conflicts across git worktrees for parallel AI coding agents")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Show potential conflicts between worktrees (supports --json for AI agents)
    Status {
        #[arg(long, help = "Output results as JSON")]
        json: bool,
    },
    /// Watch for conflicts in real-time with interactive TUI
    Watch {
        // TODO: Add --debounce flag for handling rapid file changes
        // When many files change quickly (e.g., during git rebase),
        // we should wait for changes to settle before rechecking conflicts,
        // currently hardcoded to 1s
    },
    /// Check a single file for conflicts and active work across worktrees (JSON output)
    Check {
        /// File path to check (reads from hook stdin if omitted)
        path: Option<String>,
    },
}

fn main() {
    // Force colors to always be enabled regardless of terminal capabilities
    // TODO: Make color behavior configurable via --color flag (always/auto/never)
    control::set_override(true);

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Status { json }) => match WorktreeManager::discover() {
            Ok(worktrees) => {
                status::run_status(&worktrees, json);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
        Some(Commands::Watch {}) => match WorktreeManager::discover() {
            Ok(worktrees) => {
                if let Err(e) = watch::run_watch_mode(worktrees) {
                    eprintln!("Error running watch mode: {}", e);
                    std::process::exit(1);
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
        Some(Commands::Check { path }) => {
            if let Some(ref p) = path {
                // Manual mode: discover from cwd, exit 2 on conflicts
                match WorktreeManager::discover() {
                    Ok(worktrees) => match check::run_check(&worktrees, Some(p)) {
                        Ok(true) => std::process::exit(2),
                        Ok(false) => {}
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            std::process::exit(1);
                        }
                    },
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                // Hook mode: discover from file path in stdin
                match check::run_check_from_hook() {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
        None => {
            println!("Clash v{}", env!("CARGO_PKG_VERSION"));
            println!("Try 'clash --help' for more information.");
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn sanity_check() {
        assert_eq!(1 + 1, 2);
    }
}
