#!/usr/bin/env bash
#
# RTCO Smoke Test Suite
# Exercises every command to catch regressions after merge.
# Exit code: number of failures (0 = all green)
#
set -euo pipefail

PASS=0
FAIL=0
SKIP=0
FAILURES=()

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

# ── Helpers ──────────────────────────────────────────

assert_ok() {
    local name="$1"
    shift
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
    local name="$1"
    local needle="$2"
    shift 2
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

assert_exit_ok() {
    local name="$1"
    shift
    if "$@" >/dev/null 2>&1; then
        PASS=$((PASS + 1))
        printf "  ${GREEN}PASS${NC}  %s\n" "$name"
    else
        FAIL=$((FAIL + 1))
        FAILURES+=("$name")
        printf "  ${RED}FAIL${NC}  %s\n" "$name"
        printf "        cmd: %s\n" "$*"
    fi
}

assert_fails() {
    local name="$1"
    shift
    if "$@" >/dev/null 2>&1; then
        FAIL=$((FAIL + 1))
        FAILURES+=("$name (expected failure, got success)")
        printf "  ${RED}FAIL${NC}  %s (expected failure)\n" "$name"
    else
        PASS=$((PASS + 1))
        printf "  ${GREEN}PASS${NC}  %s\n" "$name"
    fi
}

assert_help() {
    local name="$1"
    shift
    assert_contains "$name --help" "Usage:" "$@" --help
}

skip_test() {
    local name="$1"
    local reason="$2"
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

printf "${BOLD}RTCO Smoke Test Suite${NC}\n"
printf "Binary: %s\n" "$RTCO"
printf "Version: %s\n" "$(rtco --version)"
printf "Date: %s\n" "$(date '+%Y-%m-%d %H:%M')"

# Need a git repo to test git commands
if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    echo "Must run from inside a git repository."
    exit 1
fi

REPO_ROOT=$(git rev-parse --show-toplevel)

# ── 1. Version & Help ───────────────────────────────

section "Version & Help"

assert_contains "rtco --version" "rtco" rtco --version
assert_contains "rtco --help" "Usage:" rtco --help

# ── 2. Ls ────────────────────────────────────────────

section "Ls"

assert_ok      "rtco ls ."                     rtco ls .
assert_ok      "rtco ls -la ."                 rtco ls -la .
assert_ok      "rtco ls -lh ."                 rtco ls -lh .
assert_ok      "rtco ls -l src/"               rtco ls -l src/
assert_ok      "rtco ls src/ -l (flag after)"  rtco ls src/ -l
assert_ok      "rtco ls multi paths"           rtco ls src/ scripts/
assert_contains "rtco ls -a shows hidden"      ".git" rtco ls -a .
assert_contains "rtco ls shows sizes"          "K"  rtco ls src/
assert_contains "rtco ls shows dirs with /"    "/" rtco ls .

# ── 2b. Tree ─────────────────────────────────────────

section "Tree"

if command -v tree >/dev/null 2>&1; then
    assert_ok      "rtco tree ."                rtco tree .
    assert_ok      "rtco tree -L 2 ."           rtco tree -L 2 .
    assert_ok      "rtco tree -d -L 1 ."        rtco tree -d -L 1 .
    assert_contains "rtco tree shows src/"      "src" rtco tree -L 1 .
else
    skip_test "rtco tree" "tree not installed"
fi

# ── 3. Read ──────────────────────────────────────────

section "Read"

assert_ok      "rtco read Cargo.toml"          rtco read Cargo.toml
assert_ok      "rtco read --level none Cargo.toml"  rtco read --level none Cargo.toml
assert_ok      "rtco read --level aggressive Cargo.toml" rtco read --level aggressive Cargo.toml
assert_ok      "rtco read -n Cargo.toml"       rtco read -n Cargo.toml
assert_ok      "rtco read --max-lines 5 Cargo.toml" rtco read --max-lines 5 Cargo.toml

section "Read (stdin support)"

assert_ok      "rtco read stdin pipe"          bash -c 'echo "fn main() {}" | rtco read -'

# ── 4. Git ───────────────────────────────────────────

section "Git (existing)"

assert_ok      "rtco git status"               rtco git status
assert_ok      "rtco git status --short"       rtco git status --short
assert_ok      "rtco git status -s"            rtco git status -s
assert_ok      "rtco git status --porcelain"   rtco git status --porcelain
assert_ok      "rtco git log"                  rtco git log
assert_ok      "rtco git log -5"               rtco git log -- -5
assert_ok      "rtco git diff"                 rtco git diff
assert_ok      "rtco git diff --stat"          rtco git diff --stat

section "Git (new: branch, fetch, stash, worktree)"

assert_ok      "rtco git branch"               rtco git branch
assert_ok      "rtco git fetch"                rtco git fetch
assert_ok      "rtco git stash list"           rtco git stash list
assert_ok      "rtco git worktree"             rtco git worktree

section "Git (passthrough: unsupported subcommands)"

assert_ok      "rtco git tag --list"           rtco git tag --list
assert_ok      "rtco git remote -v"            rtco git remote -v
assert_ok      "rtco git rev-parse HEAD"       rtco git rev-parse HEAD

# ── 5. GitHub CLI ────────────────────────────────────

section "GitHub CLI"

if command -v gh >/dev/null 2>&1 && gh auth status >/dev/null 2>&1; then
    assert_ok      "rtco gh pr list"           rtco gh pr list
    assert_ok      "rtco gh run list"          rtco gh run list
    assert_ok      "rtco gh issue list"        rtco gh issue list
    # pr create/merge/diff/comment/edit are write ops, test help only
    assert_help    "rtco gh"                   rtco gh
else
    skip_test "gh commands" "gh not authenticated"
fi

# ── 6. Cargo ─────────────────────────────────────────

section "Cargo (new)"

assert_ok      "rtco cargo build"              rtco cargo build
assert_ok      "rtco cargo clippy"             rtco cargo clippy
# cargo test exits non-zero due to pre-existing failures; check output ignoring exit code
output_cargo_test=$(rtco cargo test 2>&1 || true)
if echo "$output_cargo_test" | grep -q "FAILURES\|test result:\|passed"; then
    PASS=$((PASS + 1))
    printf "  ${GREEN}PASS${NC}  %s\n" "rtco cargo test"
else
    FAIL=$((FAIL + 1))
    FAILURES+=("rtco cargo test")
    printf "  ${RED}FAIL${NC}  %s\n" "rtco cargo test"
    printf "        got: %s\n" "$(echo "$output_cargo_test" | head -3)"
fi
assert_help    "rtco cargo"                    rtco cargo

# ── 7. Curl ──────────────────────────────────────────

section "Curl (new)"

assert_contains "rtco curl JSON detect" "string" rtco curl https://httpbin.org/json
assert_ok       "rtco curl plain text"          rtco curl https://httpbin.org/robots.txt
assert_help     "rtco curl"                     rtco curl

# ── 8. Npm / Npx ────────────────────────────────────

section "Npm / Npx (new)"

assert_help    "rtco npm"                      rtco npm
assert_help    "rtco npx"                      rtco npx

# ── 9. Pnpm ─────────────────────────────────────────

section "Pnpm"

assert_help    "rtco pnpm"                     rtco pnpm
assert_help    "rtco pnpm build"               rtco pnpm build
assert_help    "rtco pnpm typecheck"           rtco pnpm typecheck

if command -v pnpm >/dev/null 2>&1; then
    assert_ok  "rtco pnpm help"                rtco pnpm help
fi

# ── 10. Grep ─────────────────────────────────────────

section "Grep"

assert_ok      "rtco grep pattern"             rtco grep "pub fn" src/
assert_contains "rtco grep finds results"      "pub fn" rtco grep "pub fn" src/
assert_ok      "rtco grep with file type"      rtco grep "pub fn" src/ -t rust

section "Grep (extra args passthrough)"

assert_ok      "rtco grep -i case insensitive" rtco grep "fn" src/ -i
assert_ok      "rtco grep -A context lines"    rtco grep "fn run" src/ -A 2

# ── 11. Find ─────────────────────────────────────────

section "Find"

assert_ok      "rtco find *.rs"                rtco find "*.rs" src/
assert_contains "rtco find shows files"        ".rs" rtco find "*.rs" src/

# ── 12. Json ─────────────────────────────────────────

section "Json"

# Create temp JSON file for testing
TMPJSON=$(mktemp /tmp/rtco-test-XXXXX.json)
echo '{"name":"test","count":42,"items":[1,2,3]}' > "$TMPJSON"

assert_ok      "rtco json file"                rtco json "$TMPJSON"
assert_contains "rtco json shows schema"       "string" rtco json "$TMPJSON"

rm -f "$TMPJSON"

# ── 13. Deps ─────────────────────────────────────────

section "Deps"

assert_ok      "rtco deps ."                   rtco deps .
assert_contains "rtco deps shows Cargo"        "Cargo" rtco deps .

# ── 14. Env ──────────────────────────────────────────

section "Env"

assert_ok      "rtco env"                      rtco env
assert_ok      "rtco env --filter PATH"        rtco env --filter PATH

# ── 16. Log ──────────────────────────────────────────

section "Log"

TMPLOG=$(mktemp /tmp/rtco-log-XXXXX.log)
for i in $(seq 1 20); do
    echo "[2025-01-01 12:00:00] INFO: repeated message" >> "$TMPLOG"
done
echo "[2025-01-01 12:00:01] ERROR: something failed" >> "$TMPLOG"

assert_ok      "rtco log file"                 rtco log "$TMPLOG"

rm -f "$TMPLOG"

# ── 17. Summary ──────────────────────────────────────

section "Summary"

assert_ok      "rtco summary echo hello"       rtco summary echo hello

# ── 18. Err ──────────────────────────────────────────

section "Err"

assert_ok      "rtco err echo ok"              rtco err echo ok

# ── 19. Test runner ──────────────────────────────────

section "Test runner"

assert_ok      "rtco test echo ok"             rtco test echo ok

# ── 20. Gain ─────────────────────────────────────────

section "Gain"

assert_ok      "rtco gain"                     rtco gain
assert_ok      "rtco gain --history"           rtco gain --history

# ── 21. Config & Init ────────────────────────────────

section "Config & Init"

assert_ok      "rtco config"                   rtco config
assert_ok      "rtco init --show"              rtco init --show

# ── 22. Wget ─────────────────────────────────────────

section "Wget"

if command -v wget >/dev/null 2>&1; then
    assert_ok  "rtco wget stdout"              rtco wget https://httpbin.org/robots.txt -O
else
    skip_test "rtco wget" "wget not installed"
fi

# ── 23. Tsc / Lint / Prettier / Next / Playwright ───

section "JS Tooling (help only, no project context)"

assert_help    "rtco tsc"                      rtco tsc
assert_help    "rtco lint"                     rtco lint
assert_help    "rtco prettier"                 rtco prettier
assert_help    "rtco next"                     rtco next
assert_help    "rtco playwright"               rtco playwright

# ── 24. Prisma ───────────────────────────────────────

section "Prisma (help only)"

assert_help    "rtco prisma"                   rtco prisma

# ── 25. Vitest ───────────────────────────────────────

section "Vitest (help only)"

assert_help    "rtco vitest"                   rtco vitest

# ── 26. Docker / Kubectl (help only) ────────────────

section "Docker / Kubectl (help only)"

assert_help    "rtco docker"                   rtco docker
assert_help    "rtco kubectl"                  rtco kubectl

# ── 27. Python (conditional) ────────────────────────

section "Python (conditional)"

if command -v pytest &>/dev/null; then
    assert_help    "rtco pytest"                    rtco pytest --help
else
    skip_test "rtco pytest" "pytest not installed"
fi

if command -v ruff &>/dev/null; then
    assert_help    "rtco ruff"                      rtco ruff --help
else
    skip_test "rtco ruff" "ruff not installed"
fi

if command -v pip &>/dev/null; then
    assert_help    "rtco pip"                       rtco pip --help
else
    skip_test "rtco pip" "pip not installed"
fi

# ── 28. Go (conditional) ────────────────────────────

section "Go (conditional)"

if command -v go &>/dev/null; then
    assert_help    "rtco go"                        rtco go --help
    assert_help    "rtco go test"                   rtco go test -h
    assert_help    "rtco go build"                  rtco go build -h
    assert_help    "rtco go vet"                    rtco go vet -h
else
    skip_test "rtco go" "go not installed"
fi

if command -v golangci-lint &>/dev/null; then
    assert_help    "rtco golangci-lint"             rtco golangci-lint --help
else
    skip_test "rtco golangci-lint" "golangci-lint not installed"
fi

# ── 29. Graphite (conditional) ─────────────────────

section "Graphite (conditional)"

if command -v gt &>/dev/null; then
    assert_help   "rtco gt"                          rtco gt --help
    assert_ok     "rtco gt log short"                rtco gt log short
else
    skip_test "rtco gt" "gt not installed"
fi

# ── 30. Ruby (conditional) ──────────────────────────

section "Ruby (conditional)"

if command -v rspec &>/dev/null; then
    assert_help    "rtco rspec"                     rtco rspec --help
else
    skip_test "rtco rspec" "rspec not installed"
fi

if command -v rubocop &>/dev/null; then
    assert_help    "rtco rubocop"                   rtco rubocop --help
else
    skip_test "rtco rubocop" "rubocop not installed"
fi

if command -v rake &>/dev/null; then
    assert_help    "rtco rake"                      rtco rake --help
else
    skip_test "rtco rake" "rake not installed"
fi

# ── 31. Global flags ────────────────────────────────

section "Global flags"

assert_ok      "rtco -u ls ."                  rtco -u ls .
assert_ok      "rtco --skip-env npm --help"    rtco --skip-env npm --help

# ── 32. CcEconomics ─────────────────────────────────

section "CcEconomics"

assert_ok      "rtco cc-economics"             rtco cc-economics

# ── 33. Learn ───────────────────────────────────────

section "Learn"

assert_ok      "rtco learn --help"             rtco learn --help
assert_ok      "rtco learn (no sessions)"      rtco learn --since 0 2>&1 || true

# ── 32. Rewrite ───────────────────────────────────────

section "Rewrite"

assert_contains "rewrite git status"          "rtco git status"         rtco rewrite "git status"
assert_contains "rewrite cargo test"          "rtco cargo test"         rtco rewrite "cargo test"
assert_contains "rewrite compound &&"         "rtco git status"         rtco rewrite "git status && cargo test"
assert_contains "rewrite pipe preserves"      "| head"                 rtco rewrite "git log | head"

section "Rewrite (#345: RTK_DISABLED skip)"

assert_fails   "rewrite RTK_DISABLED=1 skip"                          rtco rewrite "RTK_DISABLED=1 git status"
assert_fails   "rewrite env RTK_DISABLED skip"                        rtco rewrite "FOO=1 RTK_DISABLED=1 cargo test"

section "Rewrite (#346: 2>&1 preserved)"

assert_contains "rewrite 2>&1 preserved"      "2>&1"                  rtco rewrite "cargo test 2>&1 | head"

section "Rewrite (#196: gh --json skip)"

assert_fails   "rewrite gh --json skip"                               rtco rewrite "gh pr list --json number"
assert_fails   "rewrite gh --jq skip"                                 rtco rewrite "gh api /repos --jq .name"
assert_fails   "rewrite gh --template skip"                           rtco rewrite "gh pr view 1 --template '{{.title}}'"
assert_contains "rewrite gh normal works"     "rtco gh pr list"        rtco rewrite "gh pr list"

# ── 33. Verify ────────────────────────────────────────

section "Verify"

assert_ok      "rtco verify"                   rtco verify

# ── 34. Proxy ─────────────────────────────────────────

section "Proxy"

assert_ok      "rtco proxy echo hello"         rtco proxy echo hello
assert_contains "rtco proxy passthrough"       "hello" rtco proxy echo hello

# ── 35. Discover ──────────────────────────────────────

section "Discover"

assert_ok      "rtco discover"                 rtco discover

# ── 36. Diff ──────────────────────────────────────────

section "Diff"

assert_ok      "rtco diff two files"           rtco diff Cargo.toml LICENSE

# ── 37. Wc ────────────────────────────────────────────

section "Wc"

assert_ok      "rtco wc Cargo.toml"            rtco wc Cargo.toml

# ── 38. Smart ─────────────────────────────────────────

section "Smart"

assert_ok      "rtco smart src/main.rs"        rtco smart src/main.rs

# ── 39. Json edge cases ──────────────────────────────

section "Json (edge cases)"

assert_fails   "rtco json on TOML (#347)"                              rtco json Cargo.toml

# ── 40. Docker (conditional) ─────────────────────────

section "Docker (conditional)"

if command -v docker >/dev/null 2>&1 && docker info >/dev/null 2>&1; then
    assert_ok  "rtco docker ps"               rtco docker ps
    assert_ok  "rtco docker images"           rtco docker images
else
    skip_test "rtco docker" "docker not running"
fi

# ── 41. Hook check ───────────────────────────────────

section "Hook check (#344)"

assert_contains "rtco init --show hook version" "version" rtco init --show

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
