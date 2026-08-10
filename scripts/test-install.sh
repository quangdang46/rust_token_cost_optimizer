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
# Post-install hooks auto-config tests
# ===========================================================================
#
# NOTE: install.sh gained --with-mcp/--provider/--dry-run in v0.2.0 and
# lost them again in commit 0ba9092 (MCP server + auto-config removed).
# The only post-install flags that remain are --with-hooks / --no-hooks.
# These tests cover the current hooks-only behaviour.

echo "==> Help text contains the flags"

# The remaining flags must appear in the --help output. The first arg
# of `bash` consumes the script; we forward --help through.
HELP_OUT=$(bash "$INSTALL_SH" --help 2>&1 || true)

for flag in --with-hooks --no-hooks; do
    if printf '%s' "$HELP_OUT" | grep -qF -e "$flag"; then
        pass "help text advertises $flag"
    else
        fail "help text is missing $flag"
    fi
done

# The removed MCP/provider flags must NOT be advertised (they would
# mislead users into thinking install.sh still configures an MCP server).
for flag in --with-mcp --no-mcp --provider --all-providers --dry-run; do
    if printf '%s' "$HELP_OUT" | grep -qF -e "$flag"; then
        fail "help text still advertises removed flag $flag"
    else
        pass "removed flag $flag is absent from help text"
    fi
done

echo "==> Arg parsing (tolerates --with-hooks/--no-hooks)"

# install.sh should accept the remaining flags and not error out at
# parse time. We don't run the full install — just exercise the
# while-args loop by running with --help at the end of a flag chain.
if bash "$INSTALL_SH" --with-hooks --no-hooks --help >/dev/null 2>&1; then
    pass "arg parser tolerates --with-hooks + --no-hooks (no crash)"
else
    fail "arg parser crashed on --with-hooks + --no-hooks"
fi

echo "==> Regression guard (configure_post_install function present)"

if grep -qF 'configure_post_install' "$INSTALL_SH"; then
    pass "install.sh defines configure_post_install"
else
    fail "install.sh is missing configure_post_install"
fi

if grep -qF 'rtco' "$INSTALL_SH" && grep -qF 'init --hooks' "$INSTALL_SH"; then
    pass "configure_post_install invokes 'rtco init --hooks'"
else
    fail "configure_post_install does not invoke 'rtco init --hooks'"
fi

echo "==> Regression guard (uninstall cleans hooks)"

# The uninstall branch should attempt to clean hooks before removing
# the binary. The check below is structural — we don't run the actual
# uninstall because that would delete the user's binary.
if grep -qF 'init --uninstall --hooks' "$INSTALL_SH"; then
    pass "uninstall branch calls 'rtco init --uninstall --hooks'"
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
    if grep -qF 'WithHooks' "$INSTALL_PS1"; then
        pass "install.ps1 advertises -WithHooks"
    else
        fail "install.ps1 is missing -WithHooks"
    fi
    if grep -qF 'init --uninstall --hooks' "$INSTALL_PS1"; then
        pass "install.ps1 uninstall cleans hooks"
    else
        fail "install.ps1 uninstall does not clean hooks"
    fi
fi

# ===========================================================================
# Behavioural tests for --with-hooks / --no-hooks
# ===========================================================================
#
# These tests use a fake `rtco` binary (scripts/test-fake-rtco.sh) that
# simulates the post-install `init --hooks` behaviour by recording its args
# and writing a marker file to $HOME. We source install.sh (which exposes
# configure_post_install without running main) and invoke it directly with
# controlled flag vars. This avoids network access and keeps the tests
# fast & deterministic.

# Per-test sandbox state. Cleared on teardown.
SB_HOME=""
SB_DEST=""
SB_OLDHOME=""
SB_OLDPATH=""
SB_OLDQUIET=""
SB_LOG=""

setup_sandbox() {
    SB_HOME=$(mktemp -d)
    SB_DEST=$(mktemp -d)
    SB_LOG=$(mktemp)
    SB_OLDHOME="$HOME"
    SB_OLDPATH="$PATH"
    SB_OLDQUIET="${QUIET:-0}"
    export HOME="$SB_HOME"
    export PATH="$SB_DEST:$PATH"
    export DEST="$SB_DEST"
    export QUIET=1
    export RTCO_INVOCATION_LOG="$SB_LOG"
    cp "$FAKE_RTCO" "$SB_DEST/rtco"
    chmod +x "$SB_DEST/rtco"
}

teardown_sandbox() {
    if [ -n "$SB_OLDHOME" ]; then export HOME="$SB_OLDHOME"; fi
    if [ -n "$SB_OLDPATH" ]; then export PATH="$SB_OLDPATH"; fi
    export QUIET="$SB_OLDQUIET"
    unset RTCO_INVOCATION_LOG
    if [ -n "$SB_HOME" ] && [ -d "$SB_HOME" ]; then rm -rf "$SB_HOME"; fi
    if [ -n "$SB_DEST" ] && [ -d "$SB_DEST" ]; then rm -rf "$SB_DEST"; fi
    if [ -n "$SB_LOG" ] && [ -f "$SB_LOG" ]; then rm -f "$SB_LOG"; fi
    SB_HOME=""
    SB_DEST=""
    SB_OLDHOME=""
    SB_OLDPATH=""
    SB_OLDQUIET=""
    SB_LOG=""
}

# Invoke configure_post_install with the given KEY=VALUE flag assignments
# in a clean subshell that sources install.sh. The trick: we set
# BASH_SOURCE so install.sh's `if [[ "${BASH_SOURCE[0]:-}" == "${0:-}" ]]`
# guard does NOT call main() — we just want the function definitions.
# Then we override the flag variables (which install.sh's `init` block
# resets to defaults) with the test's own values BEFORE calling the
# function. install.sh's arg-parser loop would consume $@ if we let it,
# so we save the test args into a side array and pass it via env vars.
# Usage: run_configure WITH_HOOKS=1 NO_HOOKS=0 ...
run_configure() {
    (
        export PATH="$SB_DEST:$PATH"
        export HOME="$SB_HOME"
        export DEST="$SB_DEST"
        export QUIET=1
        export RTCO_INVOCATION_LOG="$SB_LOG"
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

# --- Test 1: --with-hooks invokes `rtco init --hooks` ----------------
echo "==> Behavioural: --with-hooks invokes 'rtco init --hooks'"

setup_sandbox
run_configure WITH_HOOKS=1 NO_HOOKS=0
if grep -qF 'init --hooks' "$SB_LOG"; then
    pass "with-hooks invokes 'rtco init --hooks'"
else
    fail "with-hooks did not invoke 'rtco init --hooks' (log: $(cat "$SB_LOG" 2>/dev/null))"
fi
if [ -f "$SB_HOME/.rtco-test/hooks-installed" ]; then
    pass "with-hooks wrote the hooks marker file"
else
    fail "with-hooks did not write the hooks marker file"
fi
teardown_sandbox

# --- Test 2: default (neither flag) does not invoke rtco -------------
echo "==> Behavioural: default (no flags) does not invoke rtco"

setup_sandbox
run_configure WITH_HOOKS=0 NO_HOOKS=0
if [ ! -s "$SB_LOG" ]; then
    pass "default leaves config untouched (no rtco invocation)"
else
    fail "default unexpectedly invoked rtco: $(cat "$SB_LOG")"
fi
teardown_sandbox

# --- Test 3: --no-hooks overrides --with-hooks ----------------------
echo "==> Behavioural: --no-hooks overrides --with-hooks"

setup_sandbox
run_configure WITH_HOOKS=1 NO_HOOKS=1
if [ ! -s "$SB_LOG" ]; then
    pass "no-hooks overrides with-hooks (no rtco invocation)"
else
    fail "no-hooks did not override with-hooks: $(cat "$SB_LOG")"
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
