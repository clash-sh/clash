# Multi-agent workflow with Claude Code using Clash
# Shows 2 AI agents working on parallel features with Clash coordination

Output ../output/multi-agent-clash-demo.gif

Set FontSize 14
Set Width 1600
Set Height 900
Set Theme "Dracula"
Set Padding 25
Set WindowBar Colorful
Set Shell "bash"

Hide
Type "cd ~/Documents/work/clash"
Enter
Type "clear"
Enter
Show

Sleep 0.5s

# Start tmux
Type "tmux"
Enter
Sleep 1s

# Split into 2 panes side by side (creates right pane and moves cursor there)
Ctrl+B
Type "%"
Sleep 0.5s

# Switch back to left pane for first agent
Ctrl+B
Type "o"
Sleep 0.3s

# Now in left pane for first agent
Type "# Starting Claude for Feature 1, Clash Cli already mentioned in Claude.md..."
Enter
Sleep 0.5s
Type "claude"
Enter
Sleep 3s

# Type entire prompt as one input (will wrap naturally in Claude)
Type "Let's work on feature 1 for clash - the 'predict' command that analyzes task descriptions to predict potential merge conflicts. Use git worktrees & `CLASH CLI` to work in the clash-predict worktree & coordinate with other worktrees using clash cli instruction in clash --help or in 'claude.md'"
Enter
Sleep 10s  # Longer pause to let people read the full prompt

# Let Claude show some activity
Sleep 5s

# Exit Claude
Ctrl+C
Sleep 0.5s
Type "# Agent 1 working in clash-predict worktree..."
Enter
Sleep 1s

# Switch to right pane for second agent
Ctrl+B
Type "o"
Sleep 0.3s

Type "# Starting Claude for Feature 2..."
Enter
Sleep 0.5s
Type "claude"
Enter
Sleep 3s

# Type entire prompt as one input (will wrap naturally in Claude)
Type "Let's work on feature 2 for clash - the 'watch' command that monitors file changes across worktrees and provides real-time conflict warnings. Use git worktrees & `CLASH CLI` to work in the clash-watch worktree & coordinate with other worktrees using clash cli instruction in clash --help or in 'claude.md'"
Enter
Sleep 10s  # Longer pause to let people read the full prompt

# Let Claude show some activity
Sleep 5s

# Exit Claude
Ctrl+C
Sleep 0.5s
Type "# Agent 2 working in clash-watch worktree..."
Enter
Sleep 1s

# Exit tmux
Ctrl+B
Type "d"
Sleep 0.5s