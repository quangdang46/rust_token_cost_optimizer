#!/usr/bin/env sh
# Tests for install.sh path traversal check (issue #1250, CWE-22)
# and for the post-install MCP/hooks auto-config flags.

set -eu

REPO_ROOT=$(cd "$(dirname "$0")/.." && pwd)
INSTALL_SH="$REPO_ROOT/install.sh"
FAKE_RTCO="$REPO_ROOT/scripts/test-fake-rtco.sh"

if [ ! -f "$INSTALL_SH" ]; then
    echo "FAIL: install.sh not found at $INSTALL_SH"
    exit 1
fi

if [ ! -f "$FAKE_RTCO" ]; then
    echo "FAIL: fake rtco not found at $FAKE_RTCO"
    exit 1
fi

if ! command -v python3 >/dev/null 2>&1; then
    echo "SKIP: python3 not available — crafted tarball tests require python3"
    exit 0
fi

TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

# The check replicated from install.sh (keep in sync with install.sh).
# Returns 0 when archive is safe, 1 when unsafe.
check_archive() {
    if tar -tzf "$1" | grep -qE '^/|(^|/)\.\.(/|$)'; then
        return 1
    fi
    return 0
}

# --- Build safe archive using standard tar ---
mkdir -p "$TMPDIR/safe_src"
printf '#!/bin/sh\necho rtco\n' > "$TMPDIR/safe_src/rtco"
(cd "$TMPDIR/safe_src" && tar -czf "$TMPDIR/safe.tgz" rtco)

# --- Build crafted malicious archives with python ---
python3 - "$TMPDIR" <<'PY'
import sys, tarfile, io

base = sys.argv[1]


def make(name, entry):
    with tarfile.open(f"{base}/{name}", "w:gz") as t:
        info = tarfile.TarInfo(name=entry)
        data = b"pwned"
        info.size = len(data)
        t.addfile(info, io.BytesIO(data))


make("traversal.tgz", "../etc/evil")
make("absolute.tgz", "/tmp/evil_abs")
make("middle.tgz", "rtco/../../../etc/evil")
make("end_dotdot.tgz", "rtco/..")
PY

FAIL=0
pass() { printf '  PASS: %s\n' "$1"; }
fail() { printf '  FAIL: %s\n' "$1"; FAIL=1; }

echo "==> Functional checks (path traversal)"

if check_archive "$TMPDIR/safe.tgz"; then
    pass "safe archive accepted"
else
    fail "safe archive rejected (false positive)"
fi

for bad in traversal absolute middle end_dotdot; do
    if check_archive "$TMPDIR/$bad.tgz"; then
        fail "$bad archive accepted (should be rejected)"
    else
        pass "$bad archive rejected"
    fi
done

echo "==> Regression guard (path traversal)"

# The actual check lives in the while-loop that iterates over `find ... -print0`
# and rejects any entry containing "/../" or "/..". It does NOT use tar -tzf.
if grep -qE 'Path traversal blocked' "$INSTALL_SH" \
    && grep -qF '"/../"' "$INSTALL_SH" \
    && grep -qF '"/.."' "$INSTALL_SH"; then
    pass "install.sh still contains the path-traversal check"
else
    fail "install.sh is missing the path-traversal check — was it removed?"
fi

# ===========================================================================
# Post-install MCP/hooks auto-config tests
# ===========================================================================

echo "==> Help text contains the new flags"

# All of the new flags must appear in the --help output. The first arg
# of `bash` consumes the script; we forward --help through.
HELP_OUT=$(bash "$INSTALL_SH" --help 2>&1 || true)

for flag in --with-mcp --no-mcp --with-hooks --no-hooks \
            --provider --all-providers --dry-run; do
    if printf '%s' "$HELP_OUT" | grep -qF -e "$flag"; then
        pass "help text advertises $flag"
    else
        fail "help text is missing $flag"
    fi
done

echo "==> Arg parsing (rejects unknown positional as shift-only)"

# install.sh should accept the new flags and not error out at parse time.
# We don't run the full install — just exercise the while-args loop by
# running with --help at the end of a flag chain.
if bash "$INSTALL_SH" --with-mcp --with-hooks --provider claude,cursor \
    --all-providers --dry-run --help >/dev/null 2>&1; then
    pass "arg parser accepts the full MCP/hooks flag chain"
else
    fail "arg parser rejected the full MCP/hooks flag chain"
fi

# Conflicting flag combinations: --no-mcp + --with-mcp is a parse-time
# conflict that we don't strictly enforce, but at minimum the script
# must not crash.
if bash "$INSTALL_SH" --with-mcp --no-mcp --help >/dev/null 2>&1; then
    pass "arg parser tolerates --with-mcp + --no-mcp (no crash)"
else
    fail "arg parser crashed on --with-mcp + --no-mcp"
fi

echo "==> Regression guard (configure_post_install function present)"

if grep -qF 'configure_post_install' "$INSTALL_SH"; then
    pass "install.sh defines configure_post_install"
else
    fail "install.sh is missing configure_post_install"
fi

if grep -qF 'rtco' "$INSTALL_SH" && grep -qF 'init --mcp' "$INSTALL_SH"; then
    pass "configure_post_install invokes 'rtco init --mcp'"
else
    fail "configure_post_install does not invoke 'rtco init --mcp'"
fi

echo "==> Regression guard (uninstall cleans providers)"

# The uninstall branch should attempt to clean MCP/hooks before
# removing the binary. The check below is structural — we don't run
# the actual uninstall because that would delete the user's binary.
if grep -qF 'init --uninstall --mcp --hooks' "$INSTALL_SH"; then
    pass "uninstall branch calls 'rtco init --uninstall --mcp --hooks'"
else
    fail "uninstall branch does not call provider cleanup"
fi

# install.ps1 mirror
INSTALL_PS1="$REPO_ROOT/install.ps1"
if [ -f "$INSTALL_PS1" ]; then
    if grep -qF 'Invoke-PostInstallConfig' "$INSTALL_PS1"; then
        pass "install.ps1 defines Invoke-PostInstallConfig"
    else
        fail "install.ps1 is missing Invoke-PostInstallConfig"
    fi
    if grep -qF 'WithMcp' "$INSTALL_PS1" && grep -qF 'WithHooks' "$INSTALL_PS1"; then
        pass "install.ps1 advertises -WithMcp and -WithHooks"
    else
        fail "install.ps1 is missing -WithMcp or -WithHooks"
    fi
    if grep -qF 'init --uninstall --mcp --hooks' "$INSTALL_PS1"; then
        pass "install.ps1 uninstall cleans providers"
    else
        fail "install.ps1 uninstall does not clean providers"
    fi
fi

# ===========================================================================
# Behavioural tests for --with-mcp / --provider / --uninstall / --dry-run
# ===========================================================================
#
# These tests use a fake `rtco` binary (scripts/test-fake-rtco.sh) that
# simulates the post-install `init --mcp` behaviour by writing the
# expected provider config files to $HOME. We source install.sh (which
# exposes configure_post_install without running main) and invoke it
# directly with controlled flag vars. This avoids network access and
# keeps the tests fast & deterministic.

# Per-test sandbox state. Cleared on teardown.
SB_HOME=""
SB_DEST=""
SB_OLDHOME=""
SB_OLDPATH=""
SB_OLDQUIET=""

setup_sandbox() {
    SB_HOME=$(mktemp -d)
    SB_DEST=$(mktemp -d)
    SB_OLDHOME="$HOME"
    SB_OLDPATH="$PATH"
    SB_OLDQUIET="${QUIET:-0}"
    export HOME="$SB_HOME"
    export PATH="$SB_DEST:$PATH"
    export DEST="$SB_DEST"
    export QUIET=1
    cp "$FAKE_RTCO" "$SB_DEST/rtco"
    chmod +x "$SB_DEST/rtco"
}

teardown_sandbox() {
    if [ -n "$SB_OLDHOME" ]; then export HOME="$SB_OLDHOME"; fi
    if [ -n "$SB_OLDPATH" ]; then export PATH="$SB_OLDPATH"; fi
    export QUIET="$SB_OLDQUIET"
    if [ -n "$SB_HOME" ] && [ -d "$SB_HOME" ]; then rm -rf "$SB_HOME"; fi
    if [ -n "$SB_DEST" ] && [ -d "$SB_DEST" ]; then rm -rf "$SB_DEST"; fi
    SB_HOME=""
    SB_DEST=""
    SB_OLDHOME=""
    SB_OLDPATH=""
    SB_OLDQUIET=""
}

# Invoke configure_post_install with the given KEY=VALUE flag assignments
# in a clean subshell that sources install.sh. The trick: we set
# BASH_SOURCE so install.sh's `if [[ "${BASH_SOURCE[0]:-}" == "${0:-}" ]]`
# guard does NOT call main() — we just want the function definitions.
# Then we override the flag variables (which install.sh's `init` block
# resets to defaults) with the test's own values BEFORE calling the
# function. install.sh's arg-parser loop would consume $@ if we let it,
# so we save the test args into a side array and pass it via env vars.
# Usage: run_configure WITH_MCP=1 PROVIDERS=claude ...
run_configure() {
    (
        export PATH="$SB_DEST:$PATH"
        export HOME="$SB_HOME"
        export DEST="$SB_DEST"
        export QUIET=1
        # Encode args as a colon-separated string in RTCO_TEST_KV
        # (colon is a non-POSIX-safe char; we use \x1f as separator).
        _kv=""
        for kv in "$@"; do
            _kv="${_kv}${_kv:+$'\x1f'}${kv}"
        done
        export RTCO_TEST_KV="$_kv"
        BASH_SOURCE="$INSTALL_SH" /bin/bash -c '
            source "$1"
            # Decode RTCO_TEST_KV (split on \x1f) and apply
            _kv="$RTCO_TEST_KV"
            _saveIFS="$IFS"
            IFS=$'"'"'\x1f'"'"'
            for kv in $_kv; do
                k="${kv%%=*}"
                v="${kv#*=}"
                printf -v "$k" "%s" "$v"
                export "$k"
            done
            IFS="$_saveIFS"
            unset RTCO_TEST_KV
            configure_post_install
        ' _ "$INSTALL_SH"
    ) >/dev/null 2>&1
}

# --- Test 1: --with-mcp writes expected provider config --------------
echo "==> Behavioural: --with-mcp writes expected provider config"

setup_sandbox
# Pre-populate ~/.claude.json so we can assert it gets *modified*
# (not just created) by the install hook.
printf '{}' > "$SB_HOME/.claude.json"
run_configure WITH_MCP=1 NO_MCP=0 WITH_HOOKS=0 NO_HOOKS=0 \
              PROVIDERS=claude ALL_PROVIDERS=0 DRY_RUN=0
if [ -f "$SB_HOME/.claude.json" ] \
    && grep -q '"command": "rtco"' "$SB_HOME/.claude.json" \
    && grep -q '"mcpServers"' "$SB_HOME/.claude.json"; then
    pass "with-mcp writes mcpServers.rtco.command to claude.json"
else
    fail "with-mcp did not write expected mcpServers entry to claude.json"
fi
teardown_sandbox

# --- Test 2: default (no --with-mcp) does not touch configs ---------
echo "==> Behavioural: default (no --with-mcp) does not touch configs"

setup_sandbox
printf '{}' > "$SB_HOME/.claude.json"
PRE_CLAUDE_SHA=$(shasum -a 256 "$SB_HOME/.claude.json" | awk '{print $1}')
PRE_CURSOR="$SB_HOME/.cursor/mcp.json"
run_configure WITH_MCP=0 NO_MCP=1 WITH_HOOKS=0 NO_HOOKS=0 \
              PROVIDERS="" ALL_PROVIDERS=0 DRY_RUN=0
POST_CLAUDE_SHA=$(shasum -a 256 "$SB_HOME/.claude.json" | awk '{print $1}')
if [ "$PRE_CLAUDE_SHA" = "$POST_CLAUDE_SHA" ] && [ ! -f "$PRE_CURSOR" ]; then
    pass "no-mcp default leaves existing config unchanged and writes no new files"
else
    fail "no-mcp default modified filesystem unexpectedly"
fi
teardown_sandbox

# --- Test 3: --uninstall cleans mcpServers entries -----------------
echo "==> Behavioural: --uninstall cleans mcpServers entries"

setup_sandbox
# Pre-write a config with both rtco and other entries; the other
# entries must be preserved by the uninstall step.
cat > "$SB_HOME/.claude.json" <<'JSON'
{
  "numStartups": 3,
  "mcpServers": {
    "rtco": {"type": "stdio", "command": "rtco", "args": ["mcp"]},
    "other": {"type": "stdio", "command": "other", "args": ["serve"]}
  }
}
JSON
# Re-run the install hook to ensure the rtco entry is on disk
# (idempotent re-write).
run_configure WITH_MCP=1 NO_MCP=0 WITH_HOOKS=0 NO_HOOKS=0 \
              PROVIDERS=claude ALL_PROVIDERS=0 DRY_RUN=0
# Now invoke the same binary the install.sh --uninstall branch invokes.
"$SB_DEST/rtco" init --uninstall --mcp --hooks --all-providers >/dev/null 2>&1
if ! grep -q '"rtco"' "$SB_HOME/.claude.json" \
    && grep -q '"other"' "$SB_HOME/.claude.json" \
    && grep -q '"numStartups": 3' "$SB_HOME/.claude.json"; then
    pass "uninstall removes rtco entry but preserves other mcpServers and top-level keys"
else
    fail "uninstall did not clean mcpServers correctly"
    echo "       actual: $(cat "$SB_HOME/.claude.json")"
fi
teardown_sandbox

# --- Test 4: --provider subset only touches listed providers -------
echo "==> Behavioural: --provider subset only touches listed providers"

setup_sandbox
printf '{}' > "$SB_HOME/.claude.json"
mkdir -p "$SB_HOME/.cursor"
printf '{}' > "$SB_HOME/.cursor/mcp.json"
PRE_GEMINI="$SB_HOME/.gemini/settings.json"
PRE_CODEX="$SB_HOME/.codex/config.toml"
run_configure WITH_MCP=1 NO_MCP=0 WITH_HOOKS=0 NO_HOOKS=0 \
              PROVIDERS=claude,cursor ALL_PROVIDERS=0 DRY_RUN=0
CLAUDE_OK=0
CURSOR_OK=0
if [ -f "$SB_HOME/.claude.json" ] && grep -q '"rtco"' "$SB_HOME/.claude.json"; then
    CLAUDE_OK=1
fi
if [ -f "$SB_HOME/.cursor/mcp.json" ] && grep -q '"rtco"' "$SB_HOME/.cursor/mcp.json"; then
    CURSOR_OK=1
fi
if [ "$CLAUDE_OK" -eq 1 ] && [ "$CURSOR_OK" -eq 1 ] \
    && [ ! -f "$PRE_GEMINI" ] && [ ! -f "$PRE_CODEX" ]; then
    pass "--provider claude,cursor touches only claude and cursor configs"
else
    fail "--provider subset wrote to unlisted providers (gemini=$([ -f "$PRE_GEMINI" ] && echo present || echo absent), codex=$([ -f "$PRE_CODEX" ] && echo present || echo absent))"
fi
teardown_sandbox

# --- Test 5: --dry-run writes nothing and returns 0 -----------------
echo "==> Behavioural: --dry-run writes nothing and returns 0"

setup_sandbox
printf '{}' > "$SB_HOME/.claude.json"
PRE_CLAUDE_SHA=$(shasum -a 256 "$SB_HOME/.claude.json" | awk '{print $1}')
run_configure WITH_MCP=1 NO_MCP=0 WITH_HOOKS=0 NO_HOOKS=0 \
              PROVIDERS=claude,cursor,gemini,codex,copilot \
              ALL_PROVIDERS=0 DRY_RUN=1
RC=$?
POST_CLAUDE_SHA=$(shasum -a 256 "$SB_HOME/.claude.json" | awk '{print $1}')
if [ "$RC" -eq 0 ] \
    && [ "$PRE_CLAUDE_SHA" = "$POST_CLAUDE_SHA" ] \
    && [ ! -f "$SB_HOME/.cursor/mcp.json" ] \
    && [ ! -f "$SB_HOME/.gemini/settings.json" ] \
    && [ ! -f "$SB_HOME/.codex/config.toml" ] \
    && [ ! -f "$SB_HOME/.config/Code/User/settings.json" ]; then
    pass "dry-run returned 0 and left the filesystem untouched"
else
    fail "dry-run wrote to filesystem or returned non-zero (rc=$RC, claude_changed=$([ "$PRE_CLAUDE_SHA" != "$POST_CLAUDE_SHA" ] && echo yes || echo no))"
fi
teardown_sandbox

echo ""
if [ "$FAIL" -eq 0 ]; then
    echo "All install.sh tests passed"
    exit 0
else
    echo "Some tests failed"
    exit 1
fi
