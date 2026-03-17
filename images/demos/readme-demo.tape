# @title: eden-skills README demo
# @description: Install skills, break one link, diagnose, repair, and verify.
# Assumes an Ubuntu recording environment with Claude Code installed and
# ~/.claude/skills/vercel-react-best-practices present after the install step.

Output readme-demo.gif

Require eden-skills
Require unlink

Set FontFamily "'IosevkaTerm Nerd Font Mono', 'Ubuntu Mono', 'monospace'"
Set FontSize 36
Set Width 2000
Set Height 1240
Set Theme "Catppuccin Mocha"
Set Padding 0
Set BorderRadius 0
Set TypingSpeed 35ms
Set Framerate 24
Set Shell "bash"
Set WindowBar Colorful
Set WindowBarSize 72
Env TERM "xterm-256color"
Env PS1 "\[\e[32m\]❯ \[\e[0m\]"

Sleep 900ms

# install

Type "eden-skills install vercel-labs/agent-skills"
Enter
Sleep 3800ms
Down
Sleep 450ms
Down
Sleep 450ms
Space
Sleep 1000ms
Enter
Sleep 3800ms

# break the symlink deliberately

Type "# simulate a broken install"
Sleep 500ms
Enter
Sleep 700ms

Type "unlink ~/.claude/skills/vercel-react-best-practices"
Sleep 500ms
Enter
Sleep 1800ms

Ctrl+L
Sleep 500ms

Type "eden-skills doctor --no-warning"
Enter
Sleep 3200ms

Type "eden-skills repair"
Enter
Sleep 3200ms

Type "eden-skills doctor --no-warning"
Enter
Sleep 3000ms

Type "eden-skills list"
Enter
Sleep 3200ms