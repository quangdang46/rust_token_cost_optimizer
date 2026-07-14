//! Hook installation and lifecycle management for AI coding agents.

pub mod constants;
pub mod hook_audit_cmd;
pub mod hook_check;
#[deny(clippy::print_stdout, clippy::print_stderr)]
pub mod hook_cmd;
pub mod init;
pub mod integrity;
pub mod permissions;
pub mod rewrite_cmd;
pub mod trust;
pub mod verify_cmd;

/// True when `command` is an `rtco hook claude` invocation, bare or absolute-path.
///
/// Matches `/opt/homebrew/bin/rtco hook claude` and quoted forms so `rtco init`
/// does not re-register a hook that is already present under a full binary path
/// (upstream RTK #2885).
pub fn is_claude_hook_command(command: &str) -> bool {
    let parts = crate::discover::lexer::shell_split(command);
    let [binary, hook, claude] = parts.as_slice() else {
        return false;
    };

    let binary_name = binary.rsplit(['/', '\\']).next().unwrap_or(binary);

    binary_name == "rtco" && *hook == "hook" && *claude == "claude"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claude_hook_command_matches_bare_and_absolute_rtco() {
        assert!(is_claude_hook_command("rtco hook claude"));
        assert!(is_claude_hook_command("/opt/homebrew/bin/rtco hook claude"));
        assert!(is_claude_hook_command(
            "\"/opt/homebrew/bin/rtco\" hook claude"
        ));
    }

    #[test]
    fn claude_hook_command_rejects_other_commands() {
        assert!(!is_claude_hook_command("not-rtco hook claude"));
        assert!(!is_claude_hook_command(
            "/opt/homebrew/bin/rtco hook cursor"
        ));
        assert!(!is_claude_hook_command("echo rtco hook claude"));
        assert!(!is_claude_hook_command("rtk hook claude"));
    }
}
