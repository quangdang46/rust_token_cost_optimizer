<p align="center">
  <img src="https://avatars.githubusercontent.com/u/258253854?v=4" alt="RTCO - Rust Token Killer" width="500">
</p>

<p align="center">
  <strong>LLM 토큰 소비를 60-90% 줄이는 고성능 CLI 프록시</strong>
</p>

<p align="center">
  <a href="https://github.com/rtco-ai/rtco/actions"><img src="https://github.com/rtco-ai/rtco/workflows/Security%20Check/badge.svg" alt="CI"></a>
  <a href="https://github.com/rtco-ai/rtco/releases"><img src="https://img.shields.io/github/v/release/rtco-ai/rtco" alt="Release"></a>
  <a href="https://opensource.org/licenses/MIT"><img src="https://img.shields.io/badge/License-MIT-yellow.svg" alt="License: MIT"></a>
  <a href="https://discord.gg/RySmvNF5kF"><img src="https://img.shields.io/discord/1478373640461488159?label=Discord&logo=discord" alt="Discord"></a>
  <a href="https://formulae.brew.sh/formula/rtco"><img src="https://img.shields.io/homebrew/v/rtco" alt="Homebrew"></a>
</p>

<p align="center">
  <a href="https://www.rtco-ai.app">웹사이트</a> &bull;
  <a href="#설치">설치</a> &bull;
  <a href="docs/TROUBLESHOOTING.md">문제 해결</a> &bull;
  <a href="docs/contributing/ARCHITECTURE.md">아키텍처</a> &bull;
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

rtk는 명령 출력이 LLM 컨텍스트에 도달하기 전에 필터링하고 압축합니다. 단일 Rust 바이너리, 의존성 없음, 10ms 미만의 오버헤드.

## 토큰 절약 (30분 Claude Code 세션)

| 작업 | 빈도 | 표준 | rtco | 절약 |
|------|------|------|-----|------|
| `ls` / `tree` | 10x | 2,000 | 400 | -80% |
| `cat` / `read` | 20x | 40,000 | 12,000 | -70% |
| `grep` / `rg` | 8x | 16,000 | 3,200 | -80% |
| `git status` | 10x | 3,000 | 600 | -80% |
| `cargo test` / `npm test` | 5x | 25,000 | 2,500 | -90% |
| **합계** | | **~118,000** | **~23,900** | **-80%** |

## 설치

### Homebrew (권장)

```bash
brew install rtco
```

### 빠른 설치 (Linux/macOS)

```bash
curl -fsSL https://raw.githubusercontent.com/rtco-ai/rtco/refs/heads/master/install.sh | sh
```

### Cargo

```bash
cargo install --git https://github.com/rtco-ai/rtco
```

### 확인

```bash
rtco --version   # "rtco 0.27.x" 표시되어야 함
rtco gain        # 토큰 절약 통계 표시되어야 함
```

## 빠른 시작

```bash
# 1. Claude Code용 hook 설치 (권장)
rtco init --global

# 2. Claude Code 재시작 후 테스트
git status  # 자동으로 rtco git status로 재작성
```

## 작동 원리

```
  rtco 없이:                                        rtco 사용:

  Claude  --git status-->  shell  -->  git          Claude  --git status-->  RTCO  -->  git
    ^                                   |             ^                      |          |
    |        ~2,000 tokens (원본)        |             |   ~200 tokens        | 필터     |
    +-----------------------------------+             +------- (필터링) -----+----------+
```

네 가지 전략:

1. **스마트 필터링** - 노이즈 제거 (주석, 공백, 보일러플레이트)
2. **그룹화** - 유사 항목 집계 (디렉토리별 파일, 유형별 에러)
3. **잘라내기** - 관련 컨텍스트 유지, 중복 제거
4. **중복 제거** - 반복 로그 라인을 카운트와 함께 통합

## 명령어

### 파일
```bash
rtco ls .                        # 최적화된 디렉토리 트리
rtco read file.rs                # 스마트 파일 읽기
rtco find "*.rs" .               # 컴팩트한 검색 결과
rtco grep "pattern" .            # 파일별 그룹화 검색
```

### Git
```bash
rtco git status                  # 컴팩트 상태
rtco git log -n 10               # 한 줄 커밋
rtco git diff                    # 압축된 diff
rtco git push                    # -> "ok main"
```

### 테스트
```bash
rtco jest                        # Jest 컴팩트
rtco vitest                      # Vitest 컴팩트
rtco pytest                      # Python 테스트 (-90%)
rtco go test                     # Go 테스트 (-90%)
rtco test <cmd>                  # 실패만 표시 (-90%)
```

### 빌드 & 린트
```bash
rtco lint                        # ESLint 규칙별 그룹화
rtco tsc                         # TypeScript 에러 그룹화
rtco cargo build                 # Cargo 빌드 (-80%)
rtco ruff check                  # Python 린트 (-80%)
```

### 분석
```bash
rtco gain                        # 절약 통계
rtco gain --graph                # ASCII 그래프 (30일)
rtco discover                    # 놓친 절약 기회 발견
```

## 문서

- **[TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md)** - 일반적인 문제 해결
- **[INSTALL.md](INSTALL.md)** - 상세 설치 가이드
- **[ARCHITECTURE.md](docs/contributing/ARCHITECTURE.md)** - 기술 아키텍처

## 기여

기여를 환영합니다! [GitHub](https://github.com/rtco-ai/rtco)에서 issue 또는 PR을 생성해 주세요.

[Discord](https://discord.gg/RySmvNF5kF) 커뮤니티에 참여하세요.

## 라이선스

MIT 라이선스 - 자세한 내용은 [LICENSE](LICENSE)를 참조하세요.

## 면책 조항

자세한 내용은 [DISCLAIMER.md](DISCLAIMER.md)를 참조하세요.
