<p align="center">
  <img src="https://avatars.githubusercontent.com/u/258253854?v=4" alt="RTCO - Rust Token Killer" width="500">
</p>

<p align="center">
  <strong>高性能 CLI 代理，将 LLM token 消耗降低 60-90%</strong>
</p>

<p align="center">
  <a href="https://github.com/rtco-ai/rtco/actions"><img src="https://github.com/rtco-ai/rtco/workflows/Security%20Check/badge.svg" alt="CI"></a>
  <a href="https://github.com/rtco-ai/rtco/releases"><img src="https://img.shields.io/github/v/release/rtco-ai/rtco" alt="Release"></a>
  <a href="https://opensource.org/licenses/Apache-2.0"><img src="https://img.shields.io/badge/License-Apache_2.0-blue.svg" alt="License: Apache 2.0"></a>
  <a href="https://discord.gg/RySmvNF5kF"><img src="https://img.shields.io/discord/1478373640461488159?label=Discord&logo=discord" alt="Discord"></a>
  <a href="https://formulae.brew.sh/formula/rtco"><img src="https://img.shields.io/homebrew/v/rtco" alt="Homebrew"></a>
</p>

<p align="center">
  <a href="https://www.rtco-ai.app">官网</a> &bull;
  <a href="#安装">安装</a> &bull;
  <a href="docs/TROUBLESHOOTING.md">故障排除</a> &bull;
  <a href="docs/contributing/ARCHITECTURE.md">架构</a> &bull;
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

rtco 在命令输出到达 LLM 上下文之前进行过滤和压缩。单一 Rust 二进制文件，零依赖，<10ms 开销。

## Token 节省（30 分钟 Claude Code 会话）

| 操作 | 频率 | 标准 | rtco | 节省 |
|------|------|------|-----|------|
| `ls` / `tree` | 10x | 2,000 | 400 | -80% |
| `cat` / `read` | 20x | 40,000 | 12,000 | -70% |
| `grep` / `rg` | 8x | 16,000 | 3,200 | -80% |
| `git status` | 10x | 3,000 | 600 | -80% |
| `git diff` | 5x | 10,000 | 2,500 | -75% |
| `cargo test` / `npm test` | 5x | 25,000 | 2,500 | -90% |
| **总计** | | **~118,000** | **~23,900** | **-80%** |

## 安装

### Homebrew（推荐）

```bash
brew install rtco
```

### 快速安装（Linux/macOS）

```bash
curl -fsSL https://raw.githubusercontent.com/rtco-ai/rtco/refs/heads/master/install.sh | sh
```

### Cargo

```bash
cargo install --git https://github.com/rtco-ai/rtco
```

### 验证

```bash
rtco --version   # 应显示 "rtco 0.27.x"
rtco gain        # 应显示 token 节省统计
```

## 快速开始

```bash
# 1. 为 Claude Code 安装 hook（推荐）
rtco init --global

# 2. 重启 Claude Code，然后测试
git status  # 自动重写为 rtco git status
```

## 工作原理

```
  没有 rtco：                                      使用 rtco：

  Claude  --git status-->  shell  -->  git         Claude  --git status-->  RTCO  -->  git
    ^                                   |            ^                      |          |
    |        ~2,000 tokens（原始）       |            |   ~200 tokens        | 过滤     |
    +-----------------------------------+            +------- （已过滤）-----+----------+
```

四种策略：

1. **智能过滤** - 去除噪音（注释、空白、样板代码）
2. **分组** - 聚合相似项（按目录分文件，按类型分错误）
3. **截断** - 保留相关上下文，删除冗余
4. **去重** - 合并重复日志行并计数

## 命令

### 文件
```bash
rtco ls .                        # 优化的目录树
rtco read file.rs                # 智能文件读取
rtco find "*.rs" .               # 紧凑的查找结果
rtco grep "pattern" .            # 按文件分组的搜索结果
```

### Git
```bash
rtco git status                  # 紧凑状态
rtco git log -n 10               # 单行提交
rtco git diff                    # 精简 diff
rtco git push                    # -> "ok main"
```

### 测试
```bash
rtco jest                        # Jest 紧凑输出
rtco vitest                      # Vitest 紧凑输出
rtco pytest                      # Python 测试（-90%）
rtco go test                     # Go 测试（-90%）
rtco test <cmd>                  # 仅显示失败（-90%）
```

### 构建 & 检查
```bash
rtco lint                        # ESLint 按规则分组
rtco tsc                         # TypeScript 错误分组
rtco cargo build                 # Cargo 构建（-80%）
rtco ruff check                  # Python lint（-80%）
```

### 容器
```bash
rtco docker ps                   # 紧凑容器列表
rtco docker logs <container>     # 去重日志
rtco kubectl pods                # 紧凑 Pod 列表
```

### 分析
```bash
rtco gain                        # 节省统计
rtco gain --graph                # ASCII 图表（30 天）
rtco discover                    # 发现遗漏的节省机会
```

## 文档

- **[TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md)** - 解决常见问题
- **[INSTALL.md](INSTALL.md)** - 详细安装指南
- **[ARCHITECTURE.md](docs/contributing/ARCHITECTURE.md)** - 技术架构

## 贡献

欢迎贡献！请在 [GitHub](https://github.com/rtco-ai/rtco) 上提交 issue 或 PR。

加入 [Discord](https://discord.gg/RySmvNF5kF) 社区。

## 许可证

Apache 2.0 许可证 - 详见 [LICENSE](LICENSE)。

## 免责声明

详见 [DISCLAIMER.md](DISCLAIMER.md)。
