# Claude Code PreToolUse hook integration with clash check
# Demonstrates zero-config conflict detection before every file write

Output ../output/clash-check-hook-demo.gif

Set FontSize 16
Set Width 1600
Set Height 900
Set Theme "Dracula"
Set Padding 25
Set WindowBar Colorful
Set Shell "bash"
Set TypingSpeed 100ms

Hide
Type "cd ~/Documents/work/clash"
Enter
Type "claude"
Enter
Sleep 4s
Show

Sleep 1s

# Ask Claude to edit a file that conflicts with another worktree
Set TypingSpeed 50ms
Type "Edit src/main.rs and add a new CLI flag --verbose for debug output"
Enter

# Claude thinks and starts working
Sleep 5s

# The hook fires - "Running PreToolUse hook..." message appears
# followed by the conflict detection and permission prompt
Sleep 15s

# User reads the conflict warning and decides
Sleep 10s

# Exit Claude
Ctrl+C
Sleep 5.5s

Set TypingSpeed 50ms
Type "# clash check detected the conflict before Claude wrote the file!"
Enter
Sleep 3s
