# Clash watch mode with real-time file change detection
# Shows the TUI automatically updating when files are modified

Output ../output/clash-watch-realtime-demo.gif

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

# Split into 2 panes side by side
Ctrl+B
Type "%"
Sleep 0.5s

# Switch to left pane
Ctrl+B
Type "o"
Sleep 0.3s

# Start clash watch in left pane
Type "cd ~/Documents/work/clash"
Enter
Sleep 0.3s
Type "clash watch"
Enter
Sleep 3s

# Switch to right pane to make file changes
Ctrl+B
Type "o"
Sleep 0.3s

Type "cd ~/Documents/work/clash-watch"
Enter
Sleep 0.5s
Type "# Making a file change..."
Enter
Sleep 1s
Type "echo '// Watch feature update' >> src/watch.rs"
Enter
Sleep 3s

Type "# Watch mode detected the change!"
Enter
Sleep 2s

Type "# Reverting change..."
Enter
Sleep 1s
Type "git restore src/watch.rs"
Enter
Sleep 5s

# Exit tmux
Ctrl+B
Type "d"
Sleep 0.5s