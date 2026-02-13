pub fn default_agent_path(agent: &str) -> Option<&'static str> {
    match agent {
        "claude-code" => Some("~/.claude/skills"),
        "cursor" => Some("~/.cursor/skills"),
        _ => None,
    }
}
