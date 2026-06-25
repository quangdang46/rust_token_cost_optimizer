#!/usr/bin/env bash
set -e

# Use local release build if available, otherwise fall back to installed rtco
if [ -f "./target/release/rtco" ]; then
  RTCO="$(cd "$(dirname ./target/release/rtco)" && pwd)/$(basename ./target/release/rtco)"
elif command -v rtco &> /dev/null; then
  RTCO="$(command -v rtco)"
else
  echo "Error: rtco not found. Run 'cargo build --release' or install rtco."
  exit 1
fi
BENCH_DIR="$(pwd)/scripts/benchmark"
RTCO_ROOT="$(pwd)"

if [ -z "$CI" ]; then
  rm -rf "$BENCH_DIR"
  mkdir -p "$BENCH_DIR/unix" "$BENCH_DIR/rtco" "$BENCH_DIR/diff"
fi

safe_name() {
  echo "$1" | tr ' /' '_-' | tr -cd 'a-zA-Z0-9_-'
}

count_tokens() {
  local input="$1"
  local len=${#input}
  echo $(( (len + 3) / 4 ))
}

TOTAL_UNIX=0
TOTAL_RTCO=0
TOTAL_TESTS=0
GOOD_TESTS=0
FAIL_TESTS=0
WARN_TESTS=0
NEGATIVE_TESTS=0

bench() {
  local name="$1"
  local unix_cmd="$2"
  local rtco_cmd="$3"

  unix_out=$(eval "$unix_cmd" 2>/dev/null || true)
  rtco_out=$(eval "$rtco_cmd" 2>/dev/null || true)

  unix_tokens=$(count_tokens "$unix_out")
  rtco_tokens=$(count_tokens "$rtco_out")

  TOTAL_TESTS=$((TOTAL_TESTS + 1))

  local icon=""
  local tag=""

  if [ -z "$rtco_out" ] && [ -n "$unix_out" ]; then
    icon="❌"
    tag="FAIL"
    FAIL_TESTS=$((FAIL_TESTS + 1))
    TOTAL_UNIX=$((TOTAL_UNIX + unix_tokens))
    TOTAL_RTCO=$((TOTAL_RTCO + unix_tokens))
  elif [ "$rtco_tokens" -gt "$unix_tokens" ] && [ "$unix_tokens" -gt 0 ]; then
    icon="🔴"
    tag="NEG"
    NEGATIVE_TESTS=$((NEGATIVE_TESTS + 1))
    TOTAL_UNIX=$((TOTAL_UNIX + unix_tokens))
    TOTAL_RTCO=$((TOTAL_RTCO + rtco_tokens))
  elif [ "$unix_tokens" -gt 0 ] && [ "$rtco_tokens" -eq "$unix_tokens" ]; then
    icon="⚠️"
    tag="WARN"
    WARN_TESTS=$((WARN_TESTS + 1))
    TOTAL_UNIX=$((TOTAL_UNIX + unix_tokens))
    TOTAL_RTCO=$((TOTAL_RTCO + rtco_tokens))
  elif [ "$unix_tokens" -gt 0 ]; then
    local savings=$(( (unix_tokens - rtco_tokens) * 100 / unix_tokens ))
    if [ "$savings" -lt 60 ]; then
      icon="⚠️"
      tag="WARN"
      WARN_TESTS=$((WARN_TESTS + 1))
    else
      icon="✅"
      tag="GOOD"
      GOOD_TESTS=$((GOOD_TESTS + 1))
    fi
    TOTAL_UNIX=$((TOTAL_UNIX + unix_tokens))
    TOTAL_RTCO=$((TOTAL_RTCO + rtco_tokens))
  else
    icon="⏭️"
    tag="SKIP"
    WARN_TESTS=$((WARN_TESTS + 1))
  fi

  if [ "$tag" = "FAIL" ]; then
    printf "%s %-24s │ %-40s │ %-40s │ %6d → %6s (--)\n" \
      "$icon" "$name" "$unix_cmd" "$rtco_cmd" "$unix_tokens" "-"
  else
    if [ "$unix_tokens" -gt 0 ]; then
      local pct=$(( (unix_tokens - rtco_tokens) * 100 / unix_tokens ))
    else
      local pct=0
    fi
    printf "%s %-24s │ %-40s │ %-40s │ %6d → %6d (%+d%%)\n" \
      "$icon" "$name" "$unix_cmd" "$rtco_cmd" "$unix_tokens" "$rtco_tokens" "$pct"
  fi

  if [ -z "$CI" ]; then
    local filename=$(safe_name "$name")
    local prefix="GOOD"
    [ "$tag" = "FAIL" ] && prefix="FAIL"
    [ "$tag" = "NEG" ] && prefix="NEG"
    [ "$tag" = "WARN" ] && prefix="WARN"
    [ "$tag" = "SKIP" ] && prefix="SKIP"

    local ts=$(date "+%d/%m/%Y %H:%M:%S")

    printf "# %s\n> %s\n\n\`\`\`bash\n$ %s\n\`\`\`\n\n\`\`\`\n%s\n\`\`\`\n" \
      "$name" "$ts" "$unix_cmd" "$unix_out" > "$BENCH_DIR/unix/${filename}.md"

    printf "# %s\n> %s\n\n\`\`\`bash\n$ %s\n\`\`\`\n\n\`\`\`\n%s\n\`\`\`\n" \
      "$name" "$ts" "$rtco_cmd" "$rtco_out" > "$BENCH_DIR/rtco/${filename}.md"

    {
      echo "# Diff: $name"
      echo "> $ts"
      echo ""
      echo "| Metric | Unix | RTCO |"
      echo "|--------|------|-----|"
      echo "| Tokens | $unix_tokens | $rtco_tokens |"
      echo ""
      echo "## Unix"
      echo "\`\`\`"
      echo "$unix_out"
      echo "\`\`\`"
      echo ""
      echo "## RTCO"
      echo "\`\`\`"
      echo "$rtco_out"
      echo "\`\`\`"
    } > "$BENCH_DIR/diff/${prefix}-${filename}.md"
  fi
}

section() {
  echo ""
  echo "── $1 ──"
}

# ═══════════════════════════════════════════
echo "RTCO Benchmark"
echo "═══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════"
printf "   %-24s │ %-40s │ %-40s │ %s\n" "TEST" "SHELL" "RTCO" "TOKENS"
echo "───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────"

# ===================
# ls
# ===================
section "ls"
bench "ls" "ls -la" "$RTCO ls"
bench "ls src/" "ls -la src/" "$RTCO ls src/"
bench "ls -l src/" "ls -l src/" "$RTCO ls -l src/"
bench "ls -la src/" "ls -la src/" "$RTCO ls -la src/"
bench "ls -lh src/" "ls -lh src/" "$RTCO ls -lh src/"
bench "ls src/ -l" "ls -l src/" "$RTCO ls src/ -l"
bench "ls -a" "ls -la" "$RTCO ls -a"
bench "ls multi" "ls -la src/ scripts/" "$RTCO ls src/ scripts/"

# ===================
# tree
# ===================
if command -v tree &>/dev/null; then
  section "tree"
  bench "tree" "tree -L 2" "$RTCO tree -L 2"
  bench "tree src/" "tree src/ -L 2" "$RTCO tree src/ -L 2"
else
  echo ""
  echo "⏭️  tree (not installed, skipped)"
fi

# ===================
# read
# ===================
section "read"
bench "read" "cat src/main.rs" "$RTCO read src/main.rs"
bench "read -l minimal" "cat src/main.rs" "$RTCO read src/main.rs -l minimal"
bench "read -l aggressive" "cat src/main.rs" "$RTCO read src/main.rs -l aggressive"
bench "read -n" "cat -n src/main.rs" "$RTCO read src/main.rs -n"

# ===================
# find
# ===================
section "find"
bench "find *" "find . -type f" "$RTCO find '*'"
bench "find *.rs" "find . -name '*.rs' -type f" "$RTCO find '*.rs'"
bench "find --max 10" "find . -not -path './target/*' -not -path './.git/*' -type f | head -10" "$RTCO find '*' --max 10"
bench "find --max 100" "find . -not -path './target/*' -not -path './.git/*' -type f | head -100" "$RTCO find '*' --max 100"

# ===================
# git
# ===================
section "git"
bench "git status" "git status" "$RTCO git status"
bench "git log -n 10" "git log -10" "$RTCO git log -n 10"
bench "git log -n 5" "git log -5" "$RTCO git log -n 5"
bench "git diff" "git diff HEAD~1 2>/dev/null || echo ''" "$RTCO git diff HEAD~1"
bench "git show" "git show HEAD --stat 2>/dev/null || true" "$RTCO git show HEAD --stat"

# ===================
# grep
# ===================
section "grep"
bench "grep fn" "grep -rn 'fn ' src/ || true" "$RTCO grep 'fn ' src/"
bench "grep struct" "grep -rn 'struct ' src/ || true" "$RTCO grep 'struct ' src/"
bench "grep -l 40" "grep -rn 'fn ' src/ || true" "$RTCO grep 'fn ' src/ -l 40"
bench "grep -c" "grep -ron 'fn ' src/ || true" "$RTCO grep 'fn ' src/ -c"

# ===================
# json
# ===================
section "json"
cat > /tmp/rtco_bench.json << 'JSONEOF'
{
  "name": "rtco",
  "version": "0.2.1",
  "config": {
    "debug": false,
    "max_depth": 10,
    "filters": ["node_modules", "target", ".git"]
  },
  "dependencies": {
    "serde": "1.0",
    "clap": "4.0",
    "anyhow": "1.0"
  }
}
JSONEOF
bench "json" "cat /tmp/rtco_bench.json" "$RTCO json /tmp/rtco_bench.json"
bench "json -d 2" "cat /tmp/rtco_bench.json" "$RTCO json /tmp/rtco_bench.json -d 2"
rm -f /tmp/rtco_bench.json

# ===================
# deps
# ===================
section "deps"
bench "deps" "cat Cargo.toml" "$RTCO deps"

# ===================
# env
# ===================
section "env"
bench "env" "env" "$RTCO env"
bench "env -f PATH" "env | grep PATH" "$RTCO env -f PATH"
bench "env --show-all" "env" "$RTCO env --show-all"

# ===================
# err
# ===================
section "err"
if command -v cargo &>/dev/null; then
  bench "err cargo build" "cargo build 2>&1 || true" "$RTCO err cargo build 2>&1"
else
  echo "⏭️  err cargo build (cargo not in PATH, skipped)"
fi

# ===================
# test
# ===================
section "test"
if command -v cargo &>/dev/null; then
  bench "test cargo test" "cargo test 2>&1 || true" "$RTCO test cargo test 2>&1"
else
  echo "⏭️  test cargo test (cargo not in PATH, skipped)"
fi

# ===================
# log
# ===================
section "log"
LOG_FILE="/tmp/rtco_bench_sample.log"
cat > "$LOG_FILE" << 'LOGEOF'
2024-01-15 10:00:01 INFO  Application started
2024-01-15 10:00:02 INFO  Loading configuration
2024-01-15 10:00:03 ERROR Connection failed: timeout
2024-01-15 10:00:04 ERROR Connection failed: timeout
2024-01-15 10:00:05 ERROR Connection failed: timeout
2024-01-15 10:00:06 ERROR Connection failed: timeout
2024-01-15 10:00:07 ERROR Connection failed: timeout
2024-01-15 10:00:08 WARN  Retrying connection
2024-01-15 10:00:09 INFO  Connection established
2024-01-15 10:00:10 INFO  Processing request
2024-01-15 10:00:11 INFO  Processing request
2024-01-15 10:00:12 INFO  Processing request
2024-01-15 10:00:13 INFO  Request completed
LOGEOF
bench "log" "cat $LOG_FILE" "$RTCO log $LOG_FILE"
rm -f "$LOG_FILE"

# ===================
# summary
# ===================
section "summary"
if command -v cargo &>/dev/null; then
  bench "summary cargo --help" "cargo --help" "$RTCO summary cargo --help"
else
  echo "⏭️  summary cargo --help (cargo not in PATH, skipped)"
fi
if command -v rustc &>/dev/null; then
  bench "summary rustc --help" "rustc --help 2>/dev/null || echo 'rustc not found'" "$RTCO summary rustc --help"
else
  echo "⏭️  summary rustc --help (rustc not in PATH, skipped)"
fi

# ===================
# cargo
# ===================
section "cargo"
if command -v cargo &>/dev/null; then
  bench "cargo build" "cargo build 2>&1 || true" "$RTCO cargo build 2>&1"
  bench "cargo test" "cargo test 2>&1 || true" "$RTCO cargo test 2>&1"
  bench "cargo clippy" "cargo clippy 2>&1 || true" "$RTCO cargo clippy 2>&1"
  bench "cargo check" "cargo check 2>&1 || true" "$RTCO cargo check 2>&1"
else
  echo "⏭️  cargo build/test/clippy/check (cargo not in PATH, skipped)"
fi

# ===================
# smart
# ===================
section "smart"
bench "smart main.rs" "cat src/main.rs" "$RTCO smart src/main.rs"

# ===================
# wc
# ===================
section "wc"
bench "wc" "wc Cargo.toml src/main.rs" "$RTCO wc Cargo.toml src/main.rs"

# ===================
# curl
# ===================
section "curl"
if command -v curl &> /dev/null; then
  bench "curl json" "curl -s https://httpbin.org/json" "$RTCO curl https://httpbin.org/json"
  bench "curl text" "curl -s https://httpbin.org/robots.txt" "$RTCO curl https://httpbin.org/robots.txt"
fi

# ===================
# wget
# ===================
if command -v wget &> /dev/null; then
  section "wget"
  bench "wget" "wget -qO- https://httpbin.org/json" "$RTCO wget https://httpbin.org/json"
  rm -f json 2>/dev/null
fi

# ===================
# npm (standalone — does not require package.json)
# ===================
if command -v npm &> /dev/null; then
  section "npm"
  bench "npm list" "npm list -g --depth 0 2>&1 || true" "$RTCO npm list -g --depth 0"
fi

# ===================
# Modern JavaScript Stack (skip si pas de package.json)
# ===================
if [ -f "package.json" ]; then
  section "modern JS stack"

  if command -v tsc &> /dev/null || [ -f "node_modules/.bin/tsc" ]; then
    bench "tsc" "tsc --noEmit 2>&1 || true" "$RTCO tsc --noEmit 2>&1"
  fi

  if command -v prettier &> /dev/null || [ -f "node_modules/.bin/prettier" ]; then
    bench "prettier --check" "prettier --check . 2>&1 || true" "$RTCO prettier --check ."
  fi

  if command -v eslint &> /dev/null || [ -f "node_modules/.bin/eslint" ]; then
    bench "lint" "eslint . 2>&1 || true" "$RTCO lint ."
  fi

  if [ -f "next.config.js" ] || [ -f "next.config.mjs" ] || [ -f "next.config.ts" ]; then
    if command -v next &> /dev/null || [ -f "node_modules/.bin/next" ]; then
      bench "next build" "next build 2>&1 || true" "$RTCO next build"
    fi
  fi

  if [ -f "playwright.config.ts" ] || [ -f "playwright.config.js" ]; then
    if command -v playwright &> /dev/null || [ -f "node_modules/.bin/playwright" ]; then
      bench "playwright test" "playwright test 2>&1 || true" "$RTCO playwright test"
    fi
  fi

  if [ -f "prisma/schema.prisma" ]; then
    if command -v prisma &> /dev/null || [ -f "node_modules/.bin/prisma" ]; then
      bench "prisma generate" "prisma generate 2>&1 || true" "$RTCO prisma generate"
    fi
  fi

  if command -v vitest &> /dev/null || [ -f "node_modules/.bin/vitest" ]; then
    bench "vitest" "vitest run --reporter=json 2>&1 || true" "$RTCO vitest"
  fi

  if command -v pnpm &> /dev/null; then
    bench "pnpm list" "pnpm list --depth 0 2>&1 || true" "$RTCO pnpm list --depth 0"
    bench "pnpm outdated" "pnpm outdated 2>&1 || true" "$RTCO pnpm outdated"
  fi
fi

# ===================
# gh (skip si pas dispo ou pas dans un repo)
# ===================
if command -v gh &> /dev/null && git rev-parse --git-dir &> /dev/null && gh auth status &> /dev/null; then
  section "gh"
  bench "gh pr list" "gh pr list 2>&1 || true" "$RTCO gh pr list"
  bench "gh run list" "gh run list 2>&1 || true" "$RTCO gh run list"
fi

# ===================
# glab
# ===================
if command -v glab &> /dev/null; then
  section "glab"
  bench "glab mr list" "glab mr list 2>&1 || true" "$RTCO glab mr list"
  bench "glab issue list" "glab issue list 2>&1 || true" "$RTCO glab issue list"
fi

# ===================
# gt (Graphite)
# ===================
if command -v gt &> /dev/null; then
  section "gt"
  bench "gt log" "gt log 2>&1 || true" "$RTCO gt log"
fi

# ===================
# docker
# ===================
if command -v docker &> /dev/null; then
  section "docker"
  bench "docker ps" "docker ps 2>/dev/null || true" "$RTCO docker ps"
  bench "docker images" "docker images 2>/dev/null || true" "$RTCO docker images"
fi

# ===================
# kubectl
# ===================
if command -v kubectl &> /dev/null; then
  section "kubectl"
  bench "kubectl pods" "kubectl get pods 2>/dev/null || true" "$RTCO kubectl pods"
  bench "kubectl services" "kubectl get services 2>/dev/null || true" "$RTCO kubectl services"
fi

# ===================
# Python (avec fixtures temporaires)
# ===================
if command -v python3 &> /dev/null && command -v ruff &> /dev/null && command -v pytest &> /dev/null; then
  section "python"

  PYTHON_FIXTURE=$(mktemp -d)
  cd "$PYTHON_FIXTURE"

  cat > pyproject.toml << 'PYEOF'
[project]
name = "rtco-bench"
version = "0.1.0"

[tool.ruff]
line-length = 88
PYEOF

  cat > sample.py << 'PYEOF'
import os
import sys
import json


def process_data(x):
    if x == None:  # E711: comparison to None
        return []
    result = []
    for i in range(len(x)):  # C416: unnecessary list comprehension
        result.append(x[i] * 2)
    return result

def unused_function():  # F841: local variable assigned but never used
    temp = 42
    return None
PYEOF

  cat > test_sample.py << 'PYEOF'
from sample import process_data

def test_process_data():
    assert process_data([1, 2, 3]) == [2, 4, 6]

def test_process_data_none():
    assert process_data(None) == []
PYEOF

  bench "ruff check" "ruff check . 2>&1 || true" "$RTCO ruff check ."
  bench "pytest" "pytest -v 2>&1 || true" "$RTCO pytest -v"

  if command -v pip &>/dev/null; then
    bench "pip list" "pip list 2>&1 || true" "$RTCO pip list"
  fi

  if command -v mypy &>/dev/null; then
    bench "mypy" "mypy sample.py 2>&1 || true" "$RTCO mypy sample.py"
  fi

  cd "$RTCO_ROOT"
  rm -rf "$PYTHON_FIXTURE"
fi

# ===================
# Go (avec fixtures temporaires)
# ===================
if command -v go &> /dev/null && command -v golangci-lint &> /dev/null; then
  section "go"

  GO_FIXTURE=$(mktemp -d)
  cd "$GO_FIXTURE"

  cat > go.mod << 'GOEOF'
module bench

go 1.21
GOEOF

  cat > main.go << 'GOEOF'
package main

import "fmt"

func Add(a, b int) int {
    return a + b
}

func Multiply(a, b int) int {
    return a * b
}

func main() {
    fmt.Println(Add(2, 3))
    fmt.Println(Multiply(4, 5))
}
GOEOF

  cat > main_test.go << 'GOEOF'
package main

import "testing"

func TestAdd(t *testing.T) {
    result := Add(2, 3)
    if result != 5 {
        t.Errorf("Add(2, 3) = %d; want 5", result)
    }
}

func TestMultiply(t *testing.T) {
    result := Multiply(4, 5)
    if result != 20 {
        t.Errorf("Multiply(4, 5) = %d; want 20", result)
    }
}
GOEOF

  bench "golangci-lint" "golangci-lint run 2>&1 || true" "$RTCO golangci-lint run"
  bench "go test" "go test -v 2>&1 || true" "$RTCO go test -v"
  bench "go build" "go build ./... 2>&1 || true" "$RTCO go build ./..."
  bench "go vet" "go vet ./... 2>&1 || true" "$RTCO go vet ./..."

  cd "$RTCO_ROOT"
  rm -rf "$GO_FIXTURE"
fi

# ===================
# Ruby
# ===================
if command -v ruby &> /dev/null; then
  section "ruby"
  if command -v rake &>/dev/null; then
    bench "rake -T" "rake -T 2>&1 || true" "$RTCO rake -T"
  fi
  if command -v rubocop &>/dev/null; then
    bench "rubocop" "rubocop --format simple 2>&1 || true" "$RTCO rubocop --format simple"
  fi
  if command -v rspec &>/dev/null; then
    bench "rspec --dry-run" "rspec --dry-run 2>&1 || true" "$RTCO rspec --dry-run"
  fi
fi

# ===================
# dotnet
# ===================
if command -v dotnet &> /dev/null; then
  section "dotnet"
  bench "dotnet --info" "dotnet --info 2>&1 || true" "$RTCO dotnet --info"
fi

# ===================
# aws
# ===================
if command -v aws &> /dev/null; then
  section "aws"
  bench "aws --version" "aws --version 2>&1 || true" "$RTCO aws --version"
fi

# ===================
# psql
# ===================
if command -v psql &> /dev/null; then
  section "psql"
  bench "psql --version" "psql --version 2>&1 || true" "$RTCO psql --version"
fi

# ===================
# rewrite (verify rewrite works with and without quotes)
# ===================
section "rewrite"

bench_rewrite() {
  local name="$1"
  local cmd="$2"
  local expected="$3"

  result=$(eval "$cmd" 2>&1 || true)

  TOTAL_TESTS=$((TOTAL_TESTS + 1))

  if [ "$result" = "$expected" ]; then
    printf "✅ %-24s │ %-40s │ %s\n" "$name" "$cmd" "$result"
    GOOD_TESTS=$((GOOD_TESTS + 1))
  else
    printf "❌ %-24s │ %-40s │ got: %s (expected: %s)\n" "$name" "$cmd" "$result" "$expected"
    FAIL_TESTS=$((FAIL_TESTS + 1))
  fi
}

bench_rewrite "rewrite quoted"       "$RTCO rewrite 'git status'"     "rtco git status"
bench_rewrite "rewrite unquoted"     "$RTCO rewrite git status"       "rtco git status"
bench_rewrite "rewrite ls -al"       "$RTCO rewrite ls -al"           "rtco ls -al"
bench_rewrite "rewrite npm exec"     "$RTCO rewrite npm exec"         "rtco npm exec"
bench_rewrite "rewrite cargo test"   "$RTCO rewrite cargo test"       "rtco cargo test"
bench_rewrite "rewrite compound"     "$RTCO rewrite 'cargo test && git push'" "rtco cargo test && rtco git push"

# ===================
# Summary
# ===================
echo ""
echo "═══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════"

if [ "$TOTAL_TESTS" -gt 0 ]; then
  GOOD_PCT=$((GOOD_TESTS * 100 / TOTAL_TESTS))
  if [ "$TOTAL_UNIX" -gt 0 ]; then
    TOTAL_SAVED=$((TOTAL_UNIX - TOTAL_RTCO))
    TOTAL_SAVE_PCT=$((TOTAL_SAVED * 100 / TOTAL_UNIX))
  else
    TOTAL_SAVED=0
    TOTAL_SAVE_PCT=0
  fi

  echo ""
  echo "  ✅ $GOOD_TESTS good  ⚠️ $WARN_TESTS warn  🔴 $NEGATIVE_TESTS negative  ❌ $FAIL_TESTS fail    $GOOD_TESTS/$TOTAL_TESTS ($GOOD_PCT%)"
  echo "  Tokens: $TOTAL_UNIX → $TOTAL_RTCO  (-$TOTAL_SAVE_PCT%)"
  echo ""

  if [ -z "$CI" ]; then
    echo "  Debug: $BENCH_DIR/{unix,rtco,diff}/"
  fi
  echo ""

  EXIT_CODE=0

  if [ "$NEGATIVE_TESTS" -gt 0 ]; then
    echo "  BENCHMARK FAILED: $NEGATIVE_TESTS filter(s) produced more tokens than raw output"
    EXIT_CODE=1
  fi

  if [ "$FAIL_TESTS" -gt 0 ]; then
    echo "  BENCHMARK FAILED: $FAIL_TESTS filter(s) returned empty output"
    EXIT_CODE=1
  fi

  if [ "$GOOD_PCT" -lt 60 ] && [ "$EXIT_CODE" -eq 0 ]; then
    echo "  WARNING: $GOOD_PCT% good (target 60%)"
  fi

  exit $EXIT_CODE
fi
