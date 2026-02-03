# Clash watch mode demonstration (simple version)
# Shows the real-time TUI monitoring interface

Output ../output/clash-watch-demo.gif

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

Type "clash watch"
Sleep 0.5s
Enter
Sleep 5s

# Exit watch mode
Type "q"
Sleep 0.5s