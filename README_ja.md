<p align="center">
  <img src="https://avatars.githubusercontent.com/u/258253854?v=4" alt="RTCO - Rust Token Killer" width="500">
</p>

<p align="center">
  <strong>LLM トークン消費を 60-90% 削減する高性能 CLI プロキシ</strong>
</p>

<p align="center">
  <a href="https://github.com/rtco-ai/rtco/actions"><img src="https://github.com/rtco-ai/rtco/workflows/Security%20Check/badge.svg" alt="CI"></a>
  <a href="https://github.com/rtco-ai/rtco/releases"><img src="https://img.shields.io/github/v/release/rtco-ai/rtco" alt="Release"></a>
  <a href="https://opensource.org/licenses/Apache-2.0"><img src="https://img.shields.io/badge/License-Apache_2.0-blue.svg" alt="License: Apache 2.0"></a>
  <a href="https://discord.gg/RySmvNF5kF"><img src="https://img.shields.io/discord/1478373640461488159?label=Discord&logo=discord" alt="Discord"></a>
  <a href="https://formulae.brew.sh/formula/rtco"><img src="https://img.shields.io/homebrew/v/rtco" alt="Homebrew"></a>
</p>

<p align="center">
  <a href="https://www.rtco-ai.app">ウェブサイト</a> &bull;
  <a href="#インストール">インストール</a> &bull;
  <a href="docs/TROUBLESHOOTING.md">トラブルシューティング</a> &bull;
  <a href="docs/contributing/ARCHITECTURE.md">アーキテクチャ</a> &bull;
  <a href="https://discord.gg/RySmvNF5kF">Discord</a>
</p>

<p align="center">
  <a href="README.md">English</a> &bull;
  <a href="README_fr.md">Francais</a> &bull;
  <a href="README_zh.md">中文</a> &bull;
  <a href="README_ja.md">日本語</a> &bull;
  <a href="README_ko.md">한국어</a> &bull;
  <a href="README_es.md">Espanol</a>
</p>

---

rtco はコマンド出力を LLM コンテキストに届く前にフィルタリング・圧縮します。単一の Rust バイナリ、依存関係ゼロ、オーバーヘッド 10ms 未満。

## トークン節約（30分の Claude Code セッション）

| 操作 | 頻度 | 標準 | rtco | 節約 |
|------|------|------|-----|------|
| `ls` / `tree` | 10x | 2,000 | 400 | -80% |
| `cat` / `read` | 20x | 40,000 | 12,000 | -70% |
| `grep` / `rg` | 8x | 16,000 | 3,200 | -80% |
| `git status` | 10x | 3,000 | 600 | -80% |
| `cargo test` / `npm test` | 5x | 25,000 | 2,500 | -90% |
| **合計** | | **~118,000** | **~23,900** | **-80%** |

## インストール

### Homebrew（推奨）

```bash
brew install rtco
```

### クイックインストール（Linux/macOS）

```bash
curl -fsSL https://raw.githubusercontent.com/rtco-ai/rtco/refs/heads/master/install.sh | sh
```

### Cargo

```bash
cargo install --git https://github.com/rtco-ai/rtco
```

### 確認

```bash
rtco --version   # "rtco 0.27.x" と表示されるはず
rtco gain        # トークン節約統計が表示されるはず
```

## クイックスタート

```bash
# 1. Claude Code 用フックをインストール（推奨）
rtco init --global

# 2. Claude Code を再起動してテスト
git status  # 自動的に rtco git status に書き換え
```

## 仕組み

```
  rtco なし：                                       rtco あり：

  Claude  --git status-->  shell  -->  git          Claude  --git status-->  RTCO  -->  git
    ^                                   |             ^                      |          |
    |        ~2,000 tokens（生出力）     |             |   ~200 tokens        | フィルタ |
    +-----------------------------------+             +------- （圧縮済）----+----------+
```

4つの戦略：

1. **スマートフィルタリング** - ノイズを除去（コメント、空白、ボイラープレート）
2. **グルーピング** - 類似項目を集約（ディレクトリ別ファイル、タイプ別エラー）
3. **トランケーション** - 関連コンテキストを保持、冗長性をカット
4. **重複排除** - 繰り返しログ行をカウント付きで統合

## コマンド

### ファイル
```bash
rtco ls .                        # 最適化されたディレクトリツリー
rtco read file.rs                # スマートファイル読み取り
rtco find "*.rs" .               # コンパクトな検索結果
rtco grep "pattern" .            # ファイル別グループ化検索
```

### Git
```bash
rtco git status                  # コンパクトなステータス
rtco git log -n 10               # 1行コミット
rtco git diff                    # 圧縮された diff
rtco git push                    # -> "ok main"
```

### テスト
```bash
rtco jest                        # Jest コンパクト
rtco vitest                      # Vitest コンパクト
rtco pytest                      # Python テスト（-90%）
rtco go test                     # Go テスト（-90%）
rtco test <cmd>                  # 失敗のみ表示（-90%）
```

### ビルド & リント
```bash
rtco lint                        # ESLint ルール別グループ化
rtco tsc                         # TypeScript エラーグループ化
rtco cargo build                 # Cargo ビルド（-80%）
rtco ruff check                  # Python リント（-80%）
```

### 分析
```bash
rtco gain                        # 節約統計
rtco gain --graph                # ASCII グラフ（30日間）
rtco discover                    # 見逃した節約機会を発見
```

## ドキュメント

- **[TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md)** - よくある問題の解決
- **[INSTALL.md](INSTALL.md)** - 詳細インストールガイド
- **[ARCHITECTURE.md](docs/contributing/ARCHITECTURE.md)** - 技術アーキテクチャ

## コントリビュート

コントリビューション歓迎！[GitHub](https://github.com/rtco-ai/rtco) で issue または PR を作成してください。

[Discord](https://discord.gg/RySmvNF5kF) コミュニティに参加。

## ライセンス

Apache 2.0 ライセンス - 詳細は [LICENSE](LICENSE) を参照。

## 免責事項

詳細は [DISCLAIMER.md](DISCLAIMER.md) を参照。
