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
Type "cd /Users/kuriakosemathew/Documents/work/clash-sh/clash"
Enter
Type "clear"
Enter
Type "claude"
Enter
Sleep 6s
Show

Sleep 1s

# Use a very direct prompt so Claude attempts a file edit quickly
Set TypingSpeed 50ms
Type "Add a comment to the top of src/main.rs saying // TODO: add verbose flag"
Enter

# Claude needs time to read context, think, and attempt the edit
# The hook fires when Claude tries to use Write/Edit tool
Sleep 10s

# Exit Claude
Ctrl+C
Sleep 1s

Set TypingSpeed 50ms
Type "# clash detected the conflict before Claude wrote the file!"
Enter
Sleep 6s
