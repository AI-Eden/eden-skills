# @title: eden-skills README demo
# @description: Install skills, break one link, diagnose, repair, and verify.
# Assumes Claude Code is installed in the recording environment and
# ~/.claude/skills/react-best-practices exists after the install step.

Output readme-demo.gif

Require eden-skills
Require unlink

Set FontFamily "JetBrains Mono"
Set FontSize 18
Set Width 1000
Set Height 620
Set Theme "Catppuccin Mocha"
Set Padding 20
Set BorderRadius 12
Set TypingSpeed 35ms
Set Framerate 24
Set Shell "bash"
Set WindowBar Colorful
Set WindowBarSize 36
Set Env "TERM" "xterm-256color"
Set Env "PS1" "$ "

Sleep 600ms

Type "eden-skills install vercel-labs/agent-skills --all"
Enter
Sleep 2800ms

Ctrl+L
Sleep 400ms

Type "unlink ~/.claude/skills/react-best-practices"
Sleep 300ms
Enter
Sleep 700ms

Type "eden-skills doctor"
Enter
Sleep 2600ms

Type "eden-skills repair"
Enter
Sleep 2600ms

Ctrl+L
Sleep 400ms

Type "eden-skills doctor"
Enter
Sleep 2400ms
