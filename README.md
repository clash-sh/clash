<p align="center">
  <img src="demos/logo.png" alt="Clash Logo" width="120"/>
</p>

<h1 align="center">Clash</h1>

<p align="center">
  <strong><em>Manage merge conflicts across git worktrees for parallel AI coding agents</em></strong>
</p>

Clash is an open-source CLI tool that helps manage/avoid merge conflicts across git worktrees.

It's designed for developers running multiple AI coding agents (Claude Code, Codex, etc.) in parallel, enabling them to identify potential conflicts early on in the development process and lets them/their coding agent work around potential conflicts.


[![Crates.io](https://img.shields.io/crates/v/clash-sh.svg)](https://crates.io/crates/clash-sh)
[![CI](https://github.com/clash-sh/clash/workflows/CI/badge.svg)](https://github.com/clash-sh/clash/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust: 1.93+](https://img.shields.io/badge/Rust-1.93+-orange.svg)](https://rust-lang.org)

## Problem Statement with Some Context

AI coding agents like Claude Code have transformed software development as we know it. Developers now can run 3-6+ agents simultaneously each working on a different feature/bug fix. The recommended approach put forward by
most frontier labs is to isolate each agent in its own git worktree, which is essentially a separate working directory with its own branch.

When multiple AI agents (or developers) work in separate git worktrees, they're blind to each other's changes & inevitably touch overlapping parts of the codebase. Conflicts only surface at feature completion, often after significant work/time is wasted.
Current tools only check for these conflicts only later down the line, we can have better results if we can catch potential conflicts earlier in the process.

Clash aims to be such a tool that helps with this and surface the **worktree-to-worktree conflicts** earlier in the process, so human/agent can be aware of overlapping changes and act on it, which will lead to better outcomes.

**Real quotes from developers:**

> "Now you have created the fun new ability to create merge conflicts with yourself."
> — GitButler Blog

> "I now know to expect potential conflicts. This happened to me recently, which was a big learning moment."
> — Developer on Medium

> "If conflicts look tricky, I throw away the work."
> — Developer workflow description

## Clash - The Saviour

Clash detects merge conflicts **between all worktree pairs** during development, helping you/agent:
- **See conflicts instantly** across all active worktrees
- **Visualize conflict matrix** showing which branches conflict
- **Monitor in real-time** as files change
- **Integrate with AI agents** 

### Clash in Action
![Clash alerts Claude Code to conflicts in another worktree, enabling smarter parallel development](https://clash.sh/demos/claude-using-clash-demo.gif)

*Clash alerts Claude Code to conflicts in another worktree, enabling smarter parallel development*

## Quick Start

### Installation

```bash
# Quick install (macOS/Linux)
curl -fsSL https://clash.sh/install.sh | sh

# Homebrew (macOS/Linux)
brew tap clash-sh/tap
brew install clash

# From crates.io
cargo install clash-sh

# From source (Rust 1.93+ required)
cargo install --path .
```

### Basic Usage
```For Humans/Agents - Add an instruction in Claude.md or ask it to ask use clash explicitly during your development. clash --help)```

```bash
# Check conflicts across all worktrees
clash status

# Output as JSON for scripts/agents
clash status --json

# Watch for conflicts in real-time
clash watch
```

![Quick demo showing clash status in action](https://clash.sh/demos/clash-status-demo-short.gif)

## Core Features

### Status Command
Shows a beautiful conflict matrix for all worktree pairs:

```
╔════════════ Conflict Matrix ════════════╗
║            │ main  feat/a  feat/b  feat/c║
╟────────────┼─────────────────────────────╢
║main        │  -     OK      OK      OK   ║
║feature/a   │  OK     -      2       1    ║
║feature/b   │  OK     2      -       3    ║
║feature/c   │  OK     1      3       -    ║
╚══════════════════════════════════════════╝
```

Numbers indicate conflicting files. "OK" means no conflicts.

### Watch Mode
Live TUI monitoring for humans to keep tab with automatic refresh on file changes:

```bash
# Shows worktree status and conflict updates in real-time
clash watch
```

![Watch mode detecting conflicts as files change in real-time](https://clash.sh/demos/clash-watch-realtime-demo.gif)

### JSON Output
Machine-readable output for CI/CD and AI agents:

```bash
clash status --json | jq '.conflicts[] | select(.conflicting_files | length > 0)'
```

```json
{
  "worktrees": [...],
  "conflicts": [
    {
      "wt1_id": "feature/auth",
      "wt2_id": "feature/api",
      "conflicting_files": ["src/models.rs", "api/schema.json"]
    }
  ]
}
```

![JSON output for AI agents, scripts, and automation pipelines](https://clash.sh/demos/clash-status-json-demo.gif)

## An Example Workflow with 2 instances. Co-ordinating with Clash

![Multiple AI agents working in parallel with Clash coordination](https://clash.sh/demos/multi-agent-clash-demo-v1.gif)

*Coordinating multiple AI agents across worktrees - each agent works independently while using Clash for monitoring for conflicts*

### For Claude Code / Cursor / Windsurf (Today)

Add to your project instructions or `.claude/instructions.md`:

```markdown
After each commit, run `clash status --json` to check for merge conflicts with other worktrees.
If conflicts are detected, examine the conflicting files in those worktrees and adapt your
approach to avoid or minimize conflicts. This prevents wasted work from incompatible changes.

Example workflow:
1. Make changes and commit
2. Run: clash status --json
3. If conflicts found: Review the conflicting worktree's changes
4. Adapt your implementation to work with their changes
```

### MCP Server (Coming Soon)

Once the MCP server is available, Clash will automatically appear in AI agent tool lists, enabling seamless conflict detection without manual configuration. Agents will be able to discover and use Clash just like any other MCP-enabled tool.

## How It Works

Clash uses `git merge-tree` (via the gix library) to perform three-way merges between worktree pairs **without modifying your repository**. It:

1. Discovers all worktrees (main + linked)
2. For each pair, finds the merge base
3. Simulates merge to detect conflicts
4. Reports conflicting files

This is **100% read-only** — your repository is never modified.

## Roadmap

### v0.1.0 (Current)
- ✅ Conflict detection across all worktrees
- ✅ Real-time watch mode
- ✅ JSON output for automation/agent use
- ✅ Beautiful conflict matrix display

### v0.2.0 (Next)
- [ ] MCP server for AI agent integration

### v0.3.0 (Coming Soon)


## Use Cases

### For AI Agent Orchestrators
Prevent wasted compute when agents' work conflicts:
```bash
# Check before expensive operations
if clash status --json | jq -e '.conflicts | length > 0'; then
  echo "Conflicts detected! Replan agent tasks"
fi
```

### For Human Teams
See what your teammates are changing before you merge:
```bash
# Add to git pre-merge hook
clash status || echo "Warning: Conflicts detected!"
```

### For CI/CD Pipelines
Fail fast when feature branches conflict:
```yaml
- name: Check for conflicts
  run: |
    cargo install clash
    clash status --json > conflicts.json
    jq -e '.conflicts | length == 0' conflicts.json
```

## Architecture

Clash is written in Rust for speed and reliability:
- **gix** - Git operations without shelling out
- **ratatui** - Terminal UI framework
- **notify** - File system watching
- **serde** - JSON serialization

Single binary, no runtime dependencies. Works anywhere git works.

## Contributing

We really appreciate all contributions big or small! 

Here are some ways to be part of clash:

⭐ Star the repo — helps others discover Clash
🗣️ Tell a friend about Clash
- 📝 [**Open an issue**](https://github.com/clash-sh/clash/issues/) — feedback, feature requests, even small friction points or confusing messages, or [AI agent coordination pain not yet solved](https://github.com/clash-sh/clash/issues/new?title=Agent%20coordination%20friction%3A%20&body=%23%23%20The%20friction%0A%0A%3C!--%20What%20agent%20coordination%20task%20is%20still%20painful%3F%20--%3E%0A%0A%23%23%20Current%20workaround%0A%0A%3C!--%20How%20do%20you%20handle%20this%20today%3F%20--%3E%0A%0A%23%23%20Ideal%20solution%0A%0A%3C!--%20What%20would%20make%20this%20easier%3F%20--%3E)
- 📢 **Share**: [X](https://twitter.com/intent/tweet?text=Clash%20%E2%80%94%20Manage%20merge%20conflicts%20across%20git%20worktrees%20for%20parallel%20AI%20coding%20agents&url=https%3A%2F%2Fgithub.com%2Fclash-sh%2Fclash) · [Reddit](https://www.reddit.com/submit?url=https%3A%2F%2Fgithub.com%2Fclash-sh%2Fclash&title=Clash%20%E2%80%94%20Manage%20merge%20conflicts%20across%20git%20worktrees%20for%20parallel%20AI%20coding%20agents) · [LinkedIn](https://www.linkedin.com/sharing/share-offsite/?url=https%3A%2F%2Fgithub.com%2Fclash-sh%2Fclash)

See [CONTRIBUTING.md](CONTRIBUTING.md) for some more details.

## FAQ

**Q: Does Clash modify my repository?**
A: Never. Clash is 100% read-only. It simulates merges in memory.

**Q: Can it resolve conflicts?**
A: Not yet. Clash detects conflicts. Resolution suggestions may be in a future version. But the underlying
agent should be able to act on the information

## Support

- 🐛 [Report bugs](https://github.com/clash-sh/clash/issues)
- 📧 [Email support](mailto:matk9@clash.sh)

## License

MIT - See [LICENSE](LICENSE) file for details.

## Acknowledgments

Built with Love, Rust & Claude.

Special thanks to the Rust community for incredible libraries that made this possible.