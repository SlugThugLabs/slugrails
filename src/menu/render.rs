//! ANSI and text rendering for the interactive setup menu.

use crate::style::Style;

pub const MENU: &str = "\
┌─────────────────────────────────────────────┐
│           CodeGuards setup                  │
└─────────────────────────────────────────────┘

  ── Setup ───────────────────────────────────

  1) Install the binary  (~/.slugthuglabs/bin)
     A stable path for agents and MCP clients to launch.

  2) Connect to an AI agent
     Register this binary as the `codeguards` MCP server in
     Hermes, Claude Code, Grok, or Codex.

  3) Add CodeGuards to another MCP client
     Print instructions + a config snippet for any other
     tool that supports MCP servers (OpenCode, Cursor, Zed, ...).

  4) Inspect guard-test library
     List available modular tests in ~/.slugthuglabs/slugrails/tests/

  ── Advanced ─────────────────────────────────

  5) Run the MCP server now
     Advanced: starts `serve` in this terminal and blocks
     until Ctrl-C. Normally your AI agent starts it for you.

  ── Exit ─────────────────────────────────────

  6) Exit

Choose an option [1-6]: ";

#[must_use]
pub fn render_menu(style: Style) -> String {
    if !style.enabled() {
        return MENU.to_owned();
    }
    let mut out = String::with_capacity(MENU.len() + 96);

    for line in MENU.lines() {
        let trimmed = line.trim();
        if let Some(label) = section_label(trimmed) {
            let prefix_len = line.find(label).unwrap_or(0);
            let before = &line[..prefix_len];
            let after = &line[prefix_len + label.len()..];
            out.push_str(before);
            out.push_str(&style.green(&style.bold(label)));
            out.push_str(&style.dim(after));
        } else if trimmed.starts_with('┌') || trimmed.starts_with('└') {
            out.push_str(&style.dim(line));
        } else if trimmed.contains("CodeGuards setup") {
            out.push_str(&line.replace(
                "CodeGuards setup",
                &style.bold(&style.cyan("CodeGuards setup")),
            ));
        } else if trimmed.starts_with("Choose an option") {
            out.push_str(&style.bold(&style.yellow(line)));
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }

    out
}

fn section_label(trimmed: &str) -> Option<&str> {
    let rest = trimmed.strip_prefix("── ")?;
    let (label, _after) = rest.split_once(" ─")?;
    if label.is_empty() || !label.chars().all(|c| c.is_alphanumeric() || c == ' ') {
        return None;
    }
    Some(label)
}
