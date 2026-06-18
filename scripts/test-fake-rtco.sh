#!/usr/bin/env bash
# Fake rtco binary for test-install.sh. Simulates `rtco init --mcp
# --hooks [--uninstall] --provider <list> [--all-providers] [--dry-run]`
# by writing provider config files to $HOME in the expected shapes.
#
# This is NOT a real rtco binary. It is loaded into a sandbox DEST by
# test-install.sh and invoked by the install.sh post-install hook to
# verify the --with-mcp / --provider / --uninstall / --dry-run plumbing
# without requiring a real install (network, cargo, etc.).
set -e

PROVIDERS=""
DO_MCP=0
DO_HOOKS=0
DO_UNINSTALL=0
DRY_RUN=0
ALL_PROVIDERS=0

args=("$@")
i=0
while [ $i -lt ${#args[@]} ]; do
    case "${args[$i]}" in
        init) ;;
        --mcp) DO_MCP=1 ;;
        --hooks) DO_HOOKS=1 ;;
        --uninstall) DO_UNINSTALL=1 ;;
        --dry-run) DRY_RUN=1 ;;
        --all-providers) ALL_PROVIDERS=1 ;;
        --provider)
            i=$((i + 1))
            PROVIDERS="${args[$i]}"
            ;;
        --provider=*)
            PROVIDERS="${args[$i]#--provider=}"
            ;;
    esac
    i=$((i + 1))
done

# Resolve provider list
if [ -z "$PROVIDERS" ] || [ "$ALL_PROVIDERS" -eq 1 ]; then
    PROVIDERS="claude cursor cline windsurf copilot opencode codex gemini amazonq warp"
fi

write_json_mcp() {
    local f="$1" key="$2" extra="$3"
    [ "$DRY_RUN" -eq 1 ] && return 0
    mkdir -p "$(dirname "$f")"
    if [ ! -f "$f" ]; then
        printf '{}' > "$f"
    fi
    if [ "$DO_UNINSTALL" -eq 1 ]; then
        if [ ! -f "$f" ]; then return 0; fi
        F="$f" K="$key" python3 -c "
import json, os
p = os.environ['F']; k = os.environ['K']
try:
    d = json.load(open(p))
    if isinstance(d.get(k), dict) and 'rtco' in d[k]:
        del d[k]['rtco']
    open(p, 'w').write(json.dumps(d))
except Exception:
    pass
"
        return 0
    fi
    F="$f" K="$key" E="$extra" python3 -c "
import json, os
p = os.environ['F']; k = os.environ['K']; extra = os.environ['E']
d = json.load(open(p))
if not isinstance(d, dict): d = {}
ms = d.get(k, {})
if not isinstance(ms, dict): ms = {}
entry = dict(type='stdio', command='rtco', args=['mcp'])
if extra:
    for kv in extra.split(','):
        if '=' in kv:
            kk, vv = kv.split('=', 1)
            entry[kk] = vv
ms['rtco'] = entry
d[k] = ms
open(p, 'w').write(json.dumps(d))
"
}

write_provider() {
    local p="$1"
    case "$p" in
        claude)
            write_json_mcp "$HOME/.claude.json" mcpServers ""
            ;;
        cursor)
            write_json_mcp "$HOME/.cursor/mcp.json" mcpServers ""
            ;;
        gemini)
            write_json_mcp "$HOME/.gemini/settings.json" mcpServers "trust=True"
            ;;
        copilot)
            write_json_mcp "$HOME/.config/Code/User/settings.json" mcpServers ""
            ;;
        cline)
            write_json_mcp "$HOME/.config/Code/User/globalStorage/saoudrizwan.claude-dev/settings/cline_mcp_settings.json" mcpServers ""
            ;;
        windsurf)
            write_json_mcp "$HOME/.codeium/windsurf/mcp_config.json" mcpServers ""
            ;;
        codex)
            local f="$HOME/.codex/config.toml"
            [ "$DRY_RUN" -eq 1 ] && return 0
            mkdir -p "$(dirname "$f")"
            if [ "$DO_UNINSTALL" -eq 1 ]; then
                return 0
            fi
            cat >> "$f" <<'TOML'

[mcp_servers.rtco]
type = "stdio"
command = "rtco"
args = ["mcp"]
TOML
            ;;
        opencode)
            write_json_mcp "$HOME/.config/opencode/config.json" mcp ""
            ;;
        amazonq)
            write_json_mcp "$HOME/.aws/amazonq/mcp.json" mcpServers ""
            ;;
        warp)
            write_json_mcp "$HOME/.warp/mcp_config.json" mcpServers ""
            ;;
    esac
}

# Split providers on comma OR space, then write each
# First normalise: replace commas and whitespace with newlines, then read
_plist=()
_normalised=$(printf '%s' "$PROVIDERS" | tr ', \t' '\n')
while IFS= read -r p; do
    p_trim=$(printf '%s' "$p" | tr -d ' \t')
    if [ -n "$p_trim" ]; then
        _plist+=("$p_trim")
    fi
done <<EOF
$_normalised
EOF
for p in "${_plist[@]}"; do
    write_provider "$p"
done

exit 0
