# CLAUDE.md - Clash Project

## Project Overview

Clash is an open-source CLI tool that helps manage/avoid merge conflicts across git worktrees.

It's designed for developers running multiple AI coding agents (Claude Code, Codex, etc.) in parallel, enabling them to identify potential conflicts early on in the development process and lets them/their coding agent work around potential conflicts.

**Repository:** github.com/clash-sh/clash  
**Domain:** clash.sh  
**License:** MIT

## Project Status

**This is a new project. Prioritize clean design over compatibility.**

We are in **greenfield** mode:
- No backward compatibility concerns yet
- Establish good patterns from day one
- Refactor freely until v1.0

When making decisions, prioritize:
1. **Best technical solution** over quick fixes
2. **Clean design** over "good enough"
3. **Modern Rust conventions** over legacy patterns

After v1.0, we'll shift to protecting external interfaces (CLI flags, config format, JSON output schema).

## Terminology

Use consistent terminology in documentation, help text, and code:

- **main worktree** — the original git directory (from clone/init); bare repos have none
- **linked worktree** — worktree created via `git worktree add` (git's official term)
- **conflict** — overlapping changes to the same file region across worktrees
- **potential conflict** — predicted conflict (not yet merged)
- **affected files** — files with changes in multiple worktrees
- **merge target** — the branch being merged into (e.g., main)

Don't say "clash" to mean conflict — it's our tool name. Say "conflict" or "potential conflict".

## Skills

Check `.claude/skills/` for specialized guidance:

- **`conflict-detection`** — Algorithm details for merge-tree analysis
- **`mcp-protocol`** — JSON-RPC message formats for MCP server
- **`output-formatting`** — Human-readable vs JSON output patterns

Load relevant skills before implementing features in those areas.

## Technical Spec

Before implementing any feature, read the relevant section of `docs/technical-plan.md`. It contains:
- Data structures (Section 3)
- Algorithms (Section 4)
- CLI interface (Section 5)
- MCP server spec (Section 6)

## Rust Guidelines

### Always Look Up Documentation

Before using any crate or standard library feature, check the docs:

```bash
# Open docs for a crate
cargo doc --open

# Or visit directly
# - https://docs.rs/gix
# - https://docs.rs/clap
# - https://doc.rust-lang.org/std/
```

### Don't Give Up on API Errors - Consult Documentation

When encountering compilation errors with external APIs:
1. **Don't assume it's "too complex"** - Often it just needs the right types
2. **Check the actual documentation** - Parameter types might be different than expected
3. **Look at type signatures carefully** - e.g., gix `merge_trees()` wants tree IDs, not Tree objects
4. **Persist through initial errors** - First attempt rarely works, that's normal

Example from our gix merge implementation:
```rust
// WRONG: Passing Tree objects
let merge_outcome = repo.merge_trees(base_tree, tree1, tree2, ...)?;

// RIGHT: Pass tree IDs (ObjectId), proper Labels type, correct Options
let labels = gix::merge::blob::builtin_driver::text::Labels {
    ancestor: Some("base".into()),
    current: Some(wt1.branch.as_str().into()),  // Note: "current" not "ours"
    other: Some(wt2.branch.as_str().into()),    // Note: "other" not "theirs"
};
let merge_outcome = repo.merge_trees(base_tree_id, tree1_id, tree2_id, labels, options)?;
```

### Edition and Version

- **Edition:** 2024
- **rust-version:** 1.93.0 (MSRV)
- Use modern Rust features (async traits, let chains, etc.)

### Code Style

```bash
# Always run before committing
cargo fmt
cargo clippy -- -D warnings
```

- Use `thiserror` for library errors, `anyhow` for application errors
- Use `&str` for function parameters (accepts both `String` and `&str`)
- Use `String` in struct fields (owned data)
- Derive `Debug, Clone` on all types (makes debugging and testing easier)
- Derive `Serialize, Deserialize` on types that need JSON output

### Error Handling

Use `anyhow` for error propagation with context:

```rust
use anyhow::{bail, Context, Result};

// Add context to I/O and external command failures
let content = std::fs::read_to_string(path)
    .context("Failed to read config file")?;

// Use bail! for business logic errors
if worktrees.is_empty() {
    bail!("no linked worktrees found");
}

// Custom errors for library code
#[derive(Debug, thiserror::Error)]
pub enum WorktreeError {
    #[error("Not a git repository: {0}")]
    NotARepository(PathBuf),
    
    #[error("Git error: {0}")]
    Git(#[from] gix::Error),
}
```

**Patterns:**
- **Use `bail!`** for business logic errors (no worktrees, invalid state)
- **Use `.context()`** for wrapping I/O and external failures
- **Let errors propagate** — don't catch and re-raise without adding info
- **No `.unwrap()` in library code** — use `?` or handle explicitly

### Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

- All tests go in the `tests/` directory (both unit and integration tests)
- Use `insta` for snapshot testing CLI output
- Keep tests focused: unit tests test logic in isolation, integration tests test end-to-end workflows

## Key Dependencies

| Crate | Purpose | Docs |
|-------|---------|------|
| `gix` | Git operations | https://docs.rs/gix |
| `clap` | CLI parsing | https://docs.rs/clap |
| `serde` | Serialization | https://docs.rs/serde |
| `serde_json` | JSON for MCP | https://docs.rs/serde_json |
| `anyhow` | App error handling | https://docs.rs/anyhow |
| `thiserror` | Library errors | https://docs.rs/thiserror |
| `ratatui` | TUI (Phase 2) | https://docs.rs/ratatui |
| `crossterm` | Terminal I/O | https://docs.rs/crossterm |

**Not using:**
- `tokio` — Clash is synchronous, no async needed
- `axum` — MCP uses stdio transport, no HTTP server needed

### Gix-Specific Patterns

When using gix for git operations:

1. **Enable required features**: `gix = { version = "0.78", features = ["merge"] }`

2. **Merge conflict detection pattern**:
```rust
// 1. Get HEAD commits from each worktree
let head1 = repo1.head()?.try_peel_to_id()?.ok_or("No commit")?;

// 2. Find merge base
let base_id = repo1.merge_base(head1, head2)?;

// 3. Get tree IDs (not tree objects!)
let tree1_id = commit1.tree_id()?;

// 4. Create labels for merge markers
let labels = gix::merge::blob::builtin_driver::text::Labels {
    ancestor: Some("base".into()),
    current: Some(branch1.into()),
    other: Some(branch2.into()),
};

// 5. Get options and perform merge
let options = repo.tree_merge_options()?;
let outcome = repo.merge_trees(base_tree_id, tree1_id, tree2_id, labels, options)?;

// 6. Check for conflicts
let has_conflicts = !outcome.conflicts.is_empty();
```

3. **Common gotchas**:
   - `merge_trees()` needs ObjectId/tree IDs, not Tree objects
   - Labels struct is in `blob::builtin_driver::text`, not in `tree` module
   - Use `try_peel_to_id()` not deprecated `try_peel_to_id_in_place()`
   - Worktree repositories need separate `gix::open()` calls

## Architecture Decisions

### Already Decided (Don't Change)

1. **Language:** Rust (not Go, not TypeScript)
2. **Git library:** `gix` (not `git2`)
3. **CLI framework:** `clap` with derive macros
4. **Serialization:** `serde` + `serde_json`
5. **MCP transport:** stdio only (no HTTP server)
6. **Execution model:** Synchronous (no async runtime)

### Design Principles

1. **Non-invasive:** Never modify git state, read-only operations
2. **Fast:** Status checks < 2 seconds for typical repos
3. **Composable:** JSON output for scripting (`--json` flag on all commands)
4. **Complement, don't compete:** Integrate with other developer tools

### Development Principles

1. **Incremental Development:** Add things only as we need them
2. **Smallest Steps:** Always work in the smallest increments possible
3. **No Jumping Ahead:** Don't implement features before they're needed
4. **YAGNI (You Aren't Gonna Need It):** Resist the urge to add "nice to have" features
5. **Iterative Refinement:** Get basics working first, then improve

## Project Structure

```
src/
├── main.rs           # Entry point
├── lib.rs            # Library root
├── cli/              # CLI commands (clap)
├── core/             # Business logic
├── git/              # Git operations (gix)
├── mcp/              # MCP server (stdio)
├── tui/              # Terminal UI (ratatui)
└── output/           # Formatters (human, json)
```

## Common Tasks

### Adding a New CLI Command

1. Create `src/cli/newcommand.rs`
2. Add subcommand to `src/cli/args.rs`
3. Implement handler in `src/cli/mod.rs`
4. Add tests
5. Update README

### Adding a New Output Format

All commands should support `--json`. Use the pattern:

```rust
if args.json {
    println!("{}", serde_json::to_string_pretty(&result)?);
} else {
    print_human_readable(&result);
}
```

## Using Clash During Development

**IMPORTANT: When working in parallel with other AI agents or developers in separate worktrees, always check for conflicts before and after making changes.**

After each commit, run `clash status --json` to check for merge conflicts with other worktrees. If conflicts are detected, examine the conflicting files in those worktrees and adapt your approach to avoid or minimize conflicts. This prevents wasted work from incompatible changes.

Example workflow:
1. Make changes and commit
2. Run: `clash status --json`
3. If conflicts found: Review the conflicting worktree's changes
4. Adapt your implementation to work with their changes

When working on Clash itself or using multiple worktrees, use Clash to check for conflicts:

### Check for Conflicts (Human-Readable)

```bash
clash status
```

Shows a colorful conflict matrix with:
- **Green "OK"** = No conflicts between worktrees
- **Yellow numbers** = 1-3 conflicts (warning)
- **Red numbers** = 4+ conflicts (danger)

Output includes:
- List of all worktrees with their status (clean/dirty)
- Conflict matrix showing conflicts between all worktree pairs
- Detailed list of conflicting files
- Summary of total conflicts

### Check for Conflicts (JSON)

```bash
clash status --json
```

Returns structured JSON for scripting and automation:

```json
{
  "worktrees": [
    {
      "id": "main",
      "path": "/Users/user/clash",
      "branch": "main",
      "status": "dirty"
    },
    {
      "id": "clash-predict",
      "path": "/Users/user/clash-predict",
      "branch": "feature/clash-predict",
      "status": "clean"
    }
  ],
  "conflicts": [
    {
      "wt1_id": "main",
      "wt2_id": "clash-predict",
      "conflicting_files": [
        "src/lib.rs",
        "src/status.rs"
      ]
    }
  ]
}
```

**Use cases:**
- CI/CD pipelines: Parse JSON to fail builds on conflicts
- Scripts: Automate conflict checking before merging
- Integration: Feed conflict data to other tools

### Real-Time Monitoring

```bash
clash watch
```

Launches an interactive TUI that:
- Monitors all worktrees for file changes
- Updates conflict status in real-time
- Shows which files are being modified
- Highlights new conflicts as they appear

Press `q` to exit watch mode.

### When to Use Clash

- **Before committing:** Check if your changes conflict with other worktrees
- **Parallel development:** When multiple AI agents or developers work simultaneously
- **CI/CD:** Automate conflict detection in your build pipeline
- **Code review:** Verify that feature branches don't conflict with each other

### Example Workflow

```bash
# Check current conflicts
clash status

# If conflicts exist, see which files
clash status --json | jq '.conflicts[].conflicting_files[]'

# Monitor in real-time while working
clash watch
```

## Common Pitfalls

### Ownership Issues

If fighting the borrow checker, consider:

```rust
// Quick fix: clone
let name = worktree.branch.clone();

// Better: take reference
fn process(branch: &str) { ... }

// Best: understand lifetime
fn process<'a>(worktree: &'a Worktree) -> &'a str {
    &worktree.branch
}
```

### String Types

```rust
// Function parameters: use &str (accepts both String and &str)
fn check_branch(name: &str) -> bool { ... }

// Struct fields: use String (owned)
struct Worktree {
    branch: String,
}

// Creating a struct: use .to_string() or .into()
let wt = Worktree { 
    branch: name.to_string() 
};
```

Simple rule: `&str` for parameters, `String` in structs.

### Execution Model

Clash is fully synchronous:
- CLI commands run sequentially
- MCP stdio reads request → processes → writes response
- TUI uses crossterm's sync event loop
- No async runtime, no `await`, no `tokio`

This keeps the codebase simple.

## Environment Setup

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install tools
rustup component add clippy rustfmt
cargo install cargo-watch cargo-insta

# Development workflow
cargo watch -x check -x test  # Auto-rebuild on changes
```

## Release Process

Uses `cargo-dist` for releases:

```bash
git tag "v0.1.0"
git push --tags
# GitHub Actions handles the rest
```

## Questions to Ask

If unclear about implementation:

1. Is this covered in `docs/technical-plan.md`?
2. What does the gix/clap documentation say?
3. Is there a Rust idiom for this pattern?

## Data Safety

Clash is a **read-only tool**. Never modify git state.

- **No writes ever** — Don't stage, commit, merge, rebase, or modify refs
- **No file modifications** — Don't write to working directory files
- **Fail-safe** — If unsure whether an operation is read-only, don't do it
- **Verify gix calls** — Some `gix` methods can write; check docs before using

This is non-negotiable. A user trusting Clash with their repo must never lose data.

## Command Execution

### Prefer gix Over Shell Commands

Always use `gix` for git operations, not shell commands:

```rust
// Good: gix API
let repo = gix::open(path)?;
let worktrees = repo.worktrees()?;

// Avoid: shell commands
let output = Command::new("git").args(["worktree", "list"]).output()?;
```

Exception: `git merge-tree` may need CLI fallback if gix doesn't support it.

### When Shell Commands Are Necessary

If you must shell out to git, use structured output:

| Operation | Avoid | Use |
|-----------|-------|-----|
| File status | `git status` | `git status --porcelain=v2` |
| Diff stats | `git diff --shortstat` | `git diff --numstat` |
| Merge-base | parsing stderr | exit codes (0=found, 1=none, 128=error) |

Never parse human-readable error messages — they break on locale changes and git updates.

## Accessor Function Naming

Function prefixes signal behavior:

| Prefix | Returns | Side Effects | Example |
|--------|---------|--------------|---------|
| (bare noun) | `Option<T>` or `T` | None | `branch()`, `worktrees()` |
| `require_*` | `Result<T>` | None, errors if absent | `require_branch()` |
| `find_*` | `Option<T>` | None | `find_worktree(name)` |
| `load_*` | `Result<T>` | File I/O | `load_config()` |
| `compute_*` | `Result<T>` | CPU-intensive | `compute_conflicts()` |

Don't use `get_*` — use bare nouns (Rust convention).

## Do NOT

- Modify git repository state (read-only tool)
- Use `unsafe` without explicit approval
- Add dependencies without justification
- Skip error handling (no `.unwrap()` in library code)
- Ignore clippy warnings
- Suppress warnings with `#[allow(dead_code)]` — delete unused code or add TODO
- Use `#[cfg(test)]` to add test-only methods to library code — put helpers in test modules
