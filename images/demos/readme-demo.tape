# @title: eden-skills README demo
# @description: Install skills, break one link, diagnose, repair, and verify.
# Assumes an Ubuntu recording environment with Claude Code installed and
# ~/.claude/skills/vercel-react-best-practices present after the install step.

Output readme-demo.gif

Require eden-skills
Require unlink

Set FontFamily "Ubuntu Mono"
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
Env TERM "xterm-256color"
Env PS1 "$ "

Sleep 600ms

Type "eden-skills install vercel-labs/agent-skills --skill vercel-react-best-practices"
Enter
Sleep 2800ms

Ctrl+L
Sleep 400ms

Type "unlink ~/.claude/skills/vercel-react-best-practices"
Sleep 300ms
Enter
Sleep 700ms

Type "eden-skills doctor --no-warning"
Enter
Sleep 2600ms

Type "eden-skills repair"
Enter
Sleep 2600ms

Ctrl+L
Sleep 400ms

Type "eden-skills doctor --no-warning"
Enter
Sleep 2400ms

Type "eden-skills list"
Enter
Sleep 2600ms