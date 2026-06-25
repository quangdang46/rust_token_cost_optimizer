---
title: Token Savings Analytics
description: Measure and analyze your RTCO token savings with rtco gain
sidebar:
  order: 1
---

# Token Savings Analytics

`rtco gain` shows how many tokens RTCO has saved across all your commands, with daily, weekly, and monthly breakdowns.

## Quick reference

```bash
# Default summary
rtco gain

# Temporal breakdowns
rtco gain --daily          # all days since tracking started
rtco gain --weekly         # aggregated by week
rtco gain --monthly        # aggregated by month
rtco gain --all            # all breakdowns at once

# Classic flags
rtco gain --graph          # ASCII graph, last 30 days
rtco gain --history        # last 10 commands
rtco gain --quota          # monthly quota savings estimate (default tier: 20x)
rtco gain --quota -t pro   # use pro tier token budget for estimate

# Export
rtco gain --all --format json > savings.json
rtco gain --all --format csv  > savings.csv
```

## Daily breakdown

```bash
rtco gain --daily
```

```
📅 Daily Breakdown (3 days)
════════════════════════════════════════════════════════════════
Date            Cmds      Input     Output      Saved   Save%
────────────────────────────────────────────────────────────────
2026-01-28        89     380.9K      26.7K     355.8K   93.4%
2026-01-29       102     894.5K      32.4K     863.7K   96.6%
2026-01-30         5        749         55        694   92.7%
────────────────────────────────────────────────────────────────
TOTAL            196       1.3M      59.2K       1.2M   95.6%
```

- **Cmds**: RTCO commands executed
- **Input**: Estimated tokens from raw command output
- **Output**: Actual tokens after filtering
- **Saved**: Input - Output (tokens that never reached the LLM)
- **Save%**: Saved / Input × 100

## Weekly and monthly breakdowns

```bash
rtco gain --weekly
rtco gain --monthly
```

Same columns as daily, aggregated by Sunday-Saturday week or calendar month.

## Export formats

| Format | Flag | Use case |
|--------|------|----------|
| `text` | default | Terminal display |
| `json` | `--format json` | Programmatic analysis, dashboards |
| `csv` | `--format csv` | Excel, Python/R, Google Sheets |

**JSON structure:**
```json
{
  "summary": {
    "total_commands": 196,
    "total_input": 1276098,
    "total_output": 59244,
    "total_saved": 1220217,
    "avg_savings_pct": 95.62
  },
  "daily": [...],
  "weekly": [...],
  "monthly": [...]
}
```

## Typical savings by command

| Command | Typical savings | Mechanism |
|---------|----------------|-----------|
| `git status` | 77-93% | Compact stat format |
| `eslint` | 84% | Group by rule |
| `jest` | 94-99% | Show failures only |
| `vitest` | 94-99% | Show failures only |
| `find` | 75% | Tree format |
| `pnpm list` | 70-90% | Compact dependencies |
| `grep` | 70% | Truncate + group |

## How token estimation works

RTCO estimates tokens using `text.len() / 4` (4 characters per token average). This is accurate to ±10% compared to actual LLM tokenization — sufficient for trend analysis.

```
Input Tokens  = estimate_tokens(raw_command_output)
Output Tokens = estimate_tokens(rtco_filtered_output)
Saved Tokens  = Input - Output
Savings %     = (Saved / Input) × 100
```

## Database

Savings data is stored locally in SQLite:

- **Location**: `~/.local/share/rtco/history.db` (Linux / macOS)
- **Retention**: 90 days (automatic cleanup)
- **Scope**: Global across all projects and Claude sessions

```bash
# Inspect raw data
sqlite3 ~/.local/share/rtco/history.db \
  "SELECT timestamp, rtco_cmd, saved_tokens FROM commands
   ORDER BY timestamp DESC LIMIT 10"

# Backup
cp ~/.local/share/rtco/history.db ~/backups/rtco-history-$(date +%Y%m%d).db

# Reset
rm ~/.local/share/rtco/history.db    # recreated on next command
```

## Analysis workflows

```bash
# Weekly progress: generate a CSV report every Monday
rtco gain --weekly --format csv > reports/week-$(date +%Y-%W).csv

# Monthly budget review
rtco gain --monthly --format json | jq '.monthly[] |
  {month, saved_tokens, quota_pct: (.saved_tokens / 6000000 * 100)}'

# Cron: daily JSON snapshot for a dashboard
0 0 * * * rtco gain --all --format json > /var/www/dashboard/rtco-stats.json
```

**Python/pandas:**
```python
import pandas as pd
import subprocess

result = subprocess.run(['rtco', 'gain', '--all', '--format', 'csv'],
                       capture_output=True, text=True)
lines = result.stdout.split('\n')
daily_start = lines.index('# Daily Data') + 2
daily_end = lines.index('', daily_start)
daily_df = pd.read_csv(pd.StringIO('\n'.join(lines[daily_start:daily_end])))
daily_df['date'] = pd.to_datetime(daily_df['date'])
daily_df.plot(x='date', y='savings_pct', kind='line')
```

**GitHub Actions (weekly stats):**
```yaml
on:
  schedule:
    - cron: '0 0 * * 1'
jobs:
  stats:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - run: cargo install rtco
      - run: rtco gain --weekly --format json > stats/week-$(date +%Y-%W).json
      - run: git add stats/ && git commit -m "Weekly rtco stats" && git push
```

## Quota estimate

`--quota` estimates how many tokens RTCO has saved relative to your monthly subscription budget, so you can see the cost impact of those savings.

```bash
rtco gain --quota          # uses 20x tier by default
rtco gain --quota -t pro   # Claude Pro plan budget
rtco gain --quota -t 5x    # 5× usage plan budget
rtco gain --quota -t 20x   # 20× usage plan budget
```

The tiers (`pro`, `5x`, `20x`) correspond to Anthropic Claude API subscription levels, each with a different monthly token allocation. RTCO uses those allocations as a denominator to express your savings as a percentage of your budget.

:::tip[Find missed savings]
`rtco gain` shows what RTCO saved. To find commands that ran *without* RTCO and calculate what you lost, see [rtco discover](./discover.md).
:::

## Troubleshooting

**No data showing:**
```bash
ls -lh ~/.local/share/rtco/history.db
sqlite3 ~/.local/share/rtco/history.db "SELECT COUNT(*) FROM commands"
git status    # run any tracked command to generate data
```

**Incorrect statistics:** Token estimation is a heuristic. For precise counts, use `tiktoken`:
```bash
pip install tiktoken
git status > output.txt
python -c "
import tiktoken
enc = tiktoken.get_encoding('cl100k_base')
print(len(enc.encode(open('output.txt').read())), 'actual tokens')
"
```
