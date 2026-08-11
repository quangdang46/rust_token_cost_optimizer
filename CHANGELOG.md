# Changelog

All notable changes to rtco (Rust Token Killer) will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.6](https://github.com/quangdang46/rust_token_cost_optimizer/compare/rtco-v0.2.5...rtco-v0.2.6) (2026-08-11)


### Features

* Add rtco hashline + rtco ffs binary proxies ([129b68b](https://github.com/quangdang46/rust_token_cost_optimizer/commit/129b68b102bfff28cbd0dda02b8f50c2b6f828ef))
* Address pr suggestions ([9bd6e6f](https://github.com/quangdang46/rust_token_cost_optimizer/commit/9bd6e6f3929cf320193e34a2654603afa1e9fa91))
* **beads:** Add full gap analysis graph - 39 beads across 7 sprints ([598bb08](https://github.com/quangdang46/rust_token_cost_optimizer/commit/598bb0839e2b41962ba755f2099fca9453eceef5))
* **beads:** Add remaining 11 gaps — 50 total beads across 7 sprints ([cff7ef9](https://github.com/quangdang46/rust_token_cost_optimizer/commit/cff7ef93779fae100684bddba93dc5fe25903fff))
* **benchmark:** Add multipass VM integration test suite ([6e7863b](https://github.com/quangdang46/rust_token_cost_optimizer/commit/6e7863bf313b0d18a47cf0ca2cdaea03cc2ed900))
* **benchmark:** Add multipass VM integration test suite ([d22759b](https://github.com/quangdang46/rust_token_cost_optimizer/commit/d22759b8c5254ad9c4a455f10cb7de75e92df581))
* **benchmark:** Add Swift ecosystem tests (6 commands + savings) ([1fbb6d9](https://github.com/quangdang46/rust_token_cost_optimizer/commit/1fbb6d935b4a0d031a7862cba312eebe1303ba9b))
* **cicd:** Add auto next release parser ([bf24972](https://github.com/quangdang46/rust_token_cost_optimizer/commit/bf24972e7d463f0432b8315e3035e9eb13ff062f))
* **cicd:** Add auto next release parser ([f3e33f3](https://github.com/quangdang46/rust_token_cost_optimizer/commit/f3e33f38872008fe0046e2e139a6762845504b8e))
* **cicd:** Enforce cicd sast & package check ([3bbbb49](https://github.com/quangdang46/rust_token_cost_optimizer/commit/3bbbb492f33f0e619ab0d1dbce4389ad49e763ae))
* **cicd:** Enforce cicd sast & package check ([4a22820](https://github.com/quangdang46/rust_token_cost_optimizer/commit/4a228208e3094a0819d10e0c62ba37ee1538698d))
* **cicd:** Target develop branch ([63da7da](https://github.com/quangdang46/rust_token_cost_optimizer/commit/63da7dafd61b5f65115989aeda01f666a64457ff))
* **cli:** Rtco mcp subcommand + --mcp/--hooks/--provider flags ([0560b5b](https://github.com/quangdang46/rust_token_cost_optimizer/commit/0560b5b0a403af53f560b7cd040bfde39a1cfd26))
* **discover:** Handle more npm/npx/pnpm/pnpx patterns ([9e96caa](https://github.com/quangdang46/rust_token_cost_optimizer/commit/9e96caa0a18a95c84da82ba57716a9d3ef86d0c8))
* **discover:** Handle more npm/npx/pnpm/pnpx patterns ([bab3a53](https://github.com/quangdang46/rust_token_cost_optimizer/commit/bab3a53f24f95a4a5821b23712f0b7f2ce3e0445))
* **gains:** Add --reset flag ([e3149cb](https://github.com/quangdang46/rust_token_cost_optimizer/commit/e3149cb7fbed18eae95f753664ddd8eaaaf6cc39))
* **glab:** Add GitLab CLI (glab) command support ([048f2f9](https://github.com/quangdang46/rust_token_cost_optimizer/commit/048f2f980bd95c5918f309d1d7ebc096d196f00d))
* **glab:** Add GitLab CLI (glab) command support ([bc31f3f](https://github.com/quangdang46/rust_token_cost_optimizer/commit/bc31f3f0f39077884e8d52c3508e840b355f682e)), closes [#851](https://github.com/quangdang46/rust_token_cost_optimizer/issues/851)
* **gradlew:** Add Gradle/gradlew support with streaming filters ([71d285f](https://github.com/quangdang46/rust_token_cost_optimizer/commit/71d285fc42d2a97d6d1472bb96fff2db9ab39bab))
* **gradlew:** Gradle support for Android/Kotlin developers ([833026b](https://github.com/quangdang46/rust_token_cost_optimizer/commit/833026b893822be4e1c64d22d640e979cd9eff51))
* Headroom algorithms + workspace + JCode integration ([#30](https://github.com/quangdang46/rust_token_cost_optimizer/issues/30)) ([37dcb0c](https://github.com/quangdang46/rust_token_cost_optimizer/commit/37dcb0cfbd47e0722b07a185f0e1adfeb331b53f))
* **hermes:** Add Hermes Agent support via rtk init --agent hermes ([55f998d](https://github.com/quangdang46/rust_token_cost_optimizer/commit/55f998d08cd80ece970fe5e61eaae3533512288b))
* **hermes:** Add rtk integration ([9d3b99d](https://github.com/quangdang46/rust_token_cost_optimizer/commit/9d3b99dec8516fd32071d151306b5bb6fd4d06e3))
* **hints:** Add tail hints for tee & hints + address reviews ([46fe7c4](https://github.com/quangdang46/rust_token_cost_optimizer/commit/46fe7c47293fcbef28159ddc9fcd118a344cc42b))
* **hook:** Add pi support ([805caf7](https://github.com/quangdang46/rust_token_cost_optimizer/commit/805caf7d069e93370a316682b36aad59d562de2e))
* **hooks:** Add Pi coding agent integration ([1da5793](https://github.com/quangdang46/rust_token_cost_optimizer/commit/1da5793b9293cca5fca3e316bda18ed02443f2e2))
* **hooks:** Add transparent_prefixes config for wrapper commands ([998f1ee](https://github.com/quangdang46/rust_token_cost_optimizer/commit/998f1ee0a3cf8d73ea0d6d87c121117f351e4992))
* **init:** Add --dry-run flag to preview changes without writing ([172ec54](https://github.com/quangdang46/rust_token_cost_optimizer/commit/172ec54580ddb0d737ef3e3be8a075eaeeb0a01b))
* **init:** Add --dry-run flag to preview changes without writing ([21a069a](https://github.com/quangdang46/rust_token_cost_optimizer/commit/21a069ad76c3d8ffeee870d24408b1942cee691c))
* **init:** Remove --pi flag, canonicalize Pi install to --agent pi ([cb1661e](https://github.com/quangdang46/rust_token_cost_optimizer/commit/cb1661e68d995e72bec49fb0e619fd8178f376a4))
* **install:** Install.sh/.ps1 --with-mcp/--with-hooks auto-config ([33ed033](https://github.com/quangdang46/rust_token_cost_optimizer/commit/33ed0335089c1f840e137ba786691c355a39d036))
* **js:** Distinguish between `jest` and `vitest` and don't rewrite `npm test` commands as we don't know which test framework is used under the hood ([45938b2](https://github.com/quangdang46/rust_token_cost_optimizer/commit/45938b2a4d3fe76685a6e008f210bb276df50319))
* **jvm:** Add rtco mvn with Surefire/Failsafe XML test summarization ([06cee37](https://github.com/quangdang46/rust_token_cost_optimizer/commit/06cee3761585474159becfe7c780725fd5682384))
* **jvm:** Add rtco mvn with Surefire/Failsafe XML test summarization (port rtk[#1974](https://github.com/quangdang46/rust_token_cost_optimizer/issues/1974)) ([d8f1677](https://github.com/quangdang46/rust_token_cost_optimizer/commit/d8f1677f816f2cbe9f07139a6527f5a801989459))
* **pnpm:** Add filter argument support ([2ba8d37](https://github.com/quangdang46/rust_token_cost_optimizer/commit/2ba8d372df186b4056a3b8906fc25cde8586dd42))
* **pnpm:** Handle pnpm build rewrite ([f936138](https://github.com/quangdang46/rust_token_cost_optimizer/commit/f93613881939bc278261e9143bd328c82c696a33))
* Port 4 upstream rtk features (Vibe, Scala/sbt, Cursor ask, pipeline_final_safe) ([5d71132](https://github.com/quangdang46/rust_token_cost_optimizer/commit/5d71132802f18b195a9552fe823892d5ef7c0581))
* Port headroom algorithms - Shannon entropy, SimHash, dedup filter ([dd60bde](https://github.com/quangdang46/rust_token_cost_optimizer/commit/dd60bde689ae4887ac58990ce902f7b4b9d91102))
* Port headroom algorithms - Shannon entropy, SimHash, dedup filter ([cc4ba76](https://github.com/quangdang46/rust_token_cost_optimizer/commit/cc4ba7610fc0f6e1fc830b55fd50c07c54592acc))
* Port RTK safety and filter gaps (guard, curl binary, uv, PHP, Pulumi) ([3dc6e13](https://github.com/quangdang46/rust_token_cost_optimizer/commit/3dc6e13948f2f6e4086733429cd181945926c4a5))
* Port upstream RTK improvements - 1200+ lines ([cc391f0](https://github.com/quangdang46/rust_token_cost_optimizer/commit/cc391f0dfc645e702b89b857029d3f7019b21902))
* Port upstream RTK improvements - SIGPIPE, args_utils, permissions, bug fixes ([c515279](https://github.com/quangdang46/rust_token_cost_optimizer/commit/c515279b7eb979ab6625dfc4823cd397d4fb5280))
* Prefer short args and cleanup comments ([fb67bb6](https://github.com/quangdang46/rust_token_cost_optimizer/commit/fb67bb6532e1a80519b08520876019cc5fa5e6eb))
* Prefer short args and cleanup comments ([53b5e79](https://github.com/quangdang46/rust_token_cost_optimizer/commit/53b5e79561fdf98f09db0b5b2006ec838fcd06f5))
* **refacto-core:** Binary hook w/ native cmd exec + streaming ([e7b7f9a](https://github.com/quangdang46/rust_token_cost_optimizer/commit/e7b7f9ab665a0f7303d41d23ad156d24e5e8964e))
* Remove MCP server and auto-config ([0ba9092](https://github.com/quangdang46/rust_token_cost_optimizer/commit/0ba9092ec96d83d8781ae16180e3fbe40904df05))
* Rename rtk -&gt; rtco (binary, package, env vars, hooks, docs, data dir migration) ([8dbaa2e](https://github.com/quangdang46/rust_token_cost_optimizer/commit/8dbaa2e27e0da6dfa74f4612f471878a185a5a3f))
* Rm rtk awareness injection ([b2a3ad9](https://github.com/quangdang46/rust_token_cost_optimizer/commit/b2a3ad9443d6242f60b4c05265bda56b2a7e72b6))
* **skills:** Add /pr-review skill for batch PR review ([21e67a1](https://github.com/quangdang46/rust_token_cost_optimizer/commit/21e67a1113041b74542d0285e5f74587dfb30b65))
* **sprint-1:** Security & infrastructure fixes ([b3b5c40](https://github.com/quangdang46/rust_token_cost_optimizer/commit/b3b5c40ac3e5e1d4b969e3bc2f3445a6689f575a))
* **sprint-2:** Critical test coverage ([118eca0](https://github.com/quangdang46/rust_token_cost_optimizer/commit/118eca0e8f55f6f47998f89e6bf274b5d00c03fe))
* **sprint-3:** Add test fixtures for all filter modules ([1d048a6](https://github.com/quangdang46/rust_token_cost_optimizer/commit/1d048a6d1c41dc68dc57deed6c7b98cf701fc402))
* **sprint-5:** CI/CD pipeline automation ([8d54e76](https://github.com/quangdang46/rust_token_cost_optimizer/commit/8d54e761a8c6b5221f49b1109571906e4afb27de))
* **sprint-6:** Implement beads 27-33 ([6a902a0](https://github.com/quangdang46/rust_token_cost_optimizer/commit/6a902a05d33b194efc0adac222d3c2741348c5e1))
* **sprint7:** Implement beads 34-39 — ContentRouter, MCP server, Prometheus metrics, design docs ([113d750](https://github.com/quangdang46/rust_token_cost_optimizer/commit/113d75071a33ee1673ca09d7d7dfd5ee90455939))
* **sqlite:** Add sqlite3 filter with table output compression ([3abd5fc](https://github.com/quangdang46/rust_token_cost_optimizer/commit/3abd5fc3bd90575287b17c25d93e08d7898c7d9b))
* **sqlite:** Add sqlite3 filter with table output compression (port rtk[#1972](https://github.com/quangdang46/rust_token_cost_optimizer/issues/1972)) ([35535db](https://github.com/quangdang46/rust_token_cost_optimizer/commit/35535dbb39c63cff6356ff4404a2c4e3b94d484c))
* **stream:** P1 fixes from PR [#956](https://github.com/quangdang46/rust_token_cost_optimizer/issues/956) review ([71eeeda](https://github.com/quangdang46/rust_token_cost_optimizer/commit/71eeedab4d771986b3d3dc5c439f5646135ff96c))
* Sync with upstream/develop (v0.40.0, Pi agent support, git-log merge fix) ([25192b5](https://github.com/quangdang46/rust_token_cost_optimizer/commit/25192b5029abdbc74b206344ebfd13111d160d70))
* **tee:** Redact sensitive output and add per-command opt-out ([2f90f98](https://github.com/quangdang46/rust_token_cost_optimizer/commit/2f90f98b2499f435541eb8299c638cacc6cab985))
* **tee:** Redact sensitive output and add per-command opt-out (port rtk[#1988](https://github.com/quangdang46/rust_token_cost_optimizer/issues/1988)) ([0f12931](https://github.com/quangdang46/rust_token_cost_optimizer/commit/0f1293117403d28ceb70a314ac6be8a7957d7726))
* **tests:** Add snapshot + savings tests across all filter modules (Sprint 4) ([7a027e5](https://github.com/quangdang46/rust_token_cost_optimizer/commit/7a027e5e5ea2c5aee1ccb1dffd798a4a649403b9))
* **tests:** Add token savings tests to all filter modules + fix low-savings filters ([0c51e9f](https://github.com/quangdang46/rust_token_cost_optimizer/commit/0c51e9f0fd38fc95de2bc88987be14252591b34b))
* **tracking:** Honour tracking.enabled and redact sensitive args ([a6e7cc6](https://github.com/quangdang46/rust_token_cost_optimizer/commit/a6e7cc6fc4d7dfe9944f01010bc7871428c21c76))
* **tracking:** Honour tracking.enabled and redact sensitive args (port rtk[#1987](https://github.com/quangdang46/rust_token_cost_optimizer/issues/1987)) ([37f239a](https://github.com/quangdang46/rust_token_cost_optimizer/commit/37f239a83a21011ce0a58e61a978fa6ab8eb076d))


### Bug Fixes

* '...' ascii to unicode, remove some comments ([3571d52](https://github.com/quangdang46/rust_token_cost_optimizer/commit/3571d5293dc463c2a0aadfa9a5587b18478ca99a))
* **agy:** Address Copilot review — hardening, tests, and docs ([89e1f8c](https://github.com/quangdang46/rust_token_cost_optimizer/commit/89e1f8c2a2579f4c879adf2c0b84ff022b08cfe5))
* **analytics:** Prevent char-boundary panic in display strings ([b25e88c](https://github.com/quangdang46/rust_token_cost_optimizer/commit/b25e88c9bde2fdb46db9fc70537908ef32f34b75))
* **antigravity:** Use hooks.json and correct PreToolHookResult overwrite format ([0741cb1](https://github.com/quangdang46/rust_token_cost_optimizer/commit/0741cb1da4d5883e43ae1482b8cbe3c40af23fc7))
* **aws:** Redact secretsmanager get-secret-value payload ([bc936fd](https://github.com/quangdang46/rust_token_cost_optimizer/commit/bc936fda892c0fef839d5e13e54050b0bb8d9956))
* **aws:** Redact secretsmanager get-secret-value payload (port rtk[#1986](https://github.com/quangdang46/rust_token_cost_optimizer/issues/1986)) ([bfee5ff](https://github.com/quangdang46/rust_token_cost_optimizer/commit/bfee5ffad24c013ca4c16a60b6bcb51bb6383c0a))
* Batch fix 20+ open issues across grep, git, hooks, parser, and docs ([87f678b](https://github.com/quangdang46/rust_token_cost_optimizer/commit/87f678bd6274c7c564cce2d56d065c9ea5314c28))
* Batch fix all 27 open GitHub issues ([307b5bd](https://github.com/quangdang46/rust_token_cost_optimizer/commit/307b5bd357376018f9c936462ea92159c1675e58))
* **benchmark:** Address PR review feedback ([87ee81f](https://github.com/quangdang46/rust_token_cost_optimizer/commit/87ee81f08be5e7b1ca79513b1a91925d455f4f5c))
* **benchmark:** Address review feedback from @FlorianBruniaux ([d13c185](https://github.com/quangdang46/rust_token_cost_optimizer/commit/d13c185aac64d14288b574df44623723a69e7b95))
* **benchmark:** Benchmark capture all fd only stream ([c590bd6](https://github.com/quangdang46/rust_token_cost_optimizer/commit/c590bd69329bb82608666958c7e06bf169a7d577))
* **benchmark:** Capture all fd for stream cmd benchmark ([e6c2523](https://github.com/quangdang46/rust_token_cost_optimizer/commit/e6c2523be1180772e40c175e2f9a523d349fb13d))
* **benchmark:** Extract format_diff_changes + remove wrong diff test ([e7ae6bf](https://github.com/quangdang46/rust_token_cost_optimizer/commit/e7ae6bf018882dba248f151ba4ec4929300b3e36))
* **ccusage:** Add --yes flag and warn when falling back to npx ([f68fa00](https://github.com/quangdang46/rust_token_cost_optimizer/commit/f68fa0087c03d6882993b7b3eaee98e1dbab41b4))
* **ci:** Add missing TEE_ENV_LOCK guard to flaky Windows test ([13f9b03](https://github.com/quangdang46/rust_token_cost_optimizer/commit/13f9b034ad45e7a7ddaba0e14caea9197e4b4b52))
* **ci:** Allow unsafe_code for libc signal handler ([bcbf8ed](https://github.com/quangdang46/rust_token_cost_optimizer/commit/bcbf8ed2b5cdec6af48585ff541a25a049f51c65))
* **cicd:** : no semgrep alert on sh call cicd ([7681daf](https://github.com/quangdang46/rust_token_cost_optimizer/commit/7681dafc76f164cfad588fe37d9a165dcb476e10))
* **cicd:** Match ":" for body prefix to catch ([5987333](https://github.com/quangdang46/rust_token_cost_optimizer/commit/5987333209cd59c1e22f9e0b247ab390cb431dbf))
* **cicd:** Match allowed repo list in pr bodies ([b1233ab](https://github.com/quangdang46/rust_token_cost_optimizer/commit/b1233ab3fbc0927145d5c0f763725b098fc7dd99))
* **cicd:** MIT to Apache 2.0 ([5a149a7](https://github.com/quangdang46/rust_token_cost_optimizer/commit/5a149a7fdb92afe758a0c28d805873ce61d8259f))
* **cicd:** MIT to Apache 2.0 ([e132896](https://github.com/quangdang46/rust_token_cost_optimizer/commit/e132896d3f3b588813f59790a5c2f7d35e40cb78))
* **cicd:** Pr-target clean msg + git app token ([e4c3ed7](https://github.com/quangdang46/rust_token_cost_optimizer/commit/e4c3ed7d889ede726df7986ade94a4714c7c7f99))
* **cicd:** Pr-target clean msg + git app token ([4ebda52](https://github.com/quangdang46/rust_token_cost_optimizer/commit/4ebda52f5ab898f9c0e8c610cc51b36a63e6eefa))
* **cicd:** Semgrep use docker (git action archived) ([8857e17](https://github.com/quangdang46/rust_token_cost_optimizer/commit/8857e1725e483d7e047f1875f68570b8c7efc5a8))
* **cicd:** Set release-please target-branch to master [skip ci] ([0c6a838](https://github.com/quangdang46/rust_token_cost_optimizer/commit/0c6a838594e87346b67bd13c092b8a46a783af87))
* **ci:** Clean up CD workflow - remove release-please, release.yml (upstream), fix concurrency branch; update CICD.md ([a61602f](https://github.com/quangdang46/rust_token_cost_optimizer/commit/a61602f82ae286078399a1c33b7035c2b4740d2a))
* **ci:** Correct include_str! fixture path from crate root to workspace root ([92a23ce](https://github.com/quangdang46/rust_token_cost_optimizer/commit/92a23ce34ba91755dc6f90cda4e43e02a61606b3))
* **ci:** Replace GitHub App token with GITHUB_TOKEN in cd.yml ([e83a1cc](https://github.com/quangdang46/rust_token_cost_optimizer/commit/e83a1cc129af291801e7b647235bf709c161cc06))
* **ci:** Replace pinned action SHAs with version tags ([1588aea](https://github.com/quangdang46/rust_token_cost_optimizer/commit/1588aea5ee4ee50d4ca659695f32c7a98bd21c26))
* **ci:** Use github.token instead of secrets.GITHUB_TOKEN ([5222ae2](https://github.com/quangdang46/rust_token_cost_optimizer/commit/5222ae23fc74f790787195762817c2dc213f0fdc))
* **ci:** Use include_str! for dotnet format test fixtures to fix CI path resolution ([9f4a4df](https://github.com/quangdang46/rust_token_cost_optimizer/commit/9f4a4df236a74c9643644d6e9e92f7bf515628ea))
* **clippy:** Show full error blocks instead of truncated headline ([95d9d13](https://github.com/quangdang46/rust_token_cost_optimizer/commit/95d9d134b0b76d83b6162614b0a79269b2135f40))
* **clippy:** Show full error blocks instead of truncated headline ([f4074f8](https://github.com/quangdang46/rust_token_cost_optimizer/commit/f4074f898a9b73b72bbcd8b18afab4831dcda406)), closes [#602](https://github.com/quangdang46/rust_token_cost_optimizer/issues/602)
* **cmds/git/diff:** Preserve POSIX/git contract for programmatic consumers ([9680700](https://github.com/quangdang46/rust_token_cost_optimizer/commit/9680700a1fd05323c5f4246e9b5b4ef3b3a3ed95))
* **cmds/git/diff:** Preserve POSIX/git contract for programmatic consumers (port rtk[#1981](https://github.com/quangdang46/rust_token_cost_optimizer/issues/1981)) ([fb5bf71](https://github.com/quangdang46/rust_token_cost_optimizer/commit/fb5bf719f80a9e2d598b83d681e4423a7fd0bd10))
* Complete rtk to rtco rename in tests and production code ([3cd39f4](https://github.com/quangdang46/rust_token_cost_optimizer/commit/3cd39f4fcdf33aecdf50acceb291292780077f4c))
* **core:** Review 956 various fix ([840571f](https://github.com/quangdang46/rust_token_cost_optimizer/commit/840571fe90ed14fb7e96f9b9000a1bac6d196d23))
* **core:** Surface truncated context inline so AI gets full diagnostics ([b4bb493](https://github.com/quangdang46/rust_token_cost_optimizer/commit/b4bb49315f48edb77d506aaf3e9bd3ed569d60fd))
* **core:** Surface truncated context inline so AI gets full diagnostics ([f557bb0](https://github.com/quangdang46/rust_token_cost_optimizer/commit/f557bb06e94ea34571f82db71e1c2d2bb0963a4c))
* Correct ARCHITECTURE.md path in README links ([2a41e03](https://github.com/quangdang46/rust_token_cost_optimizer/commit/2a41e039903049543aa6c69482747eddcce9ee5a))
* Correct ARCHITECTURE.md path in README links ([f2da381](https://github.com/quangdang46/rust_token_cost_optimizer/commit/f2da381ae2353d31dd7252af6c868c56f6aa3db8))
* **curl:** Gate force_tee_hint, extend JSON heuristic, avoid full-body alloc ([2ed53c7](https://github.com/quangdang46/rust_token_cost_optimizer/commit/2ed53c7fa26922860af20c445b39cbb66862f180))
* **curl:** JSON passthrough + IsTerminal gate to prevent invalid JSON output ([02da3d0](https://github.com/quangdang46/rust_token_cost_optimizer/commit/02da3d070271f800731a94a3249f3feb9dd7c7b8)), closes [#1536](https://github.com/quangdang46/rust_token_cost_optimizer/issues/1536) [#1282](https://github.com/quangdang46/rust_token_cost_optimizer/issues/1282)
* **curl:** Skip JSON schema conversion for internal/localhost URLs ([577c311](https://github.com/quangdang46/rust_token_cost_optimizer/commit/577c311ecaaa8ae94f22dbe252152424d4333d04))
* **discover:** Also encode '_', '\', and non-ASCII chars in project path slug ([73a05c3](https://github.com/quangdang46/rust_token_cost_optimizer/commit/73a05c3262b6410cb24370d939c428d1dc0c7a77)), closes [#1457](https://github.com/quangdang46/rust_token_cost_optimizer/issues/1457)
* **discover:** Encode '.' as '-' in project path slug ([2d031f3](https://github.com/quangdang46/rust_token_cost_optimizer/commit/2d031f32e9ad4452c2cc229c030ea6c0936c8bec)), closes [#1457](https://github.com/quangdang46/rust_token_cost_optimizer/issues/1457)
* **discover:** Exclude_commands bypass for env-prefix, sub cmd + regex ([ca4c59c](https://github.com/quangdang46/rust_token_cost_optimizer/commit/ca4c59c230306d310069bed3c0ba930068dc4dc4))
* **discover:** Exclude_commands bypass for env-prefix, sub cmd + regex ([42d3161](https://github.com/quangdang46/rust_token_cost_optimizer/commit/42d3161872713bc0b20ef49b0714add40c40d5e3))
* **discover:** Preserve golangci-lint flags in rewrite ([d85303e](https://github.com/quangdang46/rust_token_cost_optimizer/commit/d85303ec4893deb904260f5dc11b7df906a50c07))
* **discover:** Skip head/tail rewrite when multiple files are passed ([#1362](https://github.com/quangdang46/rust_token_cost_optimizer/issues/1362)) ([ec3a4e9](https://github.com/quangdang46/rust_token_cost_optimizer/commit/ec3a4e96fdcda6592afb08fb0ad0afaf41e07385))
* **discover:** Weighted savings rate per bucket, decimal already_rtk percent ([82c62eb](https://github.com/quangdang46/rust_token_cost_optimizer/commit/82c62eb893966b8f170ea22ec72e79f14789e12e))
* **discover:** Word boundary in exclude_commands ([0ea115b](https://github.com/quangdang46/rust_token_cost_optimizer/commit/0ea115bca5fa66daa69fda2f0eeaaf103346b3a4))
* **docker:** Forward --tail flag in compose logs ([5f1d8b0](https://github.com/quangdang46/rust_token_cost_optimizer/commit/5f1d8b0e14f0a0f82cd139443a80e680249c3137))
* **docker:** Forward --tail flag in compose logs ([b70b0fe](https://github.com/quangdang46/rust_token_cost_optimizer/commit/b70b0feec680356db81561d3920a3a9373dd43d8))
* **docs:** Add missing docs for exclude commands patterns ([2e401ac](https://github.com/quangdang46/rust_token_cost_optimizer/commit/2e401ac38feec88de8d5e46f0301c8a532b95614))
* **docs:** Replace remaining MIT license references with Apache 2.0 ([4c099e4](https://github.com/quangdang46/rust_token_cost_optimizer/commit/4c099e4b5f64793ea95e14ac3fef96125f3a6fff))
* **docs:** Replace remaining MIT license references with Apache 2.0 ([9bbc7bb](https://github.com/quangdang46/rust_token_cost_optimizer/commit/9bbc7bb79e702ada58f5cfe2499d441715aa6bbd))
* **docs:** Use release please changelog no manual ([7591a14](https://github.com/quangdang46/rust_token_cost_optimizer/commit/7591a14e4ceb732ab7ca160ac01a852926abe77a))
* **docs:** User facing docs ([c8d6878](https://github.com/quangdang46/rust_token_cost_optimizer/commit/c8d68787fb8b31c52125e9fc7ea62e0aa590485f))
* Don't inject -json for go test -bench runs ([380a7c9](https://github.com/quangdang46/rust_token_cost_optimizer/commit/380a7c9f1189fafe7d0b878b3821a720ac6ab4b2))
* Don't inject -json for go test -bench runs ([b058c96](https://github.com/quangdang46/rust_token_cost_optimizer/commit/b058c960f48535227cdec93392a70ee84f3cd2ee)), closes [#1609](https://github.com/quangdang46/rust_token_cost_optimizer/issues/1609)
* Dotnet cmd test flakiness ([17ffe62](https://github.com/quangdang46/rust_token_cost_optimizer/commit/17ffe624d415f05ca4c29e97ca650594778231be))
* **dotnet:** 🐛 format build/test/restore output sections ([106305b](https://github.com/quangdang46/rust_token_cost_optimizer/commit/106305b1978ad5fdd47139d3543cfa53a5e8172e))
* **dotnet:** 🐛 format build/test/restore output summaries ([271bc53](https://github.com/quangdang46/rust_token_cost_optimizer/commit/271bc53f35c23b39dc42002e8eb3032557f845ec))
* **dotnet:** 🐛 format warnings section in build/test/restore outputs ([c5245d7](https://github.com/quangdang46/rust_token_cost_optimizer/commit/c5245d74fafc066072615d804c27d5c2892db7d9))
* **dotnet:** Move build/test/restore status line to the bottom ([ed161b0](https://github.com/quangdang46/rust_token_cost_optimizer/commit/ed161b0a33a2a784bb933792501aa2747b0df3c3)), closes [#1574](https://github.com/quangdang46/rust_token_cost_optimizer/issues/1574)
* **filters:** Add test for aggressive filter batch fix ([f6b28c2](https://github.com/quangdang46/rust_token_cost_optimizer/commit/f6b28c292b517d55733ad1d3868f320b017901a5))
* **filters:** Address adversarial test-suite findings on aggressive filtering ([62fc0e0](https://github.com/quangdang46/rust_token_cost_optimizer/commit/62fc0e0d2159e82aaa8c36a18d69ca569a1ce0b5))
* **filters:** Aggresivity batch fix ([90c285c](https://github.com/quangdang46/rust_token_cost_optimizer/commit/90c285c38057a552f3e2ea8459fe82d715a9dd17))
* **filters:** Benchmark ci update + fix stream + filter quality ([137af04](https://github.com/quangdang46/rust_token_cost_optimizer/commit/137af0493189a86020da1feaa1de74df92466137))
* **filters:** Benchmark ci update + fix stream filter quality ([88d9f6a](https://github.com/quangdang46/rust_token_cost_optimizer/commit/88d9f6a0d94fd2b5b3d40c956e966756670a2704))
* **filters:** Split docker ps/-a paths, cap ruff violations at 50 ([f21b864](https://github.com/quangdang46/rust_token_cost_optimizer/commit/f21b8642dea5ac37ade5308bcf443315d63665e8))
* **find:** Include hidden files when pattern targets dotfiles ([#1101](https://github.com/quangdang46/rust_token_cost_optimizer/issues/1101)) ([dbeeaed](https://github.com/quangdang46/rust_token_cost_optimizer/commit/dbeeaed16aee79674ec2fd3778b7b11b10b847c6))
* **git:** Address review feedback on status state surfacing ([316e65e](https://github.com/quangdang46/rust_token_cost_optimizer/commit/316e65ef5baa6b926725b8d9a08c8d2ab52c159d))
* **git:** Compact in-progress status state ([cff391e](https://github.com/quangdang46/rust_token_cost_optimizer/commit/cff391e50b5fa89ae83eed5fd4274c7c444d37f0))
* **git:** Drop -uall from compact status so output never exceeds raw ([06476d1](https://github.com/quangdang46/rust_token_cost_optimizer/commit/06476d17cbd49a8a6d06beae9b4a9f0cb9f96f00))
* **git:** Drop -uall from compact status so output never exceeds raw ([7753e48](https://github.com/quangdang46/rust_token_cost_optimizer/commit/7753e487b3595886d39492be9b43ecad26c826ca))
* **git:** Drop state-hint extraction in compact status ([e91dee5](https://github.com/quangdang46/rust_token_cost_optimizer/commit/e91dee568bdcca0933b137edccc077db9ff006fa))
* **git:** Fix empty output when branch name contains '/' in git diff ([e070226](https://github.com/quangdang46/rust_token_cost_optimizer/commit/e0702260a94377b6bbec5cb79d91d81cba17b0ec))
* **git:** Fix empty output when branch name contains '/' in git diff ([13188a8](https://github.com/quangdang46/rust_token_cost_optimizer/commit/13188a88b22f692157b89874f4c76287a0b3ecae)), closes [#1431](https://github.com/quangdang46/rust_token_cost_optimizer/issues/1431)
* **git:** Port upstream commit hash parsing for multibyte branch names ([f6fd085](https://github.com/quangdang46/rust_token_cost_optimizer/commit/f6fd0859ef5427cdceb12c391d41c5944bf43cd1))
* **git:** Preserve full status paths and untracked files ([3ba1634](https://github.com/quangdang46/rust_token_cost_optimizer/commit/3ba1634555c0b9818560c4f512af916620946181))
* **git:** Re-insert -- separator when clap consumes it from git diff args ([#1215](https://github.com/quangdang46/rust_token_cost_optimizer/issues/1215)) ([9979c69](https://github.com/quangdang46/rust_token_cost_optimizer/commit/9979c699307a4adad2c2df0f2bc3b663df653311))
* **git:** Remove -u short alias from --ultra-compact to fix git push -u ([6b76fdb](https://github.com/quangdang46/rust_token_cost_optimizer/commit/6b76fdb87d7c54cfc2a1b0e6117dd78b8430910b))
* **git:** Resolve status completeness conflicts ([6ebde6d](https://github.com/quangdang46/rust_token_cost_optimizer/commit/6ebde6d2c277db648b6caefd7063a1bd1873fb6e))
* **git:** Stream push output to avoid spurious 30s timeout ([#963](https://github.com/quangdang46/rust_token_cost_optimizer/issues/963)) ([d6c5647](https://github.com/quangdang46/rust_token_cost_optimizer/commit/d6c56475e818b52b89906baf3a6631aaa506a4c8))
* **git:** Stream push output via FilterMode::Streaming ([#963](https://github.com/quangdang46/rust_token_cost_optimizer/issues/963)) ([be51783](https://github.com/quangdang46/rust_token_cost_optimizer/commit/be5178377fd7c155f70fda94dd134aa5a7b9361d))
* **git:** Surface in-progress state in compact `rtk git status` ([017d0f9](https://github.com/quangdang46/rust_token_cost_optimizer/commit/017d0f9ee6bb799717958d9f3fd3eee4b0e6ca3c))
* **golangci-lint:** Accept null source lines ([8eb2fa2](https://github.com/quangdang46/rust_token_cost_optimizer/commit/8eb2fa2d4eaf5dc39ff9aef2032f2a4d86ea2e32))
* **golangci-lint:** Accept null source lines (port rtk[#1969](https://github.com/quangdang46/rust_token_cost_optimizer/issues/1969)) ([22758a5](https://github.com/quangdang46/rust_token_cost_optimizer/commit/22758a57e24706e6effa0bd8ae85b0aff4cfe07d))
* **golangci-lint:** Restore run wrapper and align guidance ([4f4e4d2](https://github.com/quangdang46/rust_token_cost_optimizer/commit/4f4e4d2b5a3529030fe4089f60d2f4b8740b5d53))
* **golangci-lint:** Support inline global flags before run ([24f2ada](https://github.com/quangdang46/rust_token_cost_optimizer/commit/24f2adaf8fb541c2564fa7dfb423947932e68fb4))
* **go:** Prevent double-counted failures when test-level fail also triggers package-level fail ([#958](https://github.com/quangdang46/rust_token_cost_optimizer/issues/958)) ([4fc15ef](https://github.com/quangdang46/rust_token_cost_optimizer/commit/4fc15ef2c1c80336ffaafa4179db4cee6f39236a))
* **go:** Prevent double-counting failures when package-level fail cascades from test failures ([#958](https://github.com/quangdang46/rust_token_cost_optimizer/issues/958)) ([9722d5e](https://github.com/quangdang46/rust_token_cost_optimizer/commit/9722d5ebd8916f9b398bdc01b1102d42ab2b8795))
* **gradlew:** Satisfy semgrep dynamic-command-execution rule ([36ec02d](https://github.com/quangdang46/rust_token_cost_optimizer/commit/36ec02d132731dbed41dcf4d124bf957f06695d0))
* **gradlew:** Use resolved_command for system gradle fallback ([9e3a5ae](https://github.com/quangdang46/rust_token_cost_optimizer/commit/9e3a5ae68d4adc3d7fc374f36235cb5164e6efc8))
* Grep false negatives, output mangling, and truncation annotations ([de41533](https://github.com/quangdang46/rust_token_cost_optimizer/commit/de415335ea069c06370855366945a3704579ee18))
* Grep false negatives, output mangling, and truncation annotations ([9bdf435](https://github.com/quangdang46/rust_token_cost_optimizer/commit/9bdf435e8de3a24be213b5d5ab359bd8e7737923))
* **grep:** Adjust the command to fall through if the output would already be as small as possible ([09e1c0a](https://github.com/quangdang46/rust_token_cost_optimizer/commit/09e1c0ad4b474631b8e058ce69ca2bbd46484c7f))
* **grep:** Adjust the command to fall through if the output would already be as small as possible ([021827c](https://github.com/quangdang46/rust_token_cost_optimizer/commit/021827c3c9965f6dd4059edc380b1a605b322f39))
* **grep:** Don't print '0 matches' on real errors (exit &gt;= 2) ([b4d7862](https://github.com/quangdang46/rust_token_cost_optimizer/commit/b4d7862a065afa535a1732e9d24151da18863828))
* **grep:** NUL-separate file from line:content per review ([0e3a6f4](https://github.com/quangdang46/rust_token_cost_optimizer/commit/0e3a6f48305c1302443cbe5206e5b5f7cffb671a))
* **grep:** Parse single-file output containing colons ([be226d4](https://github.com/quangdang46/rust_token_cost_optimizer/commit/be226d4f1dcdcc0df29ad6f9968be5ab37ed5c19))
* **grep:** Parse single-file output containing colons ([#1554](https://github.com/quangdang46/rust_token_cost_optimizer/issues/1554)) ([8e04ac4](https://github.com/quangdang46/rust_token_cost_optimizer/commit/8e04ac4212ea2a7ec680b95a31e17b4c942e6209))
* Handle GNU `--format=long` and `--format=verbose` long args ([dc00951](https://github.com/quangdang46/rust_token_cost_optimizer/commit/dc00951f0b58c4a16cd7c161cc915e2d8b235f09))
* Handle GNU `--format=long` and `--format=verbose` long args ([8fcc0dc](https://github.com/quangdang46/rust_token_cost_optimizer/commit/8fcc0dc56c0bbcdf0508bbf72897085348884fd5))
* Head/tail multi-file rewrite falls back to native command ([#1362](https://github.com/quangdang46/rust_token_cost_optimizer/issues/1362)) ([f75a10b](https://github.com/quangdang46/rust_token_cost_optimizer/commit/f75a10b1a2bd824814247a03bded76fa49ddf663))
* Honor explicit -n N limit for git log on merge commits ([26c8890](https://github.com/quangdang46/rust_token_cost_optimizer/commit/26c88907d945ec81a25fe631a39dee3830faa0ec))
* Honor explicit -n N limit for git log on merge commits ([b3adcb1](https://github.com/quangdang46/rust_token_cost_optimizer/commit/b3adcb18ef762d8652272e2ea68c0f416f33c740))
* **hook:** Collapse bash line continuations before matching ([#1564](https://github.com/quangdang46/rust_token_cost_optimizer/issues/1564)) ([2543be5](https://github.com/quangdang46/rust_token_cost_optimizer/commit/2543be59126f5b6825b48570a5b5fb99d0112dab))
* **hook:** Collapse bash line continuations before matching ([#1572](https://github.com/quangdang46/rust_token_cost_optimizer/issues/1572)) ([85ad9ef](https://github.com/quangdang46/rust_token_cost_optimizer/commit/85ad9ef8ddedaafd4378334bf36d80e7046d2808)), closes [#1564](https://github.com/quangdang46/rust_token_cost_optimizer/issues/1564)
* **hook:** Collapse bash line continuations before matching ([#1572](https://github.com/quangdang46/rust_token_cost_optimizer/issues/1572)) ([39f044e](https://github.com/quangdang46/rust_token_cost_optimizer/commit/39f044efd87e5de74c60b090de69882297918dd9)), closes [#1564](https://github.com/quangdang46/rust_token_cost_optimizer/issues/1564)
* **hooks/init:** Preserve user content in copilot-instructions.md ([a04aa7e](https://github.com/quangdang46/rust_token_cost_optimizer/commit/a04aa7e848a28bf5115bfb1d6b706fbff21ea112))
* **hooks/init:** Preserve user content in copilot-instructions.md ([d108165](https://github.com/quangdang46/rust_token_cost_optimizer/commit/d10816516b4c199b06af18278ab53c76d26c2d87))
* **hooks/rewrite:** Restore php rewrite rule ([fa31090](https://github.com/quangdang46/rust_token_cost_optimizer/commit/fa310909c1530b6301b36a55636bd227420c6dcf))
* **hooks/rewrite:** Restore php rewrite rule (port rtk[#1983](https://github.com/quangdang46/rust_token_cost_optimizer/issues/1983)) ([51a3731](https://github.com/quangdang46/rust_token_cost_optimizer/commit/51a373167fa5bd7d375426623bfeba2352ba8499))
* **hooks:** Add regression test for windows native ([115e448](https://github.com/quangdang46/rust_token_cost_optimizer/commit/115e44853b8cdd2d7af3af2b52c9c31e924a45d3))
* **hooks:** Address transparent prefix review ([fdf0ed0](https://github.com/quangdang46/rust_token_cost_optimizer/commit/fdf0ed0b597f1ebdc96a2793df2725a1e62bc65c))
* **hooks:** Address transparent prefix review comments ([041de2b](https://github.com/quangdang46/rust_token_cost_optimizer/commit/041de2b6baa6a27af7d9b429d807fbe887780c90))
* **hooks:** Also rename RTK_VERSION in claude rewrite script ([aed4301](https://github.com/quangdang46/rust_token_cost_optimizer/commit/aed43012b8f3665a0e3460c38378dd787dc3bc8f))
* **hooks:** Compose env and transparent prefixes ([b234bc6](https://github.com/quangdang46/rust_token_cost_optimizer/commit/b234bc6db1ab301334412409a4cfd67fe99c58f0))
* **hooks:** Ensure default permission verdict prompts user for confirmation ([40462c0](https://github.com/quangdang46/rust_token_cost_optimizer/commit/40462c05e66f116928de365a0d271bdfd61cec72))
* **hooks:** Make Cursor preToolUse rewrites work and stay visible ([2d6e10a](https://github.com/quangdang46/rust_token_cost_optimizer/commit/2d6e10a923d18e022f5fdc4ed9b69ae0d43b2f79))
* **hooks:** Make Cursor preToolUse rewrites work and stay visible ([f00977a](https://github.com/quangdang46/rust_token_cost_optimizer/commit/f00977aa338ce6bafe8df69c271679951310b045))
* **hooks:** Rename lingering RTK_VERSION var in cursor rewrite script ([d928c3f](https://github.com/quangdang46/rust_token_cost_optimizer/commit/d928c3f1c948739de84357b46fde313a39a12484))
* **hooks:** Require all segments to match allow rules ([#1213](https://github.com/quangdang46/rust_token_cost_optimizer/issues/1213)) ([40c9dbc](https://github.com/quangdang46/rust_token_cost_optimizer/commit/40c9dbc7dbbf9332d6859060765c582a880f0fde))
* **hooks:** Windows use 'rtk hook claude' no fallback ([da3c432](https://github.com/quangdang46/rust_token_cost_optimizer/commit/da3c432201240f0da9627d8cc6bc70e5b7f8bdfe))
* **hooks:** Windows use 'rtk hook claude' no fallback ([0e29650](https://github.com/quangdang46/rust_token_cost_optimizer/commit/0e29650e11959730f4c4a2e6d6c0519e14dc8595))
* **hook:** Use maintainer regex suggestion for line continuation ([42ac86c](https://github.com/quangdang46/rust_token_cost_optimizer/commit/42ac86ce2f5bedb011f84532c2112cd5093500ab))
* **init-uninstall:** Uninstall removes --claude-md artifacts on Windows ([d395f97](https://github.com/quangdang46/rust_token_cost_optimizer/commit/d395f975c3db7e1cbc825006091e1dcc3867844d))
* **init-uninstall:** Uninstall removes --claude-md artifacts on Windows ([aad0db8](https://github.com/quangdang46/rust_token_cost_optimizer/commit/aad0db8b5213bd0940ca05f684ecda87de0d93af))
* **init:** Honor CODEX_HOME for Codex global paths ([d442799](https://github.com/quangdang46/rust_token_cost_optimizer/commit/d442799e34d522c87a6eb60c2ff373385d201315))
* **init:** Honor dry-run for Pi install and uninstall paths ([64ed4f8](https://github.com/quangdang46/rust_token_cost_optimizer/commit/64ed4f85504d94ac54e481b289e11362a5f75fdd))
* **init:** Install Codex global instructions in CODEX_HOME ([a257688](https://github.com/quangdang46/rust_token_cost_optimizer/commit/a2576883a27c5f915ba0ae7883a51006411b3ae5))
* **init:** Make --pi route to Pi-only mode and skip CLAUDE.md injection ([2f163e3](https://github.com/quangdang46/rust_token_cost_optimizer/commit/2f163e3a41cbc6e46c814b80976993a73df3bea0))
* **install.ps1:** Strip UTF-8 BOM, force TLS 1.2, ASCII-ize headers ([7fde9fc](https://github.com/quangdang46/rust_token_cost_optimizer/commit/7fde9fc2c192085abc0b33135a9893b4b1e16a32))
* **install.ps1:** Strip UTF-8 BOM, force TLS 1.2, ASCII-ize headers ([f35ce19](https://github.com/quangdang46/rust_token_cost_optimizer/commit/f35ce193f0a566bf6388ece03d2e692079fd6946))
* **install:** Allow TMP root dir in path traversal safety check ([51ccbde](https://github.com/quangdang46/rust_token_cost_optimizer/commit/51ccbde581b633de40f7624c98588f84bd5f8dd6))
* **install:** Reject archive with path traversal before extraction ([#1250](https://github.com/quangdang46/rust_token_cost_optimizer/issues/1250)) ([e827184](https://github.com/quangdang46/rust_token_cost_optimizer/commit/e8271848d7d6b0d34c2ba5c2c3783ddc48247546))
* **install:** Reject archive with path traversal before extraction ([#1250](https://github.com/quangdang46/rust_token_cost_optimizer/issues/1250)) ([ac9b22c](https://github.com/quangdang46/rust_token_cost_optimizer/commit/ac9b22c4d93f9089d30a3f28043cbe057e445f89))
* **install:** Resolve version via redirect to avoid API rate limits ([f67ae3b](https://github.com/quangdang46/rust_token_cost_optimizer/commit/f67ae3bcd2b1f0f843b969c3800ee6bf751894f4))
* **install:** Resolve version via redirect to avoid GitHub API rate limits ([5e1a641](https://github.com/quangdang46/rust_token_cost_optimizer/commit/5e1a64180f094ae456780a78b675f243312089c6))
* Isolate cursor hook tests from local settings (determinist) ([d8ddefe](https://github.com/quangdang46/rust_token_cost_optimizer/commit/d8ddefe78efe25c35bb2a2f9083f2eacb9dd7274))
* **json:** Expand char boundary truncation test ([7840030](https://github.com/quangdang46/rust_token_cost_optimizer/commit/784003055e85b5e6a51f69c2ce0b10662f1b36af))
* **json:** Rename --schema to --keys-only, closes [#621](https://github.com/quangdang46/rust_token_cost_optimizer/issues/621) ([c16713a](https://github.com/quangdang46/rust_token_cost_optimizer/commit/c16713a973b563a6cba283c830b67c8c470e419f))
* **json:** Use char boundary when truncating long string values ([533894a](https://github.com/quangdang46/rust_token_cost_optimizer/commit/533894a77ec5b8f7374547e994124bcf3a730f0b))
* **kubectl:** Compact get pods and services aliases ([2dd0ec9](https://github.com/quangdang46/rust_token_cost_optimizer/commit/2dd0ec91ab11feea13f5c40755f337208dcb3f7e))
* **kubectl:** Compact get pods and services aliases ([b8172e5](https://github.com/quangdang46/rust_token_cost_optimizer/commit/b8172e5b1de2fd3a27d992ffba484f01b47d84d4))
* **ls:** Add LC_ALL=C and fallback to raw on unrecognized locale ([0d70760](https://github.com/quangdang46/rust_token_cost_optimizer/commit/0d70760ef98749203dbaeaebe2323169436afdf3))
* **ls:** Distinguish empty dir from unparseable locale content ([25727e3](https://github.com/quangdang46/rust_token_cost_optimizer/commit/25727e343dd0eb6727d890e0e13dc53dfa08148c))
* **ls:** Handle all file types (device, pipe, socket) in ls filter ([e456be1](https://github.com/quangdang46/rust_token_cost_optimizer/commit/e456be1c1674a32839694446504310a2c16ce7dd))
* **ls:** Handle device files (block, char, pipe, socket) in ls filter ([cac8ce7](https://github.com/quangdang46/rust_token_cost_optimizer/commit/cac8ce775b695c5837b36ea788ba6812bcae214d)), closes [#844](https://github.com/quangdang46/rust_token_cost_optimizer/issues/844)
* **ls:** LC_ALL=C + fallback to raw on unrecognized locale ([bf6d4b2](https://github.com/quangdang46/rust_token_cost_optimizer/commit/bf6d4b2ea22f026d3ec4d909aef81156b0436509))
* **ls:** LC_ALL=C + fallback to raw on unrecognized locale ([b51a815](https://github.com/quangdang46/rust_token_cost_optimizer/commit/b51a815451c18b935633f6690e6c3b93a29b97f8))
* **ls:** Preserve permission info as octal when -l/-la is passed ([6c4dfc7](https://github.com/quangdang46/rust_token_cost_optimizer/commit/6c4dfc727af0423318bf21a66d53474089d25bf4)), closes [#1672](https://github.com/quangdang46/rust_token_cost_optimizer/issues/1672)
* **ls:** Preserve permission info as octal when -l/-la is passed ([#1675](https://github.com/quangdang46/rust_token_cost_optimizer/issues/1675)) ([6e76f91](https://github.com/quangdang46/rust_token_cost_optimizer/commit/6e76f911d6dfffb47fea99417ecde89df36913e4))
* Minor code cleanup, avoid duplicating logic ([20cac8a](https://github.com/quangdang46/rust_token_cost_optimizer/commit/20cac8a4e7c2b7e0e2675dbcab4fbd0fb1ad79ed))
* New rewite_command test call after rebase ([5cfb8e1](https://github.com/quangdang46/rust_token_cost_optimizer/commit/5cfb8e1d2bdf85d60633868cb420aba9a7b923f4))
* **npm:** Regex match end line ([5e84e94](https://github.com/quangdang46/rust_token_cost_optimizer/commit/5e84e9471736fe58e89094854f4123ecb07c2d3b))
* **npx:** Dispatch unknown tools to npx instead of npm ([2c4569c](https://github.com/quangdang46/rust_token_cost_optimizer/commit/2c4569caa64d013ad4ada0b7580f9f16d8334c19)), closes [#815](https://github.com/quangdang46/rust_token_cost_optimizer/issues/815)
* P0+P1 fixes from pre-merge review of hook engine ([df8e035](https://github.com/quangdang46/rust_token_cost_optimizer/commit/df8e03558d4d6cc2f5cbac91c63ab1b3b51d3bcd))
* P0+P1 fixes from pre-merge review of hook engine ([d34389c](https://github.com/quangdang46/rust_token_cost_optimizer/commit/d34389c3d0936c2b0790e14f450bb50a28a7edf7))
* **permissions:** Glob_matches middle-wildcard matches commands without trailing args ([#1105](https://github.com/quangdang46/rust_token_cost_optimizer/issues/1105)) ([3db8070](https://github.com/quangdang46/rust_token_cost_optimizer/commit/3db8070b51b9a312fcca20a8460d3d6259cc38b7))
* **permissions:** Normalize whitespace to prevent deny-rule evasions ([0c934a5](https://github.com/quangdang46/rust_token_cost_optimizer/commit/0c934a5e52453f086a70fdcdc748165de14ad70e))
* **pkg:** Rtk is Apache 2.0 and no MIT ([fa11c6c](https://github.com/quangdang46/rust_token_cost_optimizer/commit/fa11c6c3ebe55f36a0c09f15995333cda19a737a))
* **pkg:** Rtk is Apache 2.0 and no MIT ([1875945](https://github.com/quangdang46/rust_token_cost_optimizer/commit/1875945bf32b3cc8f7f4c8da81427e17ded74c7f))
* **pnpm:** Install don't take a list of packages ([492aa76](https://github.com/quangdang46/rust_token_cost_optimizer/commit/492aa76ed3842549d2a453becbf2782caba765f1))
* **pnpm:** Install don't take a list of packages ([9a50efa](https://github.com/quangdang46/rust_token_cost_optimizer/commit/9a50efadc38c71cc4a0f3cb05b46cbe41d925a0c))
* **pnpm:** List command not working ([ba235d8](https://github.com/quangdang46/rust_token_cost_optimizer/commit/ba235d85974c0a85b25e290a8bb83648800438a6))
* **PR#2101:** Complete RTK→RTCO rename in hook_cmd.rs, rules.rs, registry.rs ([577b13f](https://github.com/quangdang46/rust_token_cost_optimizer/commit/577b13f98fc7b6d75420ca55cab96470e1f71936))
* Purge stale rtk references across codebase ([b284851](https://github.com/quangdang46/rust_token_cost_optimizer/commit/b28485112e48caa3d23db0c5678bdf9d814a0f08))
* **pytest:** -q mode summary line not detected ([57502a5](https://github.com/quangdang46/rust_token_cost_optimizer/commit/57502a5bef1fb56109a57cf2ea7377fd271253a7))
* **python:** Port upstream fail-report + ANSI strip + ruff output-format ([e180f18](https://github.com/quangdang46/rust_token_cost_optimizer/commit/e180f1861b505c0fe04596ce6c606b82a87924cc))
* Re-add env python as noisy dir ([4eefe2f](https://github.com/quangdang46/rust_token_cost_optimizer/commit/4eefe2f225ea512a2f1bf800dd20c09994721108))
* README master→main URLs + fix tracking PoisonError in tests ([ba94f52](https://github.com/quangdang46/rust_token_cost_optimizer/commit/ba94f52d7c4a2b65c1e615e233271ce941aa90ea))
* **release:** Include v prefix in artifact names for install.sh compat ([975751f](https://github.com/quangdang46/rust_token_cost_optimizer/commit/975751f8b2135b85ce6d2b0e1f06b0b3a4a75964))
* **release:** Name artifacts with os-arch format for install script compat ([74c5dd2](https://github.com/quangdang46/rust_token_cost_optimizer/commit/74c5dd23c33e845c30ef22bc1b129b21f4e9b471))
* **release:** Use nullglob for cross-platform SHA256 checksums ([6dc12c1](https://github.com/quangdang46/rust_token_cost_optimizer/commit/6dc12c1645f49a3319976330e730688cca7466b6))
* Remove duplicate test functions from cherry-pick ([f043515](https://github.com/quangdang46/rust_token_cost_optimizer/commit/f043515126e5d8ae5ce4d766d291f58f07fe0217))
* Remove pnpx -&gt; rtk pnpx rule as rtk pnpx command doesn't exist ([325a42e](https://github.com/quangdang46/rust_token_cost_optimizer/commit/325a42e09fd8de6d0c26914ff29ab11d8680f354))
* Remove remaining noise and rework comments ([75df4c5](https://github.com/quangdang46/rust_token_cost_optimizer/commit/75df4c5804430b56c81f437f15c74217708c485c))
* Remove remaining noise and rework comments ([3c356b3](https://github.com/quangdang46/rust_token_cost_optimizer/commit/3c356b32416be0815bee9e4364c20c4efa15d829))
* Remove wrong cicd benchmark + npm test regex ([7e3690a](https://github.com/quangdang46/rust_token_cost_optimizer/commit/7e3690a23ab158ca8e1e890650554e20e3a0c17b))
* Rename ship.md to ship/SKILL.md to match develop ([5916ecd](https://github.com/quangdang46/rust_token_cost_optimizer/commit/5916ecd86fb319c2519a0b4fb2891309833a3bb4))
* Replace PGPASSWORD test fixture with PGHOST in registry tests ([87ac19d](https://github.com/quangdang46/rust_token_cost_optimizer/commit/87ac19d2a65ce34320b008e2140cff29a7933f5c))
* Replace remaining MIT license references with Apache 2.0 across all README files ([7954304](https://github.com/quangdang46/rust_token_cost_optimizer/commit/7954304b938be59135f28b2f2318c311be5892e4))
* Resolve CI/CD pipeline issues ([75ca0d3](https://github.com/quangdang46/rust_token_cost_optimizer/commit/75ca0d3c3a8886a6784336e066a7f24417a6893f))
* Resolve fork issues [#39](https://github.com/quangdang46/rust_token_cost_optimizer/issues/39)-[#44](https://github.com/quangdang46/rust_token_cost_optimizer/issues/44) ([e94bf7f](https://github.com/quangdang46/rust_token_cost_optimizer/commit/e94bf7fa58a3e2baa20df1af0b102517d3cbc744))
* Resolve merge conflict artifacts in init.rs ([4830d50](https://github.com/quangdang46/rust_token_cost_optimizer/commit/4830d50f6e3ad7adbd24ba11f3e392869723a020))
* Resolve remaining fork issues [#40](https://github.com/quangdang46/rust_token_cost_optimizer/issues/40) and [#42](https://github.com/quangdang46/rust_token_cost_optimizer/issues/42) ([f002fa9](https://github.com/quangdang46/rust_token_cost_optimizer/commit/f002fa982894ccbd96b6344dbf84d4fc0b5d2786))
* Restore working hook files from agy-pr to resolve CI/CD build failures ([5d990b1](https://github.com/quangdang46/rust_token_cost_optimizer/commit/5d990b11eff5d03c908bac1afe08d75b028baaa9))
* **review:** Address all review-swarm findings ([290e177](https://github.com/quangdang46/rust_token_cost_optimizer/commit/290e177460a8cb5a8afd50d5ae9848826d622238))
* **rtco:** Comprehensive rtk→rtco rename in user-facing strings and package metadata ([f6cd4e1](https://github.com/quangdang46/rust_token_cost_optimizer/commit/f6cd4e193f1bcf3bbc53f5be77bc6a0c04b3c66e))
* **rtco:** Rename all rtk→rtco in user-facing strings and file paths ([181c892](https://github.com/quangdang46/rust_token_cost_optimizer/commit/181c8923eb066fcf0db5e9a9dbd930bc831b571e))
* **rtco:** Rename package rtk→rtco in Cargo.toml + deb/rpm assets + build.rs comment ([44cf9ca](https://github.com/quangdang46/rust_token_cost_optimizer/commit/44cf9cab2e7ac80ab2cbab349966665271e37c20))
* Rtk_cmd→rtco_cmd in registry.rs after upstream cherry-pick ([15c3dc1](https://github.com/quangdang46/rust_token_cost_optimizer/commit/15c3dc1bd9cd716562aed137553714f61a67b6e7))
* **runner:** Preserve fd separation on command failure ([e92d099](https://github.com/quangdang46/rust_token_cost_optimizer/commit/e92d0993c93f0b732316dfa932d265aeca7488d6))
* **rust:** Multi-line blocks used with tail hint ([4960630](https://github.com/quangdang46/rust_token_cost_optimizer/commit/49606303d6738525c250149230752fb6133383d1))
* **security:** Pin workflow actions to SHA, clean up tempfile on failure ([26b96ec](https://github.com/quangdang46/rust_token_cost_optimizer/commit/26b96ec6c4f40f992ccffa190af9a4de8d7636b1))
* **security:** Replace insecure tmp, lock git perm, set sha for actions ([54d1f87](https://github.com/quangdang46/rust_token_cost_optimizer/commit/54d1f8736f4acdd0667eb86c81d0e4c7843306f4))
* **security:** Replace insecure tmp, lock git workflow perm ([cd6ac2f](https://github.com/quangdang46/rust_token_cost_optimizer/commit/cd6ac2f47a008c6dca04b567faf68aaedfd87ca9))
* **stream:** Add semgrep flag for sh tests ([7cfcdbe](https://github.com/quangdang46/rust_token_cost_optimizer/commit/7cfcdbec8681b15b794b6aef982ccb38feb79fd7))
* **stream:** Add semgrep flag for sh tests ([d327724](https://github.com/quangdang46/rust_token_cost_optimizer/commit/d327724f814b6875903366db0b0616780b454ad1))
* **stream:** Missing stderr fields ([a1d46f3](https://github.com/quangdang46/rust_token_cost_optimizer/commit/a1d46f39c291e3356b9c26a062bde05ba1de591a))
* **stream:** P0 fixes from PR [#956](https://github.com/quangdang46/rust_token_cost_optimizer/issues/956) review ([2bb5265](https://github.com/quangdang46/rust_token_cost_optimizer/commit/2bb5265595c4a80fe1ad7e9ab3ffc8dd013b019c))
* **stream:** Route to respective fd ([605e335](https://github.com/quangdang46/rust_token_cost_optimizer/commit/605e335f0546d2ed8554a95e7749a0b494c510e3))
* **stream:** Route to respective fd ([81a1be6](https://github.com/quangdang46/rust_token_cost_optimizer/commit/81a1be6a744942515347dd296ddcf7d9f126200d))
* **stream:** Stream engine & template + BlockHandler, RegexBlockFilter, signal guard, pipe rewrite ([b3936b8](https://github.com/quangdang46/rust_token_cost_optimizer/commit/b3936b8e5d5fc2b5036b90aada61917fa3f3f7de))
* **tee:** Make home-relative path test use real home dir ([0fc0a2a](https://github.com/quangdang46/rust_token_cost_optimizer/commit/0fc0a2a31d15a072219a55416a73e8433fdd6fd6))
* **tee:** Safe truncation caps and compose-ps tee content fix ([548e4dd](https://github.com/quangdang46/rust_token_cost_optimizer/commit/548e4dd995d5de6e52d7c8e7bb0a0f81fa2c0328))
* **tee:** Safe truncation caps and tee/hint coverage ([15a0d2e](https://github.com/quangdang46/rust_token_cost_optimizer/commit/15a0d2e7d6e3f33442675f502ed8bc868710dfd6))
* **tee:** Serialize env mutations in preview tests (Windows CI) ([8fef61e](https://github.com/quangdang46/rust_token_cost_optimizer/commit/8fef61ee1068772c2ca75fbb5f6ccab094c5b9af))
* **telemetry:** Clean code ([8156081](https://github.com/quangdang46/rust_token_cost_optimizer/commit/81560812610686fa5ca3633c2bf0b79c05eaa7d9))
* **telemetry:** Consent, erasure, auth, docs ([2e4cc4b](https://github.com/quangdang46/rust_token_cost_optimizer/commit/2e4cc4bb5226444c8c0bfc827baf0c101c3759e8))
* **telemetry:** Non-terminal consent, single config load ([7821e98](https://github.com/quangdang46/rust_token_cost_optimizer/commit/7821e9872fd1f1ae9b40eb8a4458049869acc36b))
* **telemetry:** RGPD-compliant, consent gate, erasure, privacy controls ([6a5bc84](https://github.com/quangdang46/rust_token_cost_optimizer/commit/6a5bc847e06cf6066e6f4aeed5a3ad0803a3649b))
* **tests:** Accept updated mypy snapshot + all 2010 tests passing ([577aedd](https://github.com/quangdang46/rust_token_cost_optimizer/commit/577aedd9a8bffef674517645eafd935c006c02f3))
* **tests:** Accept updated snapshots — all 2010+651+46 tests passing ([0e55624](https://github.com/quangdang46/rust_token_cost_optimizer/commit/0e55624215d03d00cdf51536b9b9a23be9b93aec))
* **tests:** Deterministic snapshot ordering + update snapshots ([4506fdd](https://github.com/quangdang46/rust_token_cost_optimizer/commit/4506fdd33714052907f97bd2b5b8e3507d05d450))
* **tests:** Repair 6 pre-existing doc tests + stabilize 4 flaky tracking tests ([490fd64](https://github.com/quangdang46/rust_token_cost_optimizer/commit/490fd64a6fde36cb53f4a6af82d0749fb8d15b48))
* **tests:** Update init.rs test calls for dry_run parameter ([add1464](https://github.com/quangdang46/rust_token_cost_optimizer/commit/add14643886e1b2c5450056433e577c003872e4d))
* **tests:** Use rewrite_command_no_prefixes in line-continuation tests ([0ec99de](https://github.com/quangdang46/rust_token_cost_optimizer/commit/0ec99de8f4ef102c10a827b57636278080e2f3cf))
* **tests:** Windows regression test fix path ([13a73dd](https://github.com/quangdang46/rust_token_cost_optimizer/commit/13a73ddfd78460560a1f5fde94b54b1f848b41b5))
* **tracking:** Isolate parallel tests by extracting with_db_path constructor ([6e6241f](https://github.com/quangdang46/rust_token_cost_optimizer/commit/6e6241f75fa7e532070ad95bb9643f0e097ebfcb))
* **tracking:** Test env path ([70b36b4](https://github.com/quangdang46/rust_token_cost_optimizer/commit/70b36b4dbc3e147219ad87cf539d073523b86a85))
* **tracking:** Unify env locks to stop macOS CI race ([9639f8a](https://github.com/quangdang46/rust_token_cost_optimizer/commit/9639f8a1bb00a600372f1655b4edcbdd59a1d318))
* **truncate:** Global caps reduce (avoid underflow and 0 results) ([d5a1731](https://github.com/quangdang46/rust_token_cost_optimizer/commit/d5a17315c52487be2d043e0058a4f7d91ec3d2bc))
* Update all test assertions and fixtures for rtk-&gt;rtco rename (1928 tests pass) ([06e1daf](https://github.com/quangdang46/rust_token_cost_optimizer/commit/06e1daffadc3bb6b754e41987a67de041e6d38c3))
* **uv:** Preserve program output on success, improve failure handling ([9d38794](https://github.com/quangdang46/rust_token_cost_optimizer/commit/9d38794381f69a336a2420c7e744b9fbcb684116))
* **verify:** Correct fnm list-remote truncation test fixture ([04386d0](https://github.com/quangdang46/rust_token_cost_optimizer/commit/04386d0ed76563a8d9d515647916050989cf2fc4))
* **vitest:** Rework command to handle differences between vitest and jest ([70610da](https://github.com/quangdang46/rust_token_cost_optimizer/commit/70610da4bbf0fd8f4226fc61895af61377eafcc8))


### Security

* Security:  ([87f678b](https://github.com/quangdang46/rust_token_cost_optimizer/commit/87f678bd6274c7c564cce2d56d065c9ea5314c28))

## [Unreleased]

## [0.2.5](https://github.com/quangdang46/rust_token_cost_optimizer/compare/v0.2.4...v0.2.5) (2026-07-14)

### Features
* **core:** add `never_worse` output guard so RTCO never emits more tokens than raw command output
* **curl:** binary download passthrough (raw bytes, skip UTF-8 lossy conversion) (#1087)
* **hooks:** detect absolute `rtco hook claude` paths in settings; honour `CLAUDE_CONFIG_DIR`
* **uv:** add `rtco uv` filter for `uv run` failure-focused output
* **php:** add PHP ecosystem filters — php, phpunit, phpstan, pest, paratest, ecs, pint
* **php:** normalize `vendor/bin/*` / `php vendor/bin/*` rewrites to bare tool names
* **filters:** add Pulumi TOML filters (up/preview/refresh/destroy/stack)
* **discover:** rewrite rules for uv run and dedicated PHP tools

### Bug Fixes
* **tracking:** unify ENV_LOCK to stop macOS CI race with `RTCO_TRACK=0` gate tests

## [0.2.3](https://github.com/quangdang46/rust_token_cost_optimizer/compare/v0.2.2...v0.2.3) (2026-06-23)

### Bug Fixes
* **ci:** fix install.sh/install.ps1 branch references — master → main
* **ci:** add deny.toml for supply chain audit (cargo-deny)
* **ci:** fix cargo test flags — --all-features → --features prometheus
* **tracking:** fix PoisonError in test mutex — unwrap() → unwrap_or_else(Into::into)

### Documentation
* **readme:** update install URLs from master to main branch

### Chores
* **ci:** add cargo-deny check step to CI workflow

## [0.2.2](https://github.com/quangdang46/rust_token_cost_optimizer/compare/v0.2.1...v0.2.2) (2026-06-19)

### Bug Fixes
* **mcp:** fix MCP protocol compliance — ToolDescription.input_schema now serializes as inputSchema (camelCase) per spec. Resolves 'expected object, received undefined' error in MCP clients.

## [0.2.2](https://github.com/quangdang46/rust_token_cost_optimizer/compare/v0.2.1...v0.2.2) (2026-06-19)

### Bug Fixes
* **mcp:** fix MCP protocol compliance —  now serializes as  (camelCase) per spec. Resolves  error in MCP clients.

## [0.2.1](https://github.com/quangdang46/rust_token_cost_optimizer/compare/v0.2.0...v0.2.1) (2026-06-19)

### Documentation
* **changelog:** move prematurely listed [Unreleased] entries into v0.2.0 section
* **version:** bump workspace and crate versions from 0.2.0 to 0.2.1

## [0.2.0](https://github.com/quangdang46/rust_token_cost_optimizer/compare/v0.1.1...v0.2.0) (2026-06-18)

### Features
* **cli:** add `rtco mcp` subcommand to expose `rtco_compress`, `rtco_analyze`, and `rtco_retrieve` as MCP tools over JSON-RPC stdio. Shares code with the standalone `rtco-mcp` binary.
* **cli:** add `rtco init --mcp` / `rtco init --hooks` / `rtco init --uninstall --mcp` to register (or strip) the `rtco` MCP server entry in 10 provider config files: Claude Code, Cursor, Cline, Windsurf, VS Code Copilot, OpenCode, Codex CLI, Gemini CLI, Amazon Q, and Warp.
* **cli:** new `McpTarget` enum, `run_mcp_install`, `run_mcp_uninstall`, and 14 unit tests covering per-provider write shape, deep-merge preservation, `.rtco.bak` backup, dry-run, and uninstall symmetry.
* **install:** install.sh and install.ps1 gain `--with-mcp`, `--no-mcp`, `--with-hooks`, `--no-hooks`, `--provider`, `--all-providers`, and `--dry-run` flags. The `configure_post_install` / `Invoke-PostInstallConfig` step probes which provider config files exist on disk and registers MCP+hooks only in those.
* **install:** `--uninstall` now also calls `rtco init --uninstall --mcp --hooks --all-providers` (best-effort) to strip the rtco entry from every detected provider before removing the binary.
* **test:** scripts/test-install.sh extended with help-text, arg-parser, configure_post_install, and uninstall-cleanup regression tests. New scripts/test-install.ps1 PowerShell mirror.

### Bug Fixes
* **hooks:** the `rtco mcp` server is now callable from inside the main `rtco` binary via a thin `bin_shim` module that re-includes `src/bin/mcp_server.rs` with `#[path]`. Single source of truth, no copy-paste.

### Changed
* **install:** default behavior is unchanged when no new flag is passed — opt-in only.


## [0.1.1](https://github.com/quangdang46/rust_token_cost_optimizer/compare/v0.1.0...v0.1.1) (2026-06-18)

### Bug Fixes
* **ci:** add missing TEE_ENV_LOCK guard to flaky Windows test
* **hooks:** rename rtk hooks to rtco, add AGENTS.md, add OpenCode plugin

## [0.41.0](https://github.com/rtco-ai/rtco/compare/v0.40.0...v0.41.0) (2026-06-16)

### Features

* **headroom:** port token estimation engine with ApproximateEstimator, Tokenizer trait, and Registry ([ceabad0](https://github.com/rtco-ai/rtco/commit/ceabad0))
* **headroom:** port line importance signals — ErrorWarningDetector (Aho-Corasick), SeparatorDetector, LengthDetector, TieredDetector ([ceabad0](https://github.com/rtco-ai/rtco/commit/ceabad0))
* **headroom:** port CCR pipeline, compressor framework, anchor system, and cache aligner ([ceabad0](https://github.com/rtco-ai/rtco/commit/ceabad0))
* **headroom:** port smart_crusher depth — anchors, constraints, error_keywords, field_detect, hashing, outliers, traits, tag_protector, upgrade planner (SmartCrusherConfig) ([89f20da](https://github.com/rtco-ai/rtco/commit/89f20da))
* **headroom:** port Shannon entropy, SimHash, dedup filter, text_stats, content-aware token estimation ([cc4ba76](https://github.com/rtco-ai/rtco/commit/cc4ba76))
* **workspace:** extract rtco-core library crate + workspace structure ([37dcb0c](https://github.com/rtco-ai/rtco/commit/37dcb0c))
* **jvm:** add `rtco mvn` with Surefire/Failsafe XML test summarization (PASS/FAIL/SKIP + failure details) ([d8f1677](https://github.com/rtco-ai/rtco/commit/d8f1677))
* **sqlite:** add `rtco sqlite3` filter with column/line/list mode table compression ([35535db](https://github.com/rtco-ai/rtco/commit/35535db))
* **rewrite:** port upstream RTK improvements — SIGPIPE handler, args_utils, permission hardening ([c515279](https://github.com/rtco-ai/rtco/commit/c515279))
* **tee:** add force_tee_hint() — truncated output saved to file with recovery hint ([ceabad0](https://github.com/rtco-ai/rtco/commit/ceabad0))
* **hooks:** add Pi coding agent integration ([1da5793](https://github.com/rtco-ai/rtco/commit/1da5793))
* **hooks:** add fnm (Fast Node Manager) filter and hook rewrite ([70b8902](https://github.com/rtco-ai/rtco/commit/70b8902))
* **discover:** add fnm rewrite rule and filter ([70b8902](https://github.com/rtco-ai/rtco/commit/70b8902))
* **git:** prefer short args, cleanup comments ([fb67bb6](https://github.com/rtco-ai/rtco/commit/fb67bb6))
* **security:** Sprint 1 — infrastructure fixes: hook integrity, permission hardening, stale file removal ([b3b5c40](https://github.com/rtco-ai/rtco/commit/b3b5c40))
* **tests:** Sprint 2 — critical test coverage: add #[cfg(test)] modules for verify_cmd, gain, telemetry, discover, learn, parser, deps, summary ([118eca0](https://github.com/rtco-ai/rtco/commit/118eca0))
* **tests:** Sprint 3 — test fixtures for all filter modules (81 new fixture files across git, js, python, ruby, system, go, jvm) ([1d048a6](https://github.com/rtco-ai/rtco/commit/1d048a6))
* **tests:** Sprint 4 — snapshot + token savings tests across all filter modules with >=60% savings assertions ([7a027e5](https://github.com/rtco-ai/rtco/commit/7a027e5))
* **ci:** Sprint 5 — CI/CD pipeline automation, workflow cleanup, branch trigger migration ([8d54e76](https://github.com/rtco-ai/rtco/commit/8d54e76))
* **bead-analysis:** add full gap analysis — 50 beads across 7 sprints ([cff7ef9](https://github.com/rtco-ai/rtco/commit/cff7ef9))
* **tracking:** honor tracking.enabled config and redact sensitive args from database ([37f239a](https://github.com/rtco-ai/rtco/commit/37f239a))
* **tee:** redact sensitive output and add per-command opt-out ([0f12931](https://github.com/rtco-ai/rtco/commit/0f12931))

### Bug Fixes

* **diff:** preserve POSIX/git contract for programmatic consumers (git apply, patch, shell loops) ([fb5bf71](https://github.com/rtco-ai/rtco/commit/fb5bf71))
* **golangci-lint:** accept null source lines in JSON output ([22758a5](https://github.com/rtco-ai/rtco/commit/22758a5))
* **hooks/rewrite:** restore php rewrite rule (Laravel/Symfony artisan) ([51a3731](https://github.com/rtco-ai/rtco/commit/51a3731))
* **core:** surface truncated context inline so AI consumers get full diagnostics ([f557bb0](https://github.com/rtco-ai/rtco/commit/f557bb0))
* **git:** drop -uall from compact status so output never exceeds raw ([7753e48](https://github.com/rtco-ai/rtco/commit/7753e48))
* **git:** honor explicit -n N limit for git log on merge commits ([b3adcb1](https://github.com/rtco-ai/rtco/commit/b3adcb1))
* **git:** don't count --max-lines truncation as savings ([70b8902](https://github.com/rtco-ai/rtco/commit/70b8902))
* **grep:** parse single-file output containing colons ([8e04ac4](https://github.com/rtco-ai/rtco/commit/8e04ac4))
* **grep:** NUL-separate file from line:content for robust parsing ([0e3a6f4](https://github.com/rtco-ai/rtco/commit/0e3a6f4))
* **ls:** preserve permission info as octal when -l/-la is passed ([6c4dfc7](https://github.com/rtco-ai/rtco/commit/6c4dfc7))
* **hook:** collapse bash line continuations before matching ([#1572](https://github.com/rtco-ai/rtco/issues/1572)) ([85ad9ef](https://github.com/rtco-ai/rtco/commit/85ad9ef))
* **hook:** respect Claude Code deny/ask permission rules on rewrite ([a051a6f](https://github.com/rtco-ai/rtco/commit/a051a6f))
* **hook:** require all segments to match allow rules ([#1213](https://github.com/rtco-ai/rtco/issues/1213)) ([40c9dbc](https://github.com/rtco-ai/rtco/commit/40c9dbc))
* **rewrite:** only rewrite find when invocation fits compact-find grammar ([70b8902](https://github.com/rtco-ai/rtco/commit/70b8902))
* **init:** respect CLAUDE_CONFIG_DIR for global paths ([70b8902](https://github.com/rtco-ai/rtco/commit/70b8902))
* **init:** preserve settings.json symlink during atomic write ([70b8902](https://github.com/rtco-ai/rtco/commit/70b8902))
* **tracking:** isolate parallel tests with with_db_path constructor ([6e6241f](https://github.com/rtco-ai/rtco/commit/6e6241f))
* **tracking:** use std::env::temp_dir() for cross-platform compatibility ([e918661](https://github.com/rtco-ai/rtco/commit/e918661))
* **ci:** correct include_str! fixture path from crate root to workspace root ([92a23ce](https://github.com/rtco-ai/rtco/commit/92a23ce))
* **ci:** use include_str! for dotnet format test fixtures ([9f4a4df](https://github.com/rtco-ai/rtco/commit/9f4a4df))
* **ci:** clean up CD workflow — remove release-please, stale upstream workflows, fix concurrency ([a61602f](https://github.com/rtco-ai/rtco/commit/a61602f))
* **ci:** change branch triggers master→main ([2e6b687](https://github.com/rtco-ai/rtco/commit/2e6b687))
* **aws:** redact secretsmanager get-secret-value payload from output ([bc936fd](https://github.com/rtco-ai/rtco/commit/bc936fd))
* **install:** fix install.ps1 — strip UTF-8 BOM, force TLS 1.2, ASCII-ize headers for PS 5.1 ([f35ce19](https://github.com/rtco-ai/rtco/commit/f35ce19))
* **license:** replace remaining MIT references with Apache 2.0 across all README files ([7954304](https://github.com/rtco-ai/rtco/commit/7954304))
* remove stale files: translated READMEs (es/fr/ja/ko/zh), SECURITY, DISCLAIMER, FEATURES ([4a0a3f4](https://github.com/rtco-ai/rtco/commit/4a0a3f4))

### Documentation

* add ASCII art banner, Mermaid architecture diagram, filter pipeline section ([8f0d1b9](https://github.com/rtco-ai/rtco/commit/8f0d1b9))
* rewrite README — minimal, concise, single-page with design philosophy ([092d73b](https://github.com/rtco-ai/rtco/commit/092d73b))
* remove nested/translation READMEs, keep only root README ([ce9b234](https://github.com/rtco-ai/rtco/commit/ce9b234))

### Refactor

* comprehensive rtk→rtco rename: binary, package, env vars, hooks, docs, data dir ([8dbaa2e](https://github.com/rtco-ai/rtco/commit/8dbaa2e))
* rename rtk_* identifiers to rtco_* across codebase ([2333da2](https://github.com/rtco-ai/rtco/commit/2333da2))

## [0.36.0](https://github.com/rtco-ai/rtco/compare/v0.35.0...v0.36.0) (2026-04-13)


### Features

* **benchmark:** add multipass VM integration test suite ([6e7863b](https://github.com/rtco-ai/rtco/commit/6e7863bf313b0d18a47cf0ca2cdaea03cc2ed900))
* **benchmark:** add multipass VM integration test suite ([d22759b](https://github.com/rtco-ai/rtco/commit/d22759b8c5254ad9c4a455f10cb7de75e92df581))
* **benchmark:** add Swift ecosystem tests (6 commands + savings) ([1fbb6d9](https://github.com/rtco-ai/rtco/commit/1fbb6d935b4a0d031a7862cba312eebe1303ba9b))
* **init:** add native support for Kilo Code and Google Antigravity ([d0a3797](https://github.com/rtco-ai/rtco/commit/d0a3797ec580f96948489d1e7c3329ac22a6c4eb))
* **init:** add support for kilocode and antigravity agents ([66b90f1](https://github.com/rtco-ai/rtco/commit/66b90f1ed3de81acdce61164c068c24ed7ef29db))
* **pnpm:** Add filter argument support ([2ba8d37](https://github.com/rtco-ai/rtco/commit/2ba8d372df186b4056a3b8906fc25cde8586dd42))
* **skills:** add /pr-review skill for batch PR review ([21e67a1](https://github.com/rtco-ai/rtco/commit/21e67a1113041b74542d0285e5f74587dfb30b65))
* **telemetry:** enrich daily ping with gap detection and quality metrics ([644c50f](https://github.com/rtco-ai/rtco/commit/644c50f786e5c567617e7ea907c5f312797b1265))


### Bug Fixes

* **benchmark:** address PR review feedback ([87ee81f](https://github.com/rtco-ai/rtco/commit/87ee81f08be5e7b1ca79513b1a91925d455f4f5c))
* **benchmark:** address review feedback from @FlorianBruniaux ([d13c185](https://github.com/rtco-ai/rtco/commit/d13c185aac64d14288b574df44623723a69e7b95))
* **ccusage:** add --yes flag and warn when falling back to npx ([f68fa00](https://github.com/rtco-ai/rtco/commit/f68fa0087c03d6882993b7b3eaee98e1dbab41b4))
* **clippy:** show full error blocks instead of truncated headline ([95d9d13](https://github.com/rtco-ai/rtco/commit/95d9d134b0b76d83b6162614b0a79269b2135f40))
* **clippy:** show full error blocks instead of truncated headline ([f4074f8](https://github.com/rtco-ai/rtco/commit/f4074f898a9b73b72bbcd8b18afab4831dcda406)), closes [#602](https://github.com/rtco-ai/rtco/issues/602)
* **curl:** skip JSON schema conversion for internal/localhost URLs ([577c311](https://github.com/rtco-ai/rtco/commit/577c311ecaaa8ae94f22dbe252152424d4333d04))
* **discover:** preserve golangci-lint flags in rewrite ([d85303e](https://github.com/rtco-ai/rtco/commit/d85303ec4893deb904260f5dc11b7df906a50c07))
* **docs:** update TELEMETRY.md to match code after review fixes ([be5c057](https://github.com/rtco-ai/rtco/commit/be5c0576d95566f37f266fd9f92e2a1b263697bd))
* **find:** include hidden files when pattern targets dotfiles ([#1101](https://github.com/rtco-ai/rtco/issues/1101)) ([dbeeaed](https://github.com/rtco-ai/rtco/commit/dbeeaed16aee79674ec2fd3778b7b11b10b847c6))
* **git:** re-insert -- separator when clap consumes it from git diff args ([#1215](https://github.com/rtco-ai/rtco/issues/1215)) ([9979c69](https://github.com/rtco-ai/rtco/commit/9979c699307a4adad2c2df0f2bc3b663df653311))
* **git:** remove -u short alias from --ultra-compact to fix git push -u ([6b76fdb](https://github.com/rtco-ai/rtco/commit/6b76fdb87d7c54cfc2a1b0e6117dd78b8430910b))
* **golangci-lint:** restore run wrapper and align guidance ([4f4e4d2](https://github.com/rtco-ai/rtco/commit/4f4e4d2b5a3529030fe4089f60d2f4b8740b5d53))
* **golangci-lint:** support inline global flags before run ([24f2ada](https://github.com/rtco-ai/rtco/commit/24f2adaf8fb541c2564fa7dfb423947932e68fb4))
* **go:** prevent double-counted failures when test-level fail also triggers package-level fail ([#958](https://github.com/rtco-ai/rtco/issues/958)) ([4fc15ef](https://github.com/rtco-ai/rtco/commit/4fc15ef2c1c80336ffaafa4179db4cee6f39236a))
* **go:** prevent double-counting failures when package-level fail cascades from test failures ([#958](https://github.com/rtco-ai/rtco/issues/958)) ([9722d5e](https://github.com/rtco-ai/rtco/commit/9722d5ebd8916f9b398bdc01b1102d42ab2b8795))
* **hooks:** ensure default permission verdict prompts user for confirmation ([40462c0](https://github.com/rtco-ai/rtco/commit/40462c05e66f116928de365a0d271bdfd61cec72))
* **hooks:** require all segments to match allow rules ([#1213](https://github.com/rtco-ai/rtco/issues/1213)) ([40c9dbc](https://github.com/rtco-ai/rtco/commit/40c9dbc7dbbf9332d6859060765c582a880f0fde))
* **init:** honor CODEX_HOME for Codex global paths ([d442799](https://github.com/rtco-ai/rtco/commit/d442799e34d522c87a6eb60c2ff373385d201315))
* **init:** install Codex global instructions in CODEX_HOME ([a257688](https://github.com/rtco-ai/rtco/commit/a2576883a27c5f915ba0ae7883a51006411b3ae5))
* **json:** rename --schema to --keys-only, closes [#621](https://github.com/rtco-ai/rtco/issues/621) ([c16713a](https://github.com/rtco-ai/rtco/commit/c16713a973b563a6cba283c830b67c8c470e419f))
* **ls:** filter quality wrong truncation ([aa6317f](https://github.com/rtco-ai/rtco/commit/aa6317fb83a5d9883623a4d3bee7a25bc99dcb4c))
* **permissions:** glob_matches middle-wildcard matches commands without trailing args ([#1105](https://github.com/rtco-ai/rtco/issues/1105)) ([3db8070](https://github.com/rtco-ai/rtco/commit/3db8070b51b9a312fcca20a8460d3d6259cc38b7))
* **pnpm:** list command not working ([ba235d8](https://github.com/rtco-ai/rtco/commit/ba235d85974c0a85b25e290a8bb83648800438a6))
* **pytest:** -q mode summary line not detected ([57502a5](https://github.com/rtco-ai/rtco/commit/57502a5bef1fb56109a57cf2ea7377fd271253a7))
* report package-level failures (timeouts, signals) in go test summary ([0b1c32b](https://github.com/rtco-ai/rtco/commit/0b1c32b3cc9a3e73418d401d1d481c1611c7ec0b))
* report package-level failures (timeouts, signals) in go test summary ([c85a387](https://github.com/rtco-ai/rtco/commit/c85a387363e2079234b6141aad26418172c0e61a)), closes [#958](https://github.com/rtco-ai/rtco/issues/958)
* **security:** correct email domain from .dev to .app ([47383e8](https://github.com/rtco-ai/rtco/commit/47383e80197fc56e38f880f33a6b54261b82523c))
* **tee:** prevent panic on UTF-8 multi-byte truncation boundary ([da486bf](https://github.com/rtco-ai/rtco/commit/da486bf394330c804cd1cd12e4b6835f18de5205))
* **telemetry:** 7 bugs in enrichment — privacy leak, broken meta_usage, pricing ([15f666d](https://github.com/rtco-ai/rtco/commit/15f666dd8dbd18648cb7bd14a6f9f3cac2f7d10b))
* **telemetry:** clean code ([8156081](https://github.com/rtco-ai/rtco/commit/81560812610686fa5ca3633c2bf0b79c05eaa7d9))
* **telemetry:** consent, erasure, auth, docs ([2e4cc4b](https://github.com/rtco-ai/rtco/commit/2e4cc4bb5226444c8c0bfc827baf0c101c3759e8))
* **telemetry:** non-terminal consent, single config load ([7821e98](https://github.com/rtco-ai/rtco/commit/7821e9872fd1f1ae9b40eb8a4458049869acc36b))
* **telemetry:** RGPD-compliant, consent gate, erasure, privacy controls ([6a5bc84](https://github.com/rtco-ai/rtco/commit/6a5bc847e06cf6066e6f4aeed5a3ad0803a3649b))

## [0.35.0](https://github.com/rtco-ai/rtco/compare/v0.34.3...v0.35.0) (2026-04-06)


### Features

* **aws:** expand CLI filters from 8 to 25 subcommands ([402c48e](https://github.com/rtco-ai/rtco/commit/402c48e66988e638a5b4f4dd193238fc1d0fe18f))


### Bug Fixes

* **cmd:** read/cat multiple file and consistent behavior ([3f58018](https://github.com/rtco-ai/rtco/commit/3f58018f4af1d7206457929cf80bb4534203c3ee))
* **docs:** clean some docs + disclaimer ([deda44f](https://github.com/rtco-ai/rtco/commit/deda44f73607981f3d27ecc6341ce927aab34d37))
* **gh:** pass through gh pr merge instead of canned response ([#938](https://github.com/rtco-ai/rtco/issues/938)) ([8465ca9](https://github.com/rtco-ai/rtco/commit/8465ca953fa9d70dcc971a941c19465d456eb7d4))
* **gh:** pass through gh pr merge instead of canned response ([#938](https://github.com/rtco-ai/rtco/issues/938)) ([e1f2845](https://github.com/rtco-ai/rtco/commit/e1f2845df06a8d8b8325945dc4940ec5f530e4cc))
* **git:** inherit stdin for commit and push to preserve SSH signing ([#733](https://github.com/rtco-ai/rtco/issues/733)) ([eefeae4](https://github.com/rtco-ai/rtco/commit/eefeae45656ff2607c3f519c8eae235e3f0fe411))
* **git:** inherit stdin for commit and push to preserve SSH signing ([#733](https://github.com/rtco-ai/rtco/issues/733)) ([6cee6c6](https://github.com/rtco-ai/rtco/commit/6cee6c60b80f914ed9505e3925d85cadec43ab97))
* **git:** preserve full diff hunk headers ([62f4452](https://github.com/rtco-ai/rtco/commit/62f445227679f3df293fe35e9b18cc5ab39d7963))
* **git:** preserve full diff hunk headers ([09b3ff9](https://github.com/rtco-ai/rtco/commit/09b3ff9424e055f5fe25e535e5b60e077f8344f9))
* **go:** avoid false build errors from download logs ([9c1cf2f](https://github.com/rtco-ai/rtco/commit/9c1cf2f403534fa7874638b1b983c2d7f918a185))
* **go:** avoid false build errors from download logs ([d44fd3e](https://github.com/rtco-ai/rtco/commit/d44fd3e034208e3bcd59c2c46f7720eec4f10c98))
* **go:** cover more build failure shapes ([2425ad6](https://github.com/rtco-ai/rtco/commit/2425ad68e5386d19e5ec9ff1ca151a6d2c9a56d3))
* **go:** preserve failing test location context ([1481bc5](https://github.com/rtco-ai/rtco/commit/1481bc590924031456a6022510275c29c09e330e))
* **go:** preserve failing test location context ([374fe64](https://github.com/rtco-ai/rtco/commit/374fe64cfbedcd676733973e81a63a6dfecbb1b7))
* **go:** restore build error coverage ([1177c9c](https://github.com/rtco-ai/rtco/commit/1177c9c873ac63b6c0bcc9e1b664a705baa0ad7a))
* **grep:** close subprocess stdin to prevent memory leak ([#897](https://github.com/rtco-ai/rtco/issues/897)) ([7217562](https://github.com/rtco-ai/rtco/commit/72175623551f40b581b4a7f6ed966c1e4a9c7358))
* **grep:** close subprocess stdin to prevent memory leak ([#897](https://github.com/rtco-ai/rtco/issues/897)) ([09979cf](https://github.com/rtco-ai/rtco/commit/09979cf29701a1b775bcac761d24ec0e055d1bec))
* **hook_check:** detect missing integrations ([9cf9ccc](https://github.com/rtco-ai/rtco/commit/9cf9ccc1ac39f8bba37e932c7d318a3aa7a34ae9))
* **init:** remove opt-out instruction from telemetry message ([7571c8e](https://github.com/rtco-ai/rtco/commit/7571c8e101c41ee64c51e2bd64697f85f9142423))
* **init:** remove telemetry info lines from init output ([7dbef2c](https://github.com/rtco-ai/rtco/commit/7dbef2ce00824d26f2057e4c3c76e429e2e23088))
* **main:** kill zombie processes + path for rtco md ([d16fc6d](https://github.com/rtco-ai/rtco/commit/d16fc6dacbfec912c21522939b15b7bbd9719487))
* **main:** kill zombie processes + path for rtco md + missing intergrations ([a919335](https://github.com/rtco-ai/rtco/commit/a919335519ed4a5259a212e56407cb312aa99bac))
* **merge:** changelog conflicts ([d92c5d2](https://github.com/rtco-ai/rtco/commit/d92c5d264a49483c8d6079e04d946a79bc990a74))
* **proxy:** kill child process on SIGINT/SIGTERM to prevent orphans ([d813919](https://github.com/rtco-ai/rtco/commit/d813919a24546e044e7844fc7ed05fef4ec24033))
* **proxy:** kill child process on SIGINT/SIGTERM to prevent orphans ([3318510](https://github.com/rtco-ai/rtco/commit/33185101fc122d0c11a25a4e02ac9f3a7dc7e3bb))
* **review:** address ChildGuard disarm, stdin dedup, hook masking ([d85fe33](https://github.com/rtco-ai/rtco/commit/d85fe3384b87c16fafd25ec7bcadbff6e69f3f1f))
* **security:** default to ask when no permission rule matches ([#886](https://github.com/rtco-ai/rtco/issues/886)) ([158c745](https://github.com/rtco-ai/rtco/commit/158c74527f6591d372e40a78cd604d73a20649a9))
* **security:** default to ask when no permission rule matches ([#886](https://github.com/rtco-ai/rtco/issues/886)) ([41a6c6b](https://github.com/rtco-ai/rtco/commit/41a6c6bf6da78a4754794fdc6a1469df2e327920))
* **tracking:** use std::env::temp_dir() for compatibility (instead of unix tmp) ([e918661](https://github.com/rtco-ai/rtco/commit/e918661440d7b50321f0535032f52c5e87aaf3cb))

## [0.34.3](https://github.com/rtco-ai/rtco/compare/v0.34.2...v0.34.3) (2026-04-02)


### Bug Fixes

* **automod:** add auto discovery for cmds ([234909d](https://github.com/rtco-ai/rtco/commit/234909d2c754ade2fdc939b0a1435a8e34ffc305))
* **ci:** fix validate-docs.sh broken module count check ([bbe3da6](https://github.com/rtco-ai/rtco/commit/bbe3da642b5fc4b065b13a65647ea0ebf5264e65))
* **cleaning:** constant extract ([aabc016](https://github.com/rtco-ai/rtco/commit/aabc0167bc013fd2d0c61a687580f6e69305500a))
* **cmds:** migrate remaining exit_code to exit_code_from_output ([ba9fa34](https://github.com/rtco-ai/rtco/commit/ba9fa345f3d1d14bd0af236ec9aa8a9a0e5581d6))
* **cmds:** more covering for run_filtered ([e48485a](https://github.com/rtco-ai/rtco/commit/e48485adc6a33d12b70664598020595cf7dfcd7e))
* **docs:** add documentation ([2f7278a](https://github.com/rtco-ai/rtco/commit/2f7278ac5992bf2e84b763fb05642d89900ba495))
* **docs:** add maintainers docs ([14265b4](https://github.com/rtco-ai/rtco/commit/14265b48c3a15e459a31da11250a51ab5830a508))
* **refacto-p1:** unified cmds execution flow  (+ rm dead code) ([75bd607](https://github.com/rtco-ai/rtco/commit/75bd607d55235f313855f5fe8c9eceafd73700a7))
* **refacto-p2:** more standardize ([47a76ea](https://github.com/rtco-ai/rtco/commit/47a76ea35ed2fe02a3600792163f727fa3a94ff2))
* **refacto-p2:** more standardize ([92c671a](https://github.com/rtco-ai/rtco/commit/92c671a175a5e2bf09720fd1a8591140bcb473a0))
* **refacto:** wrappers for standardization, exit codes lexer tokenizer, constants, code clean ([bff0258](https://github.com/rtco-ai/rtco/commit/bff02584243f1b73418418b0c05365acf56fbb36))
* **registry:** quoted env prefix + inline regex cleanup + routing docs ([f3217a4](https://github.com/rtco-ai/rtco/commit/f3217a467b543a3181605b257162f2b3ab5d5df0))
* **review:** address PR [#910](https://github.com/rtco-ai/rtco/issues/910) review feedback ([0a8b8fd](https://github.com/rtco-ai/rtco/commit/0a8b8fd0693fa504f376146cbbcafe9ddf4632c8))
* **review:** PR [#934](https://github.com/rtco-ai/rtco/issues/934) ([5bd35a3](https://github.com/rtco-ai/rtco/commit/5bd35a33ad6abe5278749726bed19912664531c2))
* **review:** PR [#934](https://github.com/rtco-ai/rtco/issues/934) ([bae7930](https://github.com/rtco-ai/rtco/commit/bae79301194bbb48d1cbb39554096c3225f7cb73))
* **rules:** add wc RtkRule with pattern field for develop compat ([d75e864](https://github.com/rtco-ai/rtco/commit/d75e864f20451a5e17918c75f2ea32672f65e1f4))
* **standardize:** git+kube sub wrappers run_filtered ([7fd221f](https://github.com/rtco-ai/rtco/commit/7fd221f44660bcf411aa333d2c35a49ff89e7961))
* **standardize:** merge pattern into rues ([08aabb9](https://github.com/rtco-ai/rtco/commit/08aabb95c3ae6e0b734f696264e1e1a8c0f0b22e))

## [0.34.2](https://github.com/rtco-ai/rtco/compare/v0.34.1...v0.34.2) (2026-03-30)


### Bug Fixes

* **emots:** replace 📊 with "Summary:" ([495a152](https://github.com/rtco-ai/rtco/commit/495a152059feabc7b516b96e804757608b87a10a))
* **refacto-codebase:** technical docs & sub folders ([927daef](https://github.com/rtco-ai/rtco/commit/927daef49b8f771d195201d196378e27e0ee8a2b))

## [0.34.1](https://github.com/rtco-ai/rtco/compare/v0.34.0...v0.34.1) (2026-03-28)


### Bug Fixes

* **security:** missing toml pkg ([51f9c88](https://github.com/rtco-ai/rtco/commit/51f9c888b81169309df92f7fa3a6f705d44adcab))
* **security:** salt device hash for telemetry ([32fdbbb](https://github.com/rtco-ai/rtco/commit/32fdbbbb6923c70d343fab14b4b0ce70424e610f))
* **security:** set 0600 permissions on salt file ([5eae11d](https://github.com/rtco-ai/rtco/commit/5eae11d16410dc4ff26e97672e5367b14efaab76))
* **telemetry:** cache salt in-process ([22dc059](https://github.com/rtco-ai/rtco/commit/22dc059310b0408adedc2d1228de339e16ea6c0a))
* **telemetry:** docs + real info from "rtco init -g" ([33195cc](https://github.com/rtco-ai/rtco/commit/33195cc686318ddcca54edfdd1215bd9fd28f891))
* **telemetry:** hash + salt ([92996b1](https://github.com/rtco-ai/rtco/commit/92996b127257eae16d3e17179592b2899f19254f))

## [0.34.0](https://github.com/rtco-ai/rtco/compare/v0.33.1...v0.34.0) (2026-03-26)


### Features

* **init:** add --copilot flag for GitHub Copilot integration ([9e19aac](https://github.com/rtco-ai/rtco/commit/9e19aac75e790ecbfd1dc5b2d01786f6b9edf506)), closes [#823](https://github.com/rtco-ai/rtco/issues/823)


### Bug Fixes

* **diff:** correct truncation overflow count in condense_unified_diff ([5399f83](https://github.com/rtco-ai/rtco/commit/5399f836a5c642121f0f6e7812ff4131d84d0509))
* **diff:** never truncate diff content — show all changes in full ([80fc29a](https://github.com/rtco-ai/rtco/commit/80fc29a839f51ef605474037e1a8fd86b4aac05a)), closes [#827](https://github.com/rtco-ai/rtco/issues/827)
* **git:** replace vague truncation markers with exact counts ([185fb97](https://github.com/rtco-ai/rtco/commit/185fb97061517922ea5844d8c6008f2eb86fd55d))
* **merge:** resolve conflict with develop in diff_cmd.rs ([6a5ae14](https://github.com/rtco-ai/rtco/commit/6a5ae1484b32c38bd99baca925175ae610e3d1e3))
* **read:** default to no filtering — show full file content ([5e0f3ba](https://github.com/rtco-ai/rtco/commit/5e0f3ba774eab52f8ca2ac603e2ae4eae79b2edc)), closes [#822](https://github.com/rtco-ai/rtco/issues/822)
* **read:** detect binary files and prevent empty output on filter failure ([8886c14](https://github.com/rtco-ai/rtco/commit/8886c14c9cf97fb4413efec3be8e50fdb84824e9)), closes [#822](https://github.com/rtco-ai/rtco/issues/822)
* rewrite swift test commands ([599ad25](https://github.com/rtco-ai/rtco/commit/599ad25deb0f8dc9ecab37f4bbe26324dac07b2e))
* truncation accuracy + Copilot init + binary file detection ([966bcbe](https://github.com/rtco-ai/rtco/commit/966bcbe638be18bbaba4298df985804643f82c85))
* **truncation:** accurate overflow counts and omission indicators ([58a9633](https://github.com/rtco-ai/rtco/commit/58a963347467613d48db05ad56bc8f1f3a06b65d))

## [0.33.1](https://github.com/rtco-ai/rtco/compare/v0.33.0...v0.33.1) (2026-03-25)


### Bug Fixes

* **cicd:** dev- prefix for pre-release tags ([522bd64](https://github.com/rtco-ai/rtco/commit/522bd648c8cae41f6cadedcd40a96d879c6ecf0a))
* **cicd:** use dev- prefix for pre-release tags ([9c21275](https://github.com/rtco-ai/rtco/commit/9c212752fc0401820f8665198f00882684496175))
* **cicd:** use dev- prefix for pre-release tags to avoid polluting release-please ([32c67e0](https://github.com/rtco-ai/rtco/commit/32c67e01326374f0365602f61542a3639a8f121b))
* hook security + stderr redirects + version bump ([#807](https://github.com/rtco-ai/rtco/issues/807)) ([0649e97](https://github.com/rtco-ai/rtco/commit/0649e974fb8f27778ef0d22aa97905d9ebc8f03c))
* **hook:** respect Claude Code deny/ask permission rules on rewrite ([a051a6f](https://github.com/rtco-ai/rtco/commit/a051a6f5e56c7ee59375a365580bced634e29c02))
* strip trailing stderr redirects before rewrite matching ([#530](https://github.com/rtco-ai/rtco/issues/530)) ([edd9c02](https://github.com/rtco-ai/rtco/commit/edd9c02e892b297a7e349031b61ef971c982b53d))
* strip trailing stderr redirects before rewrite matching ([#530](https://github.com/rtco-ai/rtco/issues/530)) ([36a6f48](https://github.com/rtco-ai/rtco/commit/36a6f482296d6fc85f8116040a16de2e128733f8))

## [0.33.0-rc.54](https://github.com/rtco-ai/rtco/compare/v0.32.0-rc.54...v0.33.0-rc.54) (2026-03-24)


### Features

* **ruby:** add Ruby on Rails support (rspec, rubocop, rake, bundle) ([#724](https://github.com/rtco-ai/rtco/issues/724)) ([15bc0f8](https://github.com/rtco-ai/rtco/commit/15bc0f8d6e135371688d5fd42decc6d8a99454f0))


### Bug Fixes

* add telemetry documentation and init notice ([#640](https://github.com/rtco-ai/rtco/issues/640)) ([#788](https://github.com/rtco-ai/rtco/issues/788)) ([0eecee5](https://github.com/rtco-ai/rtco/commit/0eecee5bf35ffd8b13f36a59ec39bd52626948d3))
* **cargo:** preserve test compile diagnostics ([97b6878](https://github.com/rtco-ai/rtco/commit/97b68783f50d209c2c599ae42cc638520749e668))
* **cicd:** explicit fetch tag ([3b94b60](https://github.com/rtco-ai/rtco/commit/3b94b602ed24b9ecec597ce001e59f325caaadd4))
* **cicd:** gete release like tag for pre-release ([53bc81e](https://github.com/rtco-ai/rtco/commit/53bc81e9e6d3d0876fb1a23dbf6f08bc074b68be))
* **cicd:** issue 668 - pre release tag ([200af43](https://github.com/rtco-ai/rtco/commit/200af436d48dd2539cb00652b082f25c57873c9c))
* **cicd:** missing doc ([8657494](https://github.com/rtco-ai/rtco/commit/865749438e67f6da7f719d054bf377d857925ad3))
* **cicd:** pre-release correct tag ([1536667](https://github.com/rtco-ai/rtco/commit/15366678adeece701f38e91204128b070c0e3fc4))
* **dotnet:** TRX injection for Microsoft.Testing.Platform projects ([8eefef1](https://github.com/rtco-ai/rtco/commit/8eefef1b496035ce898effc5446e6851084d6fa4))
* **formatter:** show full error message for test failures ([#690](https://github.com/rtco-ai/rtco/issues/690)) ([dc6b026](https://github.com/rtco-ai/rtco/commit/dc6b0260ab4c1bdbccb4b775d879eb473b212c21))
* **formatter:** show full error message for test failures ([#690](https://github.com/rtco-ai/rtco/issues/690)) ([f7b09fc](https://github.com/rtco-ai/rtco/commit/f7b09fc86a693acf2b52954215ff0c4e6c5d03f9))
* **gh:** passthrough --comments flag in issue/pr view ([75cd223](https://github.com/rtco-ai/rtco/commit/75cd2232e274f898d8a335ba866fc507ce64b949))
* **gh:** passthrough --comments flag in issue/pr view ([fdeb09f](https://github.com/rtco-ai/rtco/commit/fdeb09fb93564e795711e9a531d2e2e20187c3a7)), closes [#720](https://github.com/rtco-ai/rtco/issues/720)
* **gh:** skip compact_diff for --name-only/--stat flags in pr diff ([2ef0690](https://github.com/rtco-ai/rtco/commit/2ef0690767eb733c705e4de56d02c64696a4acc6)), closes [#730](https://github.com/rtco-ai/rtco/issues/730)
* **gh:** skip compact_diff for --name-only/--stat in pr diff ([c576249](https://github.com/rtco-ai/rtco/commit/c57624931a96181f869645817fdd96bc056da044))
* **golangci-lint:** add v2 compatibility with runtime version detection ([95a4961](https://github.com/rtco-ai/rtco/commit/95a4961e4aa3ba5307b3dfad246c6168c4caeab8))
* **golangci:** use resolved_command for version detection, move test fixture to file ([6aa5e90](https://github.com/rtco-ai/rtco/commit/6aa5e90dc466f87c88a2401b4eb2aa0f323379f4))
* increase signal in git diff, git log, and json filters ([#621](https://github.com/rtco-ai/rtco/issues/621)) ([#708](https://github.com/rtco-ai/rtco/issues/708)) ([4edc3fc](https://github.com/rtco-ai/rtco/commit/4edc3fc0838e25ee6d1754c7e987b5507742f600))
* **playwright:** add tee_and_hint pass-through on failure ([#690](https://github.com/rtco-ai/rtco/issues/690)) ([b4ccf04](https://github.com/rtco-ai/rtco/commit/b4ccf046f59ce6ed1396e4d8c46f8a35152d6d09))
* preserve cargo test compile diagnostics ([15d5beb](https://github.com/rtco-ai/rtco/commit/15d5beb9f70caf1f84e9b506faaf840c70c1cf4e))
* **ruby:** use rails test for positional file args in rtco rake ([ec92c43](https://github.com/rtco-ai/rtco/commit/ec92c43f231eb2321a4b423b0eb8487f98161aac))
* **ruby:** use rails test for positional file args in rtco rake ([138e914](https://github.com/rtco-ai/rtco/commit/138e91411b4802e445a97429056cca73282d09e1))
* update Discord invite link ([#711](https://github.com/rtco-ai/rtco/issues/711)) ([#786](https://github.com/rtco-ai/rtco/issues/786)) ([af56573](https://github.com/rtco-ai/rtco/commit/af56573ae2b234123e4685fd945980e644f40fa3))

## [0.31.0](https://github.com/rtco-ai/rtco/compare/v0.30.1...v0.31.0) (2026-03-19)


### Features

* 9-tool AI agent support + emoji removal ([#704](https://github.com/rtco-ai/rtco/issues/704)) ([737dada](https://github.com/rtco-ai/rtco/commit/737dada4a56c0d7a482cc438e7280340d634f75d))

## [0.30.1](https://github.com/rtco-ai/rtco/compare/v0.30.0...v0.30.1) (2026-03-18)


### Bug Fixes

* remove all decorative emojis from CLI output ([#687](https://github.com/rtco-ai/rtco/issues/687)) ([#686](https://github.com/rtco-ai/rtco/issues/686)) ([4792008](https://github.com/rtco-ai/rtco/commit/4792008fc15553cbb9aeaa602f773a5f8f7f7afe))

## [0.30.0](https://github.com/rtco-ai/rtco/compare/v0.29.0...v0.30.0) (2026-03-16)


### Features

* add rtco session command for adoption overview ([be67d66](https://github.com/rtco-ai/rtco/commit/be67d660100c06a0751c08d943dc884ad5bff6a3))
* add rtco session command for adoption overview ([12d44c4](https://github.com/rtco-ai/rtco/commit/12d44c4068d7d4f65d5bd7551af29ab5a2352ed1)), closes [#487](https://github.com/rtco-ai/rtco/issues/487)
* add worktree slash commands for isolated development ([#364](https://github.com/rtco-ai/rtco/issues/364)) ([ab83e79](https://github.com/rtco-ai/rtco/commit/ab83e7933ebc26ca76f843d33285729875efb913))
* Claude Code tooling — 2 agents, 7 commands, 2 rules, 4 skills ([#491](https://github.com/rtco-ai/rtco/issues/491)) ([7b7a5ae](https://github.com/rtco-ai/rtco/commit/7b7a5ae4b6d23fbb882ed7d5e815e2ed0672c46c))


### Bug Fixes

* 6 critical bugs — exit codes, unwrap, lazy regex ([#626](https://github.com/rtco-ai/rtco/issues/626)) ([3005ebd](https://github.com/rtco-ai/rtco/commit/3005ebd0ad07912ae919687f6d3d49482aabaeac))
* align 7 TOML filter tests with on_empty behavior ([04ed6d8](https://github.com/rtco-ai/rtco/commit/04ed6d8c314dcbf86b147903b5a7f1cd956dc980))
* align 7 TOML filter tests with on_empty behavior ([9a499b9](https://github.com/rtco-ai/rtco/commit/9a499b9714e97a553d5603680ab1f843034acf28))
* **cicd-docs:** add agent reviewer + some contribute guidelines ([de710f4](https://github.com/rtco-ai/rtco/commit/de710f4ea30c333130c46f8a2e2c5b6b9edd4889))
* **cicd-docs:** some logs to understand what is happening when check docs ([191ea9a](https://github.com/rtco-ai/rtco/commit/191ea9af9f99ee78d74385fe1952ce83045e4afe))
* **cicd:** Clean cicd, rework depends and add pre-release ([d24a765](https://github.com/rtco-ai/rtco/commit/d24a7650e26aca89224a3ec5d263f1ce7c7121d6))
* **cicd:** Clean cicd, rework depends and add pre-release ([6303e95](https://github.com/rtco-ai/rtco/commit/6303e9530a379a8e3939e6c122ab4cf07cb16751))
* **cicd:** clippy - do not treat warn as error ([5da5db2](https://github.com/rtco-ai/rtco/commit/5da5db222d9927394995ccaeb3afc103e80c22bd))
* failing context for doc analyze -&gt; cat from files ([c6b7db2](https://github.com/rtco-ai/rtco/commit/c6b7db2e5a6cd9a05262e934b4fc7a44c699c3b0))
* git log --oneline regression drops commits ([#619](https://github.com/rtco-ai/rtco/issues/619)) ([8e85d67](https://github.com/rtco-ai/rtco/commit/8e85d676d78b12d2c421bb892f93971fc222fb39))
* improve adoption metric by detecting hook-rewritten commands ([eb8a2c4](https://github.com/rtco-ai/rtco/commit/eb8a2c4a71072870fca4b64e90189a4453acff84))
* normalize binlogs CRLF ([5344af9](https://github.com/rtco-ai/rtco/commit/5344af9a51f06b5dc42692e42c948ff11a3173c6))
* preserve commit body in git log output ([e189bbb](https://github.com/rtco-ai/rtco/commit/e189bbbe749120eda4d98a2130937269d8c0e92a))
* preserve first line of commit body in git log output ([c3416eb](https://github.com/rtco-ai/rtco/commit/c3416eb45f2f97297ec149d296a6a500697d302b))
* remove version check from validate-docs CI ([#476](https://github.com/rtco-ai/rtco/issues/476)) ([#543](https://github.com/rtco-ai/rtco/issues/543)) ([6e61c24](https://github.com/rtco-ai/rtco/commit/6e61c2447cc03af94220ce6ce83686f155e18086))
* split chained commands in adoption metric ([127f85c](https://github.com/rtco-ai/rtco/commit/127f85c02efd52a64e461005fa142d05f81615f8))
* support git -C &lt;path&gt; in rewrite registry ([c916bab](https://github.com/rtco-ai/rtco/commit/c916bab33ae9760b234fd720c944a849141f0d2e)), closes [#555](https://github.com/rtco-ai/rtco/issues/555)
* test-all.sh aborts when gt not installed ([#500](https://github.com/rtco-ai/rtco/issues/500)) ([#544](https://github.com/rtco-ai/rtco/issues/544)) ([26f5473](https://github.com/rtco-ai/rtco/commit/26f547371798ad32aed3569965303bc4857789ed))
* trust boundary followup — TOML key typo + missing meta commands ([#625](https://github.com/rtco-ai/rtco/issues/625)) ([8d8e188](https://github.com/rtco-ai/rtco/commit/8d8e188705e5784829693a83b2076d6118154764))
* windows path fix for git tests ([0a904e2](https://github.com/rtco-ai/rtco/commit/0a904e264d58f8f4b5f10e37ec3b11f717458fe0))

## [0.29.0](https://github.com/rtco-ai/rtco/compare/v0.28.2...v0.29.0) (2026-03-12)


### Features

* rewrite engine, OpenCode support, hook system improvements ([#539](https://github.com/rtco-ai/rtco/issues/539)) ([c1de10d](https://github.com/rtco-ai/rtco/commit/c1de10d94c0a35f825b71713e2db4624310c03d1))

## [0.28.2](https://github.com/rtco-ai/rtco/compare/v0.28.1...v0.28.2) (2026-03-10)


### Bug Fixes

* add tokens_saved to telemetry payload ([#471](https://github.com/rtco-ai/rtco/issues/471)) ([#472](https://github.com/rtco-ai/rtco/issues/472)) ([f8b7d52](https://github.com/rtco-ai/rtco/commit/f8b7d52d2d25d09a44f391576bad6a7b271f1f8c))

## [0.28.1](https://github.com/rtco-ai/rtco/compare/v0.28.0...v0.28.1) (2026-03-10)


### Bug Fixes

* 4 critical bugs + telemetry enrichment ([#462](https://github.com/rtco-ai/rtco/issues/462)) ([7d76af8](https://github.com/rtco-ai/rtco/commit/7d76af84b95e0f040e8b91a154edb89f80e5c380))
* restore lost telemetry install_method enrichment ([#469](https://github.com/rtco-ai/rtco/issues/469)) ([0c5cde9](https://github.com/rtco-ai/rtco/commit/0c5cde9ec234a2b7b0376adbcb78f2be48a98e86))

## [0.28.0](https://github.com/rtco-ai/rtco/compare/v0.27.2...v0.28.0) (2026-03-10)


### Features

* **gt:** add Graphite CLI support ([#290](https://github.com/rtco-ai/rtco/issues/290)) ([7fbc4ef](https://github.com/rtco-ai/rtco/commit/7fbc4ef4b553d5e61feeb6e73d8f6a96b6df3dd9))
* TOML Part 1 — filter DSL engine + 14 built-in filters ([#349](https://github.com/rtco-ai/rtco/issues/349)) ([adda253](https://github.com/rtco-ai/rtco/commit/adda2537be1fe69625ac280f15e8c8067d08c711))
* TOML Part 2 — user-global config, shadow warning, rtco init templates, 4 new built-in filters ([#351](https://github.com/rtco-ai/rtco/issues/351)) ([926e6a0](https://github.com/rtco-ai/rtco/commit/926e6a0dd4512c4cbb0f5ac133e60cb6134a3174))
* TOML Part 3 — 15 additional built-in filters (ping, rsync, dotnet, swift, shellcheck, hadolint, poetry, composer, brew, df, ps, systemctl, yamllint, markdownlint, uv) ([#386](https://github.com/rtco-ai/rtco/issues/386)) ([b71a8d2](https://github.com/rtco-ai/rtco/commit/b71a8d24e2dbd3ff9bb423c849638bfa23830c0b))

## [0.27.2](https://github.com/rtco-ai/rtco/compare/v0.27.1...v0.27.2) (2026-03-06)


### Bug Fixes

* gh pr edit/comment pass correct subcommand to gh ([#332](https://github.com/rtco-ai/rtco/issues/332)) ([799f085](https://github.com/rtco-ai/rtco/commit/799f0856e4547318230fe150a43f50ab82e1cf03))
* pass through -R/--repo flag in gh view commands ([#328](https://github.com/rtco-ai/rtco/issues/328)) ([0a1bcb0](https://github.com/rtco-ai/rtco/commit/0a1bcb05e5737311211369dcb92b3f756a6230c6)), closes [#223](https://github.com/rtco-ai/rtco/issues/223)
* reduce gh diff / git diff / gh api truncation ([#354](https://github.com/rtco-ai/rtco/issues/354)) ([#370](https://github.com/rtco-ai/rtco/issues/370)) ([e356c12](https://github.com/rtco-ai/rtco/commit/e356c1280da9896195d0dff91e152c5f20347a65))
* strip npx/bunx/pnpm prefixes in lint linter detection ([#186](https://github.com/rtco-ai/rtco/issues/186)) ([#366](https://github.com/rtco-ai/rtco/issues/366)) ([27b35d8](https://github.com/rtco-ai/rtco/commit/27b35d84a341622aa4bf686c2ce8867f8feeb742))

## [0.27.1](https://github.com/rtco-ai/rtco/compare/v0.27.0...v0.27.1) (2026-03-06)


### Bug Fixes

* only rewrite docker compose ps/logs/build, skip unsupported subcommands ([#336](https://github.com/rtco-ai/rtco/issues/336)) ([#363](https://github.com/rtco-ai/rtco/issues/363)) ([dbc9503](https://github.com/rtco-ai/rtco/commit/dbc950395e31b4b0bc48710dc52ad01d4d73f9ba))
* preserve -- separator for cargo commands and silence fallback ([#326](https://github.com/rtco-ai/rtco/issues/326)) ([45f9344](https://github.com/rtco-ai/rtco/commit/45f9344f033d27bc370ff54c4fc0c61e52446076)), closes [#286](https://github.com/rtco-ai/rtco/issues/286) [#287](https://github.com/rtco-ai/rtco/issues/287)
* prettier false positive when not installed ([#221](https://github.com/rtco-ai/rtco/issues/221)) ([#359](https://github.com/rtco-ai/rtco/issues/359)) ([85b0b3e](https://github.com/rtco-ai/rtco/commit/85b0b3eb0bad9cbacdc32d2e9ba525728acd7cbe))
* support git commit -am, --amend and other flags ([#327](https://github.com/rtco-ai/rtco/issues/327)) ([#360](https://github.com/rtco-ai/rtco/issues/360)) ([409aed6](https://github.com/rtco-ai/rtco/commit/409aed6dbcdd7cac2a48ec5655e6f1fd8d5248e3))

## [0.27.0](https://github.com/rtco-ai/rtco/compare/v0.26.0...v0.27.0) (2026-03-05)


### Features

* warn when installed hook is outdated ([#344](https://github.com/rtco-ai/rtco/issues/344)) ([#350](https://github.com/rtco-ai/rtco/issues/350)) ([3141fec](https://github.com/rtco-ai/rtco/commit/3141fecf958af5ae98c232543b913f3ca388254f))


### Bug Fixes

* bugs [#196](https://github.com/rtco-ai/rtco/issues/196) [#344](https://github.com/rtco-ai/rtco/issues/344) [#345](https://github.com/rtco-ai/rtco/issues/345) [#346](https://github.com/rtco-ai/rtco/issues/346) [#347](https://github.com/rtco-ai/rtco/issues/347) — gh --json, hook check, RTK_DISABLED, 2&gt;&1, json TOML ([8953af0](https://github.com/rtco-ai/rtco/commit/8953af0fc06759b37f16743ef383af0a52af2bed))
* RTK_DISABLED ignored, 2&gt;&1 broken, json TOML error ([#345](https://github.com/rtco-ai/rtco/issues/345), [#346](https://github.com/rtco-ai/rtco/issues/346), [#347](https://github.com/rtco-ai/rtco/issues/347)) ([6c13d23](https://github.com/rtco-ai/rtco/commit/6c13d234364d314f53b6698c282a621019635fd6))
* skip rewrite for gh --json/--jq/--template ([#196](https://github.com/rtco-ai/rtco/issues/196)) ([079ee9a](https://github.com/rtco-ai/rtco/commit/079ee9a4ea868ecf4e7beffcbc681ca1ba8b165c))

## [0.26.0](https://github.com/rtco-ai/rtco/compare/v0.25.0...v0.26.0) (2026-03-05)


### Features

* add Claude Code skills for PR and issue triage ([#343](https://github.com/rtco-ai/rtco/issues/343)) ([6ad6ffe](https://github.com/rtco-ai/rtco/commit/6ad6ffeccee9b622013f8e1357b6ca4c94aacb59))
* anonymous telemetry ping (1/day, opt-out) ([#334](https://github.com/rtco-ai/rtco/issues/334)) ([baff6a2](https://github.com/rtco-ai/rtco/commit/baff6a2334b155c0d68f38dba85bd8d6fe9e20af))


### Bug Fixes

* curl JSON size guard ([#297](https://github.com/rtco-ai/rtco/issues/297)) + exclude_commands config ([#243](https://github.com/rtco-ai/rtco/issues/243)) ([#342](https://github.com/rtco-ai/rtco/issues/342)) ([a8d6106](https://github.com/rtco-ai/rtco/commit/a8d6106f736e049013ecb77f0f413167266dd40e))

## [0.25.0](https://github.com/rtco-ai/rtco/compare/v0.24.0...v0.25.0) (2026-03-05)


### Features

* `rtco rewrite` — single source of truth for LLM hook rewrites ([#241](https://github.com/rtco-ai/rtco/issues/241)) ([f447a3d](https://github.com/rtco-ai/rtco/commit/f447a3d5b136dd5b1df3d5cc4969e29a68ba3f89))


### Bug Fixes

* **find:** accept native find flags (-name, -type, etc.) ([#211](https://github.com/rtco-ai/rtco/issues/211)) ([7ac5bc4](https://github.com/rtco-ai/rtco/commit/7ac5bc4bd3942841cc1abb53399025b4fcae10c9))

## [0.24.0](https://github.com/rtco-ai/rtco/compare/v0.23.0...v0.24.0) (2026-03-04)


### Features

* add AWS CLI and psql modules with token-optimized output ([#216](https://github.com/rtco-ai/rtco/issues/216)) ([b934466](https://github.com/rtco-ai/rtco/commit/b934466364c131de2656eefabe933965f8424e18))
* passthrough fallback when Clap parse fails + review fixes ([#200](https://github.com/rtco-ai/rtco/issues/200)) ([772b501](https://github.com/rtco-ai/rtco/commit/772b5012ede833c3f156816f212d469560449a30))
* **security:** add SHA-256 hook integrity verification ([f2caca3](https://github.com/rtco-ai/rtco/commit/f2caca3abc330fb45a466af6a837ed79c3b00b40))


### Bug Fixes

* **git:** propagate exit codes in push/pull/fetch/stash/worktree ([#234](https://github.com/rtco-ai/rtco/issues/234)) ([5cfaecc](https://github.com/rtco-ai/rtco/commit/5cfaeccaba2fc6e1fe5284f57b7af7ec7c0a224d))
* **playwright:** fix JSON parser to match real Playwright output format ([#193](https://github.com/rtco-ai/rtco/issues/193)) ([4eb6cf4](https://github.com/rtco-ai/rtco/commit/4eb6cf4b1a2333cb710970e40a96f1004d4ab0fa))
* support additional git global options (--no-pager, --no-optional-locks, --bare, --literal-pathspecs) ([68ca712](https://github.com/rtco-ai/rtco/commit/68ca7126d45609a41dbff95e2770d58a11ebc0a3))
* support git global options (-C, -c, --git-dir, --work-tree, --no-pager, --no-optional-locks, --bare, --literal-pathspecs) ([a6ccefe](https://github.com/rtco-ai/rtco/commit/a6ccefe8e71372b61e6e556f0d36a944d1bcbd70))
* support git global options (-C, -c, --git-dir, --work-tree) ([982084e](https://github.com/rtco-ai/rtco/commit/982084ee34c17d2fe89ff9f4839374bf0caa2d19))
* update version refs to 0.23.0, module count to 51, fmt upstream files ([eed0188](https://github.com/rtco-ai/rtco/commit/eed018814b141ada8140f350adc26d9f104cf368))

## [0.23.0](https://github.com/rtco-ai/rtco/compare/v0.22.2...v0.23.0) (2026-02-28)


### Features

* add mypy command with grouped error output ([#109](https://github.com/rtco-ai/rtco/issues/109)) ([e8ef341](https://github.com/rtco-ai/rtco/commit/e8ef3418537247043808dc3c88bfd189b717a0a1))
* **gain:** add per-project token savings with -p flag ([#128](https://github.com/rtco-ai/rtco/issues/128)) ([2b550ee](https://github.com/rtco-ai/rtco/commit/2b550eebd6219a4844488d8fde1842ba3c6dec25))


### Bug Fixes

* eliminate duplicate output when grep-ing function names from git show ([#248](https://github.com/rtco-ai/rtco/issues/248)) ([a6f65f1](https://github.com/rtco-ai/rtco/commit/a6f65f11da71936d148a2562216ab45b4c4b04a0))
* filter docker compose hook rewrites to supported subcommands ([#245](https://github.com/rtco-ai/rtco/issues/245)) ([dbbf980](https://github.com/rtco-ai/rtco/commit/dbbf980f3ba9a51d0f7eb703e7b3c52fde2b784f)), closes [#244](https://github.com/rtco-ai/rtco/issues/244)
* **registry:** "fi" in IGNORED_PREFIXES shadows find commands ([#246](https://github.com/rtco-ai/rtco/issues/246)) ([48965c8](https://github.com/rtco-ai/rtco/commit/48965c85d2dd274bbdcf27b11850ccd38909e6f4))
* remove personal preferences from project CLAUDE.md ([3a8044e](https://github.com/rtco-ai/rtco/commit/3a8044ef6991b2208d904b7401975fcfcb165cdb))
* remove personal preferences from project CLAUDE.md ([d362ad0](https://github.com/rtco-ai/rtco/commit/d362ad0e4968cfc6aa93f9ef163512a692ca5d1b))
* remove remaining personal project reference from CLAUDE.md ([5b59700](https://github.com/rtco-ai/rtco/commit/5b597002dcd99029cb9c0da9b6d38b44021bdb3a))
* remove remaining personal project reference from CLAUDE.md ([dc09265](https://github.com/rtco-ai/rtco/commit/dc092655fb84a7c19a477e731eed87df5ad0b89f))
* surface build failures in go test summary ([#274](https://github.com/rtco-ai/rtco/issues/274)) ([b405e48](https://github.com/rtco-ai/rtco/commit/b405e48ca6c4be3ba702a5d9092fa4da4dff51dc))

## [0.22.2](https://github.com/rtco-ai/rtco/compare/v0.22.1...v0.22.2) (2026-02-20)


### Bug Fixes

* **grep:** accept -n flag for grep/rg compatibility ([7d561cc](https://github.com/rtco-ai/rtco/commit/7d561cca51e4e177d353e6514a618e5bb09eebc6))
* **playwright:** fix JSON parser and binary resolution ([#215](https://github.com/rtco-ai/rtco/issues/215)) ([461856c](https://github.com/rtco-ai/rtco/commit/461856c8fd78cce8e2d875ae878111d7cb3610cd))
* propagate rg exit code in rtco grep for CLI parity ([#227](https://github.com/rtco-ai/rtco/issues/227)) ([f1be885](https://github.com/rtco-ai/rtco/commit/f1be88565e602d3b6777f629d417e957a62daae2)), closes [#162](https://github.com/rtco-ai/rtco/issues/162)

## [0.22.1](https://github.com/rtco-ai/rtco/compare/v0.22.0...v0.22.1) (2026-02-19)


### Bug Fixes

* git branch creation silently swallowed by list mode ([#194](https://github.com/rtco-ai/rtco/issues/194)) ([88dc752](https://github.com/rtco-ai/rtco/commit/88dc752220dc79dfa09b871065b28ae6ef907231))
* **git:** support multiple -m flags in git commit ([292225f](https://github.com/rtco-ai/rtco/commit/292225f2dd09bfc5274cc8b4ed92d1a519929629))
* **git:** support multiple -m flags in git commit ([c18553a](https://github.com/rtco-ai/rtco/commit/c18553a55c1192610525a5341a183da46c59d50c))
* **grep:** translate BRE \| alternation and strip -r flag for rg ([#206](https://github.com/rtco-ai/rtco/issues/206)) ([70d1b04](https://github.com/rtco-ai/rtco/commit/70d1b04093a3dfcc99991502f1530cbb13bae872))
* propagate linter exit code in rtco lint ([#207](https://github.com/rtco-ai/rtco/issues/207)) ([8e826fc](https://github.com/rtco-ai/rtco/commit/8e826fc89fe7350df82ee2b1bae8104da609f2b2)), closes [#185](https://github.com/rtco-ai/rtco/issues/185)
* smart markdown body filter for gh issue/pr view ([#188](https://github.com/rtco-ai/rtco/issues/188)) ([#214](https://github.com/rtco-ai/rtco/issues/214)) ([4208015](https://github.com/rtco-ai/rtco/commit/4208015cce757654c150f3d71ddd004d22b4dd25))

## [0.22.0](https://github.com/rtco-ai/rtco/compare/v0.21.1...v0.22.0) (2026-02-18)


### Features

* add `rtco wc` command for compact word/line/byte counts ([#175](https://github.com/rtco-ai/rtco/issues/175)) ([393fa5b](https://github.com/rtco-ai/rtco/commit/393fa5ba2bda0eb1f8655a34084ea4c1e08070ae))

## [0.21.1](https://github.com/rtco-ai/rtco/compare/v0.21.0...v0.21.1) (2026-02-17)


### Bug Fixes

* gh run view drops --log-failed, --log, --json flags ([#159](https://github.com/rtco-ai/rtco/issues/159)) ([d196c2d](https://github.com/rtco-ai/rtco/commit/d196c2d2df9b7a807e02ace557a4eea45cfee77d))

## [0.21.0](https://github.com/rtco-ai/rtco/compare/v0.20.1...v0.21.0) (2026-02-17)


### Features

* **docker:** add docker compose support ([#110](https://github.com/rtco-ai/rtco/issues/110)) ([510c491](https://github.com/rtco-ai/rtco/commit/510c491238731b71b58923a0f20443ade6df5ae7))

## [0.20.1](https://github.com/rtco-ai/rtco/compare/v0.20.0...v0.20.1) (2026-02-17)


### Bug Fixes

* install to ~/.local/bin instead of /usr/local/bin (closes [#155](https://github.com/rtco-ai/rtco/issues/155)) ([#161](https://github.com/rtco-ai/rtco/issues/161)) ([0b34772](https://github.com/rtco-ai/rtco/commit/0b34772a679f3c6b5dd9609af2f6eec6d79e4a64))

## [0.20.0](https://github.com/rtco-ai/rtco/compare/v0.19.0...v0.20.0) (2026-02-16)


### Features

* add hook audit mode for verifiable rewrite metrics ([#151](https://github.com/rtco-ai/rtco/issues/151)) ([70c3786](https://github.com/rtco-ai/rtco/commit/70c37867e7282ee0ccf200022ecef8c6e4ab52f4))

## [0.19.0](https://github.com/rtco-ai/rtco/compare/v0.18.1...v0.19.0) (2026-02-16)


### Features

* tee raw output to file for LLM re-read without re-run ([#134](https://github.com/rtco-ai/rtco/issues/134)) ([a08a62b](https://github.com/rtco-ai/rtco/commit/a08a62b4e3b3c6a2ad933978b1143dcfc45cf891))

## [0.18.1](https://github.com/rtco-ai/rtco/compare/v0.18.0...v0.18.1) (2026-02-15)


### Bug Fixes

* update ARCHITECTURE.md version to 0.18.0 ([398cb08](https://github.com/rtco-ai/rtco/commit/398cb08125410a4de11162720cf3499d3c76f12d))
* update version references to 0.16.0 in README.md and CLAUDE.md ([ec54833](https://github.com/rtco-ai/rtco/commit/ec54833621c8ca666735e1a08ed5583624b250c1))
* update version references to 0.18.0 in docs ([c73ed47](https://github.com/rtco-ai/rtco/commit/c73ed470a79ab9e4771d2ad65394859e672b4123))

## [0.18.0](https://github.com/rtco-ai/rtco/compare/v0.17.0...v0.18.0) (2026-02-15)


### Features

* **gain:** colored dashboard with efficiency meter and impact bars ([#129](https://github.com/rtco-ai/rtco/issues/129)) ([606b86e](https://github.com/rtco-ai/rtco/commit/606b86ed43902dc894e6f1711f6fe7debedc2530))

## [0.17.0](https://github.com/rtco-ai/rtco/compare/v0.16.0...v0.17.0) (2026-02-15)


### Features

* **cargo:** add cargo nextest support with failures-only output ([#107](https://github.com/rtco-ai/rtco/issues/107)) ([68fd570](https://github.com/rtco-ai/rtco/commit/68fd570f2b7d5aaae7b37b07eb24eae21542595e))
* **hook:** handle global options before subcommands ([#99](https://github.com/rtco-ai/rtco/issues/99)) ([7401f10](https://github.com/rtco-ai/rtco/commit/7401f1099f3ef14598f11947262756e3f19fce8f))

## [0.16.0](https://github.com/rtco-ai/rtco/compare/v0.15.4...v0.16.0) (2026-02-14)


### Features

* **python:** add lint dispatcher + universal format command ([#100](https://github.com/rtco-ai/rtco/issues/100)) ([4cae6b6](https://github.com/rtco-ai/rtco/commit/4cae6b6c9a4fbc91c56a99f640d217478b92e6d9))

## [0.15.4](https://github.com/rtco-ai/rtco/compare/v0.15.3...v0.15.4) (2026-02-14)


### Bug Fixes

* **git:** fix for issue [#82](https://github.com/rtco-ai/rtco/issues/82) ([04e6bb0](https://github.com/rtco-ai/rtco/commit/04e6bb032ccd67b51fb69e326e27eff66c934043))
* **git:** Returns "Not a git repository" when git status is executed in a non-repo folder [#82](https://github.com/rtco-ai/rtco/issues/82) ([d4cb2c0](https://github.com/rtco-ai/rtco/commit/d4cb2c08100d04755fa776ec8000c0b9673e4370))

## [0.15.3](https://github.com/rtco-ai/rtco/compare/v0.15.2...v0.15.3) (2026-02-13)


### Bug Fixes

* prevent UTF-8 panics on multi-byte characters ([#93](https://github.com/rtco-ai/rtco/issues/93)) ([155e264](https://github.com/rtco-ai/rtco/commit/155e26423d1fe2acbaed3dc1aab8c365324d53e0))

## [0.15.2](https://github.com/rtco-ai/rtco/compare/v0.15.1...v0.15.2) (2026-02-13)


### Bug Fixes

* **hook:** use POSIX character classes for cross-platform grep compatibility ([#98](https://github.com/rtco-ai/rtco/issues/98)) ([4aafc83](https://github.com/rtco-ai/rtco/commit/4aafc832d4bdd438609358e2737a96bee4bb2467))

## [0.15.1](https://github.com/rtco-ai/rtco/compare/v0.15.0...v0.15.1) (2026-02-12)


### Bug Fixes

* improve CI reliability and hook coverage ([#95](https://github.com/rtco-ai/rtco/issues/95)) ([ac80bfa](https://github.com/rtco-ai/rtco/commit/ac80bfa88f91dfaf562cdd786ecd3048c554e4f7))
* **vitest:** robust JSON extraction for pnpm/dotenv prefixes ([#92](https://github.com/rtco-ai/rtco/issues/92)) ([e5adba8](https://github.com/rtco-ai/rtco/commit/e5adba8b214a6609cf1a2cda05f21bcf2a1adb94))

## [0.15.0](https://github.com/rtco-ai/rtco/compare/v0.14.0...v0.15.0) (2026-02-12)


### Features

* add Python and Go support ([#88](https://github.com/rtco-ai/rtco/issues/88)) ([a005bb1](https://github.com/rtco-ai/rtco/commit/a005bb15c030e16b7b87062317bddf50e12c6f32))
* **cargo:** aggregate test output into single line ([#83](https://github.com/rtco-ai/rtco/issues/83)) ([#85](https://github.com/rtco-ai/rtco/issues/85)) ([06b1049](https://github.com/rtco-ai/rtco/commit/06b10491f926f9eca4323c80d00530a1598ec649))
* make install-local.sh self-contained ([#89](https://github.com/rtco-ai/rtco/issues/89)) ([b82ad16](https://github.com/rtco-ai/rtco/commit/b82ad168533881757f45e28826cb0c4bd4cc6f97))

## [0.14.0](https://github.com/rtco-ai/rtco/compare/v0.13.1...v0.14.0) (2026-02-12)


### Features

* **ci:** automate Homebrew formula update on release ([#80](https://github.com/rtco-ai/rtco/issues/80)) ([a0d2184](https://github.com/rtco-ai/rtco/commit/a0d2184bfef4d0a05225df5a83eedba3c35865b3))


### Bug Fixes

* add website URL (rtco-ai.app) across project metadata ([#81](https://github.com/rtco-ai/rtco/issues/81)) ([c84fa3c](https://github.com/rtco-ai/rtco/commit/c84fa3c060c7acccaedb617852938c894f30f81e))
* update stale repo URLs from pszymkowiak/rtco to rtco-ai/rtco ([#78](https://github.com/rtco-ai/rtco/issues/78)) ([55d010a](https://github.com/rtco-ai/rtco/commit/55d010ad5eced14f525e659f9f35d051644a1246))

## [0.13.1](https://github.com/rtco-ai/rtco/compare/v0.13.0...v0.13.1) (2026-02-12)


### Bug Fixes

* **ci:** fix release artifacts not uploading ([#73](https://github.com/rtco-ai/rtco/issues/73)) ([bb20b1e](https://github.com/rtco-ai/rtco/commit/bb20b1e9e1619e0d824eb0e0b87109f30bf4f513))
* **ci:** fix release workflow not uploading artifacts to GitHub releases ([bd76b36](https://github.com/rtco-ai/rtco/commit/bd76b361908d10cce508aff6ac443340dcfbdd76))

## [0.13.0](https://github.com/rtco-ai/rtco/compare/v0.12.0...v0.13.0) (2026-02-12)


### Features

* **sqlite:** add custom sqlite db location ([6e181ae](https://github.com/rtco-ai/rtco/commit/6e181aec087edb50625e08b72fe7abdadbb6c72b))
* **sqlite:** add custom sqlite db location ([93364b5](https://github.com/rtco-ai/rtco/commit/93364b5457619201c656fc2423763fea77633f15))

## [0.12.0](https://github.com/rtco-ai/rtco/compare/v0.11.0...v0.12.0) (2026-02-09)


### Features

* **cargo:** add `cargo install` filtering with 80-90% token reduction ([645a773](https://github.com/rtco-ai/rtco/commit/645a773a65bb57dc2635aa405a6e2b87534491e3)), closes [#69](https://github.com/rtco-ai/rtco/issues/69)
* **cargo:** add cargo install filtering ([447002f](https://github.com/rtco-ai/rtco/commit/447002f8ba3bbd2b398f85db19b50982df817a02))

## [0.11.0](https://github.com/rtco-ai/rtco/compare/v0.10.0...v0.11.0) (2026-02-07)


### Features

* **init:** auto-patch settings.json for frictionless hook installation ([2db7197](https://github.com/rtco-ai/rtco/commit/2db7197e020857c02857c8ef836279c3fd660baf))

## [0.10.0](https://github.com/rtco-ai/rtco/compare/v0.9.4...v0.10.0) (2026-02-07)


### Features

* Hook-first installation with 99.5% token reduction ([e7f80ad](https://github.com/rtco-ai/rtco/commit/e7f80ad29481393d16d19f55b3c2171a4b8b7915))
* **init:** refactor to hook-first with slim RTCO.md ([9620f66](https://github.com/rtco-ai/rtco/commit/9620f66cd64c299426958d4d3d65bd8d1a9bc92d))

## [0.9.4](https://github.com/rtco-ai/rtco/compare/v0.9.3...v0.9.4) (2026-02-06)


### Bug Fixes

* **discover:** add cargo check support, wire RtkStatus::Passthrough, enhance rtco init ([d5f8a94](https://github.com/rtco-ai/rtco/commit/d5f8a9460421821861a32eedefc0800fb7720912))

## [0.9.3](https://github.com/rtco-ai/rtco/compare/v0.9.2...v0.9.3) (2026-02-06)


### Bug Fixes

* P0 crashes + cargo check + dedup utilities + discover status ([05078ff](https://github.com/rtco-ai/rtco/commit/05078ff2dab0c8745b9fb44b1d462c0d32ae8d77))
* P0 crashes + cargo check + dedup utilities + discover status ([60d2d25](https://github.com/rtco-ai/rtco/commit/60d2d252efbedaebae750b3122385b2377ab01eb))

## [0.9.2](https://github.com/rtco-ai/rtco/compare/v0.9.1...v0.9.2) (2026-02-05)


### Bug Fixes

* **git:** accept native git flags in add command (including -A) ([2ade8fe](https://github.com/rtco-ai/rtco/commit/2ade8fe030d8b1bc2fa294aa710ed1f5f877136f))
* **git:** accept native git flags in add command (including -A) ([40e7ead](https://github.com/rtco-ai/rtco/commit/40e7eadbaf0b89a54b63bea73014eac7cf9afb05))

## [0.9.1](https://github.com/rtco-ai/rtco/compare/v0.9.0...v0.9.1) (2026-02-04)


### Bug Fixes

* **tsc:** show every TypeScript error instead of collapsing by code ([3df8ce5](https://github.com/rtco-ai/rtco/commit/3df8ce552585d8d0a36f9c938d381ac0bc07b220))
* **tsc:** show every TypeScript error instead of collapsing by code ([67e8de8](https://github.com/rtco-ai/rtco/commit/67e8de8732363d111583e5b514d05e092355b97e))

## [0.9.0](https://github.com/rtco-ai/rtco/compare/v0.8.1...v0.9.0) (2026-02-03)


### Features

* add rtco tree + fix rtco ls + audit phase 1-2 ([278cc57](https://github.com/rtco-ai/rtco/commit/278cc5700bc39770841d157f9c53161f8d62df1e))
* audit phase 3 + tracking validation + rtco learn ([7975624](https://github.com/rtco-ai/rtco/commit/7975624d0a83c44dfeb073e17fd07dbc62dc8329))
* **git:** add fallback passthrough for unsupported subcommands ([32bbd02](https://github.com/rtco-ai/rtco/commit/32bbd025345872e46f67e8c999ecc6f71891856b))
* **grep:** add extra args passthrough (-i, -A/-B/-C, etc.) ([a240d1a](https://github.com/rtco-ai/rtco/commit/a240d1a1ee0d94c178d0c54b411eded6c7839599))
* **pnpm:** add fallback passthrough for unsupported subcommands ([614ff5c](https://github.com/rtco-ai/rtco/commit/614ff5c13f526f537231aaa9fa098763822b4ee0))
* **read:** add stdin support via "-" path ([060c38b](https://github.com/rtco-ai/rtco/commit/060c38b3c1ab29070c16c584ea29da3d5ca28f3d))
* rtco tree + fix rtco ls + full audit (phase 1-2-3) ([cb83da1](https://github.com/rtco-ai/rtco/commit/cb83da104f7beba3035225858d7f6eb2979d950c))


### Bug Fixes

* **docs:** escape HTML tags in rustdoc comments ([b13d92c](https://github.com/rtco-ai/rtco/commit/b13d92c9ea83e28e97847e0a6da696053364bbfc))
* **find:** rewrite with ignore crate + fix json stdin + benchmark pipeline ([fcc1462](https://github.com/rtco-ai/rtco/commit/fcc14624f89a7aa9742de4e7bc7b126d6d030871))
* **ls:** compact output (-72% tokens) + fix discover panic ([ea7cdb7](https://github.com/rtco-ai/rtco/commit/ea7cdb7a3b622f62e0a085144a637a22108ffdb7))

## [0.8.1](https://github.com/rtco-ai/rtco/compare/v0.8.0...v0.8.1) (2026-02-02)


### Bug Fixes

* allow git status to accept native flags ([a7ea143](https://github.com/rtco-ai/rtco/commit/a7ea1439fb99a9bd02292068625bed6237f6be0c))
* allow git status to accept native flags ([a27bce8](https://github.com/rtco-ai/rtco/commit/a27bce82f09701cb9df2ed958f682ab5ac8f954e))

## [0.8.0](https://github.com/rtco-ai/rtco/compare/v0.7.1...v0.8.0) (2026-02-02)


### Features

* add comprehensive security review workflow for PRs ([1ca6e81](https://github.com/rtco-ai/rtco/commit/1ca6e81bdf16a7eab503d52b342846c3519d89ff))
* add comprehensive security review workflow for PRs ([66101eb](https://github.com/rtco-ai/rtco/commit/66101ebb65076359a1530d8f19e11a17c268bce2))

## [0.7.1](https://github.com/pszymkowiak/rtco/compare/v0.7.0...v0.7.1) (2026-02-02)


### Features

* **execution time tracking**: Add command execution time metrics to `rtco gain` analytics
  - Total execution time and average time per command displayed in summary
  - Time column in "By Command" breakdown showing average execution duration
  - Daily breakdown (`--daily`) includes time metrics per day
  - JSON export includes `total_time_ms` and `avg_time_ms` fields
  - CSV export includes execution time columns
  - Backward compatible: historical data shows 0ms (pre-tracking)
  - Negligible overhead: <0.1ms per command
  - New SQLite column: `exec_time_ms` in commands table
* **parser infrastructure**: Three-tier fallback system for robust output parsing
  - Tier 1: Full JSON parsing with complete structured data
  - Tier 2: Degraded parsing with regex fallback and warnings
  - Tier 3: Passthrough with truncated raw output and error markers
  - Guarantees RTCO never returns false data silently
* **migrate commands to OutputParser**: vitest, playwright, pnpm now use robust parsing
  - JSON parsing with safe fallbacks for all modern JS tooling
  - Improved error handling and debugging visibility
* **local LLM analysis**: Add economics analysis and comprehensive test scripts
  - `scripts/rtco-economics.sh` for token savings ROI analysis
  - `scripts/test-all.sh` with 69 assertions covering all commands
  - `scripts/test-aristote.sh` for T3 Stack project validation


### Bug Fixes

* convert rtco ls from reimplementation to native proxy for better reliability
* trigger release build after release-please creates tag


### Documentation

* add execution time tracking test guide (TEST_EXEC_TIME.md)
* comprehensive parser infrastructure documentation (src/parser/README.md)

## [0.7.0](https://github.com/pszymkowiak/rtco/compare/v0.6.0...v0.7.0) (2026-02-01)


### Features

* add discover command, auto-rewrite hook, and git show support ([ff1c759](https://github.com/pszymkowiak/rtco/commit/ff1c7598c240ca69ab51f507fe45d99d339152a0))
* discover command, auto-rewrite hook, git show ([c9c64cf](https://github.com/pszymkowiak/rtco/commit/c9c64cfd30e2c867ce1df4be508415635d20132d))


### Bug Fixes

* forward args in rtco git push/pull to support -u, remote, branch ([4bb0130](https://github.com/pszymkowiak/rtco/commit/4bb0130695ad2f5d91123afac2e3303e510b240c))

## [0.6.0](https://github.com/pszymkowiak/rtco/compare/v0.5.2...v0.6.0) (2026-02-01)


### Features

* cargo build/test/clippy with compact output ([bfd5646](https://github.com/pszymkowiak/rtco/commit/bfd5646f4eac32b46dbec05f923352a3e50c19ef))
* curl with auto-JSON detection ([314accb](https://github.com/pszymkowiak/rtco/commit/314accbfd9ac82cc050155c6c47dfb76acab14ce))
* gh pr create/merge/diff/comment/edit + gh api ([517a93d](https://github.com/pszymkowiak/rtco/commit/517a93d0e4497414efe7486410c72afdad5f8a26))
* git branch, fetch, stash, worktree commands ([bc31da8](https://github.com/pszymkowiak/rtco/commit/bc31da8ad9d9e91eee8af8020e5bd7008da95dd2))
* npm/npx routing, pnpm build/typecheck, --skip-env flag ([49b3cf2](https://github.com/pszymkowiak/rtco/commit/49b3cf293d856ff3001c46cff8fee9de9ef501c5))
* shared infrastructure for new commands ([6c60888](https://github.com/pszymkowiak/rtco/commit/6c608880e9ecbb2b3569f875e7fad37d1184d751))
* shared infrastructure for new commands ([9dbc117](https://github.com/pszymkowiak/rtco/commit/9dbc1178e7f7fab8a0695b624ed3744ab1a8bf02))

## [0.5.2](https://github.com/pszymkowiak/rtco/compare/v0.5.1...v0.5.2) (2026-01-30)


### Bug Fixes

* release pipeline trigger and version-agnostic package URLs ([108d0b5](https://github.com/pszymkowiak/rtco/commit/108d0b5ea316ab33c6998fb57b2caf8c65ebe3ef))
* release pipeline trigger and version-agnostic package URLs ([264539c](https://github.com/pszymkowiak/rtco/commit/264539cf20a29de0d9a1a39029c04cb8eb1b8f10))

## [0.5.1](https://github.com/pszymkowiak/rtco/compare/v0.5.0...v0.5.1) (2026-01-30)


### Bug Fixes

* 3 issues (latest tag, ccusage fallback, versioning) ([d773ec3](https://github.com/pszymkowiak/rtco/commit/d773ec3ea515441e6c62bbac829f45660cfaccde))
* patrick's 3 issues (latest tag, ccusage fallback, versioning) ([9e322e2](https://github.com/pszymkowiak/rtco/commit/9e322e2aee9f7239cf04ce1bf9971920035ac4bb))

## [0.5.0](https://github.com/pszymkowiak/rtco/compare/v0.4.0...v0.5.0) (2026-01-30)


### Features

* add comprehensive claude code economics analysis ([ec1cf9a](https://github.com/pszymkowiak/rtco/commit/ec1cf9a56dd52565516823f55f99a205cfc04558))
* comprehensive economics analysis and code quality improvements ([8e72e7a](https://github.com/pszymkowiak/rtco/commit/8e72e7a8b8ac7e94e9b13958d8b6b8e9bf630660))


### Bug Fixes

* comprehensive code quality improvements ([5b840cc](https://github.com/pszymkowiak/rtco/commit/5b840cca492ea32488d8c80fd50d3802a0c41c72))
* optimize HashMap merge and add safety checks ([3b847f8](https://github.com/pszymkowiak/rtco/commit/3b847f863a90b2e9a9b7eb570f700a376bce8b22))

## [0.4.0](https://github.com/pszymkowiak/rtco/compare/v0.3.1...v0.4.0) (2026-01-30)


### Features

* add comprehensive temporal audit system for token savings analytics ([76703ca](https://github.com/pszymkowiak/rtco/commit/76703ca3f5d73d3345c2ed26e4de86e6df815aff))
* Comprehensive Temporal Audit System for Token Savings Analytics ([862047e](https://github.com/pszymkowiak/rtco/commit/862047e387e95b137973983b4ebad810fe5b4431))

## [0.3.1](https://github.com/pszymkowiak/rtco/compare/v0.3.0...v0.3.1) (2026-01-29)


### Bug Fixes

* improve command robustness and flag support ([c2cd691](https://github.com/pszymkowiak/rtco/commit/c2cd691c823c8b1dd20d50d01486664f7fd7bd28))
* improve command robustness and flag support ([d7d8c65](https://github.com/pszymkowiak/rtco/commit/d7d8c65b86d44792e30ce3d0aff9d90af0dd49ed))

## [0.3.0](https://github.com/pszymkowiak/rtco/compare/v0.2.1...v0.3.0) (2026-01-29)


### Features

* add --quota flag to rtco gain with tier-based analysis ([26b314d](https://github.com/pszymkowiak/rtco/commit/26b314d45b8b0a0c5c39fb0c17001ecbde9d97aa))
* add CI/CD automation (release management and automated metrics) ([22c3017](https://github.com/pszymkowiak/rtco/commit/22c3017ed5d20e5fb6531cfd7aea5e12257e3da9))
* add GitHub CLI integration (depends on [#9](https://github.com/pszymkowiak/rtco/issues/9)) ([341c485](https://github.com/pszymkowiak/rtco/commit/341c48520792f81889543a5dc72e572976856bbb))
* add GitHub CLI integration with token optimizations ([0f7418e](https://github.com/pszymkowiak/rtco/commit/0f7418e958b23154cb9dcf52089a64013a666972))
* add modern JavaScript tooling support ([b82fa85](https://github.com/pszymkowiak/rtco/commit/b82fa85ae5fe0cc1f17d8acab8c6873f436a4d62))
* add modern JavaScript tooling support (lint, tsc, next, prettier, playwright, prisma) ([88c0174](https://github.com/pszymkowiak/rtco/commit/88c0174d32e0603f6c5dcc7f969fa8f988573ec6))
* add Modern JS Stack commands to benchmark script ([b868987](https://github.com/pszymkowiak/rtco/commit/b868987f6f48876bb2ce9a11c9cad12725401916))
* add quota analysis with multi-tier support ([64c0b03](https://github.com/pszymkowiak/rtco/commit/64c0b03d4e4e75a7051eac95be2d562797f1a48a))
* add shared utils module for JS stack commands ([0fc06f9](https://github.com/pszymkowiak/rtco/commit/0fc06f95098e00addf06fe71665638ab2beb1aac))
* CI/CD automation (versioning, benchmarks, README auto-update) ([b8bbfb8](https://github.com/pszymkowiak/rtco/commit/b8bbfb87b4dc2b664f64ee3b0231e346a2244055))


### Bug Fixes

* **ci:** correct rust-toolchain action name ([9526471](https://github.com/pszymkowiak/rtco/commit/9526471530b7d272f32aca38ace7548fd221547e))

  - Supports generate, migrate (dev/status/deploy), and db push
- `utils` module with common utilities (truncate, strip_ansi, execute_command)
  - Shared functionality for consistent output formatting
  - ANSI escape code stripping for clean parsing

### Changed
- Refactored duplicated code patterns into `utils.rs` module
- Improved package manager detection across all modern JS commands

## [0.2.1] - 2026-01-29

See upstream: https://github.com/pszymkowiak/rtco

## Links

- **Repository**: https://github.com/rtco-ai/rtco (maintained by pszymkowiak)
- **Issues**: https://github.com/rtco-ai/rtco/issues
