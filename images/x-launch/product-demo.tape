# @title: eden-skills X launch demo
# @description: Shorter X-ready terminal demo focused on recovery.

Output product-demo.mp4

Require eden-skills
Require unlink

Set FontFamily "Iosevka Nerd Font Mono"
Set FontSize 32
Set Width 1920
Set Height 1080
Set Theme "Catppuccin Mocha"
Set Padding 0
Set BorderRadius 0
Set TypingSpeed 28ms
Set Framerate 24
Set Shell "bash"
Set WindowBar Colorful
Set WindowBarSize 64
Env TERM "xterm-256color"
Env PS1 "\[\e[32m\]❯ \[\e[0m\]"

Sleep 500ms

# install

Type "eden-skills install vercel-labs/agent-skills"
Enter
Sleep 2500ms
Down
Sleep 260ms
Down
Sleep 260ms
Space
Sleep 500ms
Enter
Sleep 2500ms

# break it

Type "unlink ~/.claude/skills/vercel-react-best-practices"
Enter
Sleep 1200ms

Ctrl+L
Sleep 350ms

Type "eden-skills doctor --no-warning"
Enter
Sleep 2300ms

Type "eden-skills repair"
Enter
Sleep 2300ms

Type "eden-skills doctor --no-warning"
Enter
Sleep 2200ms
