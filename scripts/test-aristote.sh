#!/usr/bin/env bash
#
# RTCO Smoke Tests — Aristote Project (Vite + React + TS + ESLint)
# Tests RTCO commands in a real JS/TS project context.
# Usage: bash scripts/test-aristote.sh
#
set -euo pipefail

ARISTOTE="/Users/florianbruniaux/Sites/MethodeAristote/aristote-school-boost"

PASS=0
FAIL=0
SKIP=0
FAILURES=()

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

assert_ok() {
    local name="$1"; shift
    local output
    if output=$("$@" 2>&1); then
        PASS=$((PASS + 1))
        printf "  ${GREEN}PASS${NC}  %s\n" "$name"
    else
        FAIL=$((FAIL + 1))
        FAILURES+=("$name")
        printf "  ${RED}FAIL${NC}  %s\n" "$name"
        printf "        cmd: %s\n" "$*"
        printf "        out: %s\n" "$(echo "$output" | head -3)"
    fi
}

assert_contains() {
    local name="$1"; local needle="$2"; shift 2
    local output
    if output=$("$@" 2>&1) && echo "$output" | grep -q "$needle"; then
        PASS=$((PASS + 1))
        printf "  ${GREEN}PASS${NC}  %s\n" "$name"
    else
        FAIL=$((FAIL + 1))
        FAILURES+=("$name")
        printf "  ${RED}FAIL${NC}  %s\n" "$name"
        printf "        expected: '%s'\n" "$needle"
        printf "        got: %s\n" "$(echo "$output" | head -3)"
    fi
}

# Allow non-zero exit but check output
assert_output() {
    local name="$1"; local needle="$2"; shift 2
    local output
    output=$("$@" 2>&1) || true
    if echo "$output" | grep -q "$needle"; then
        PASS=$((PASS + 1))
        printf "  ${GREEN}PASS${NC}  %s\n" "$name"
    else
        FAIL=$((FAIL + 1))
        FAILURES+=("$name")
        printf "  ${RED}FAIL${NC}  %s\n" "$name"
        printf "        expected: '%s'\n" "$needle"
        printf "        got: %s\n" "$(echo "$output" | head -3)"
    fi
}

skip_test() {
    local name="$1"; local reason="$2"
    SKIP=$((SKIP + 1))
    printf "  ${YELLOW}SKIP${NC}  %s (%s)\n" "$name" "$reason"
}

section() {
    printf "\n${BOLD}${CYAN}── %s ──${NC}\n" "$1"
}

# ── Preamble ─────────────────────────────────────────

RTCO=$(command -v rtco || echo "")
if [[ -z "$RTCO" ]]; then
    echo "rtco not found in PATH. Run: cargo install --path ."
    exit 1
fi

if [[ ! -d "$ARISTOTE" ]]; then
    echo "Aristote project not found at $ARISTOTE"
    exit 1
fi

printf "${BOLD}RTCO Smoke Tests — Aristote Project${NC}\n"
printf "Binary: %s (%s)\n" "$RTCO" "$(rtco --version)"
printf "Project: %s\n" "$ARISTOTE"
printf "Date: %s\n\n" "$(date '+%Y-%m-%d %H:%M')"

# ── 1. File exploration ──────────────────────────────

section "Ls & Find"

assert_ok       "rtco ls project root"           rtco ls "$ARISTOTE"
assert_ok       "rtco ls src/"                   rtco ls "$ARISTOTE/src"
assert_ok       "rtco ls --depth 3"              rtco ls --depth 3 "$ARISTOTE/src"
assert_contains "rtco ls shows components/"      "components" rtco ls "$ARISTOTE/src"
assert_ok       "rtco find *.tsx"                rtco find "*.tsx" "$ARISTOTE/src"
assert_ok       "rtco find *.ts"                 rtco find "*.ts" "$ARISTOTE/src"
assert_contains "rtco find finds App.tsx"        "App.tsx" rtco find "*.tsx" "$ARISTOTE/src"

# ── 2. Read ──────────────────────────────────────────

section "Read"

assert_ok       "rtco read tsconfig.json"        rtco read "$ARISTOTE/tsconfig.json"
assert_ok       "rtco read package.json"         rtco read "$ARISTOTE/package.json"
assert_ok       "rtco read App.tsx"              rtco read "$ARISTOTE/src/App.tsx"
assert_ok       "rtco read --level aggressive"   rtco read --level aggressive "$ARISTOTE/src/App.tsx"
assert_ok       "rtco read --max-lines 10"       rtco read --max-lines 10 "$ARISTOTE/src/App.tsx"

# ── 3. Grep ──────────────────────────────────────────

section "Grep"

assert_ok       "rtco grep import"               rtco grep "import" "$ARISTOTE/src"
assert_ok       "rtco grep with type filter"     rtco grep "useState" "$ARISTOTE/src" -t tsx
assert_contains "rtco grep finds components"     "import" rtco grep "import" "$ARISTOTE/src"

# ── 4. Git ───────────────────────────────────────────

section "Git (in Aristote repo)"

# rtco git doesn't support -C, use git -C via subshell
assert_ok       "rtco git status"                bash -c "cd $ARISTOTE && rtco git status"
assert_ok       "rtco git log"                   bash -c "cd $ARISTOTE && rtco git log"
assert_ok       "rtco git branch"                bash -c "cd $ARISTOTE && rtco git branch"

# ── 5. Deps ──────────────────────────────────────────

section "Deps"

assert_ok       "rtco deps"                      rtco deps "$ARISTOTE"
assert_contains "rtco deps shows package.json"   "package.json" rtco deps "$ARISTOTE"

# ── 6. Json ──────────────────────────────────────────

section "Json"

assert_ok       "rtco json tsconfig"             rtco json "$ARISTOTE/tsconfig.json"
assert_ok       "rtco json package.json"         rtco json "$ARISTOTE/package.json"

# ── 7. Env ───────────────────────────────────────────

section "Env"

assert_ok       "rtco env"                       rtco env
assert_ok       "rtco env --filter NODE"         rtco env --filter NODE

# ── 8. Tsc ───────────────────────────────────────────

section "TypeScript (tsc)"

if command -v npx >/dev/null 2>&1 && [[ -d "$ARISTOTE/node_modules" ]]; then
    assert_output "rtco tsc (in aristote)" "error\|✅\|TS" rtco tsc --project "$ARISTOTE"
else
    skip_test "rtco tsc" "node_modules not installed"
fi

# ── 9. ESLint ────────────────────────────────────────

section "ESLint (lint)"

if command -v npx >/dev/null 2>&1 && [[ -d "$ARISTOTE/node_modules" ]]; then
    assert_output "rtco lint (in aristote)" "error\|warning\|✅\|violations\|clean" rtco lint --project "$ARISTOTE"
else
    skip_test "rtco lint" "node_modules not installed"
fi

# ── 10. Build (Vite) ─────────────────────────────────

section "Build (Vite via rtco next)"

if [[ -d "$ARISTOTE/node_modules" ]]; then
    # Aristote uses Vite, not Next — but rtco next wraps the build script
    # Test with a timeout since builds can be slow
    skip_test "rtco next build" "Vite project, not Next.js — use npm run build directly"
else
    skip_test "rtco next build" "node_modules not installed"
fi

# ── 11. Diff ─────────────────────────────────────────

section "Diff"

# Diff two config files that exist in the project
assert_ok       "rtco diff tsconfigs"            rtco diff "$ARISTOTE/tsconfig.json" "$ARISTOTE/tsconfig.app.json"

# ── 12. Summary & Err ────────────────────────────────

section "Summary & Err"

assert_ok       "rtco summary ls"                rtco summary ls "$ARISTOTE/src"
assert_ok       "rtco err ls"                    rtco err ls "$ARISTOTE/src"

# ── 13. Gain ─────────────────────────────────────────

section "Gain (after above commands)"

assert_ok       "rtco gain"                      rtco gain
assert_ok       "rtco gain --history"            rtco gain --history

# ══════════════════════════════════════════════════════
# Report
# ══════════════════════════════════════════════════════

printf "\n${BOLD}══════════════════════════════════════${NC}\n"
printf "${BOLD}Results: ${GREEN}%d passed${NC}, ${RED}%d failed${NC}, ${YELLOW}%d skipped${NC}\n" "$PASS" "$FAIL" "$SKIP"

if [[ ${#FAILURES[@]} -gt 0 ]]; then
    printf "\n${RED}Failures:${NC}\n"
    for f in "${FAILURES[@]}"; do
        printf "  - %s\n" "$f"
    done
fi

printf "${BOLD}══════════════════════════════════════${NC}\n"

exit "$FAIL"
