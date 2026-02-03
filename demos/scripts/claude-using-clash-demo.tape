# Claude Code using Clash to analyze conflicts
# Demonstrates AI-assisted conflict resolution workflow

Output ../output/claude-using-clash-demo.gif

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

# Ask Claude to use Clash to check for conflicts - SLOW and DRAMATIC
Set TypingSpeed 50ms
Type "Hey Claude, can you use the clash CLI to check for conflicts between the worktrees in our project and help me understand & resolve the conflicts that exist?"
Enter

# Let Claude think and process
Sleep 3s

# Claude working (you'll see the actual output here)
Sleep 20s

# Give time to read the results
Sleep 15s

# Exit Claude
Ctrl+C
Sleep 5.5s

Set TypingSpeed 50ms
Type "# Claude analyzed the conflicts using clash!"
Enter
Sleep 3s