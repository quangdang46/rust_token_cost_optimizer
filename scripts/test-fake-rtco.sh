#!/usr/bin/env bash
# Fake rtco binary for test-install.sh. Simulates `rtco init --hooks`
# (and `rtco init --uninstall --hooks`) by recording the args it received
# and writing a hooks marker file into $HOME.
#
# This is NOT a real rtco binary. It is loaded into a sandbox DEST by
# test-install.sh and invoked by the install.sh post-install hook to
# verify the --with-hooks / --no-hooks plumbing without requiring a real
# install (network, cargo, etc.).
set -e

# Append every arg to the invocation log so tests can assert the exact
# flag vector install.sh builds (e.g. "init --hooks").
echo "$*" >> "${RTCO_INVOCATION_LOG:-/dev/null}"

DO_HOOKS=0
DO_UNINSTALL=0

args=("$@")
i=0
while [ $i -lt ${#args[@]} ]; do
    case "${args[$i]}" in
        init) ;;
        --hooks) DO_HOOKS=1 ;;
        --uninstall) DO_UNINSTALL=1 ;;
    esac
    i=$((i + 1))
done

# Write a marker so tests can confirm the hook path ran. Mirrors what the
# real `rtco init --hooks` does (registering the rtco rewrite hook in the
# provider config) in a lightweight, sandbox-local way.
mkdir -p "$HOME/.rtco-test"
if [ "$DO_UNINSTALL" -eq 1 ]; then
    rm -f "$HOME/.rtco-test/hooks-installed"
else
    printf 'rtco-hooks' > "$HOME/.rtco-test/hooks-installed"
fi

exit 0
