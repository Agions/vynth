# Changelog

All notable changes to Synerix will be documented in this file.

## [0.2.2] — 2026-06-27

### 🪟 Windows 原生安装支持

- **新增 `install.ps1`** — Windows PowerShell 安装脚本，等效 `curl | bash`
  - 一行命令：`iwr -useb https://raw.githubusercontent.com/Agions/synerix/main/install.ps1 | iex`
  - 自动检测架构（x86_64 / aarch64），下载对应预编译二进制
  - GitHub 优先 + Gitee 兜底的双源下载策略
  - 自动安装到 `%LOCALAPPDATA%\Programs\synerix`
  - 自动添加到用户 PATH（后续终端生效）
  - 支持 `$env:SYNERIX_HOME` 自定义安装目录

- **优化 `release.yml`** — Windows 构建产物改为 `.zip` 格式
  - 统一发布文件命名：`synerix-{tag}-{os}-{arch}.{ext}`
  - Linux: `.tar.gz` / macOS: `.tar.gz` / Windows: `.zip`

### 📖 文档

- **README 安装章节重构**：
  - 分平台展示（Linux/macOS / Windows）
  - Gitee 国内镜像同时展示 bash + powershell 命令
  - Windows 卸载指引

## [0.2.1] — 2026-06-08

### 🧹 全面死代码清理

- **删除 2 个死目录**：`tests/`（5 个文件）、`git/` 模块（433 行，零引用）
- **删除 9 个 `AppError` 死变体**（`ApprovalDenied`、`SessionNotFound`、`StreamClosed` 等）
- **删除假指标**：`db_open_ms` 启动指标、假字段移除
- **净删除 4,117 行**，净新增仅 96 行

### ♻️ DRY 原则 — 消除 4 处重复

- **`walk_dir()` 提取**：3 处复制 → `crate::util` 共享函数
- **`handle_command_key` / `handle_search_key` 合并**：30 行重复 → `handle_common_key` 共享
- **`App::new()` / `App::new_with_channel()` 合并**：45 行重复 → 3 行包装器
- **`Settings::defaults()` 默认值委托**：11 处硬编码转移到 `default_*()` 函数

### 🧩 架构精简

- **ToolRegistry 移除 Mutex 缓存**：44 行缓存 → 直接 HashMap 迭代，文件 203→159 行
- **Cargo.toml**：移除 `moka` 依赖、`criterion html_reports` 冗余 features
- **clippy 收紧**：`pedantic = "allow"` → 10 个精选 pedantic lint（`single_match`、`redundant_clone` 等），代码库已完全合规

### 🧪 测试

- **694 测试全过**，零失败
- `cargo check` 零错误零警告
- `cargo clippy` 全绿

## [0.2.0] — 2026-06-06

### 🧘 Vibe Coding Mode (沉浸式编程模式)

- **New `/mode vibe`** — 第五种编码模式，专为 AI 驱动心流设计
- **零阻碍执行**：低/中风险操作自动放行，不打断心流
- **自动迭代闭环**：编译/测试失败自动回注 LLM 循环修复，无需手动干预
- **系统提示词驱动**：6条中文指令注入 Vibe 行为规则（沉浸式迭代、错误驱动修复、即时验证）
- **别名支持**：`/mode v`、`/mode 沉浸`、`/mode 氛围` 均可切换
- **状态栏标识**：Teal 青绿色 `(#507882)` 视觉区隔，一眼识别 Vibe 模式

### 🎨 TUI 全面美化

- **Tokyo Night 主题统一**：所有面板使用 `BorderType::Rounded` 圆角边框
- **角色前缀彩色化**：`▸ 用户`(青色) / `◆ AI`(绿色) / `⚙ Tool`(紫色) — 一眼区分消息来源
- **布局优化**：sidebar 15% / chat 55% / diff 20% / input 10% — 信息密度更合理
- **输入框增强**：模式图标 (`⌨`/`✏`/`≡`) + 光标视觉效果优化
- **状态栏重写**：全主题颜色版，Vibe 模式专属茶绿配色
- **Diff 视图优化**：前置彩色字符（`+`绿 / `-`红 / `~`黄）

### 🪄 SVG 视觉重生

- **新 Logo** (512×512)：渐变 `>_S` 符号 + Vibe 波纹动感
- **新 Icon** (64×64)：精简版 Logo，适用 favicon / 系统托盘
- **新 Banner** (1200×340)：含 TUI 模拟截图，适配 README 头图
- 全部使用 CSS 无依赖内联 SVG，渲染零依赖

### 🧹 死代码清理 (ZERO 警告里程碑)

- **删除 3 个死模块**：`chat_bubble.rs`(342行)、`list_select.rs`(45行)、`spinner.rs`(37行)
- **theme.rs 精简 50%**：ColorPalette 29→14 字段，删除 Theme 枚举 + ThemeStyles + 9 个死函数 + 6 个死颜色常量
- **Sidebar/DiffView 纯函数化**：删除 Sidebar/DiffView 结构体及相关 Widget impl，保留纯 render() 函数
- **零编译器警告**：`#[allow(dead_code)]` 从 26 处降为 0，`cargo clippy -D warnings` 通过

### 📖 README 故事驱动重写

- **352 行全英文**：痛点表格 → 一行安装 → 6 能力卡片（2×3 网格）
- **叙事结构**：先讲 Why Synerix（痛点驱动），再亮方案，后跟快速上手
- **Vibe Coding 章节**：完整文档化新模式的启用方式和工作原则
- **配置精炼**：去样板化，只保留关键可调参数

### 🧪 测试

- **706 测试全过**，零失败，零警告
- 新增 5 项 Vibe 模式测试：权限验证、别名解析、display 标签
- 更新 phase3 集成测试适配精简后的 theme API

## [0.1.1] — 2026-06-01

### 🎛️ Slash Command System Redesign

- **Registry architecture**: Replaced hardcoded `match` routing with declarative `CmdDef` registration table — commands are now self-documenting structs with name, description, category, aliases, and handler
- **Unified argument parsing**: `subcmd()`, `nth_arg()`, `rest_from()` helper functions replace ad-hoc per-command parsing logic
- **Hierarchical help**: `/help` now displays commands grouped by category (💡 Help, 📋 Session, 🤖 Model, ⚙️ Config, 🎯 Goal, 📦 Workflow); `/help <cmd>` shows aliases and usage
- **Alias support**: `/h`, `/?`, `/c`, `/cls`, `/m`, `/re`, `/quit`, `/q`, `/wf`, `/skills`, `/cfg`, `/g` shortcuts for common commands
- **15 new tests** covering alias resolution, help system hierarchy, and command completeness

### 🧹 Audit Fixes

- **Release profile**: `lto = true` → `lto = "fat"` for cross-crate LTO optimization
- **CI**: Added `cargo audit` step for vulnerability scanning
- **Features**: Restructured with `default = ["tui"]`, added `headless` mode feature
- **TokenBudget**: Hardcoded `2000/3000/4096` replaced with named associated constants (`DEFAULT_SYSTEM_OVERHEAD`, `DEFAULT_TOOLS_OVERHEAD`, `DEFAULT_RESERVED`)
- **Security**: `Sandbox::Auto` mode now has explicit `⚠️ Security Warning` doc annotation
- **Documentation**: Added `//!` module-level docs to 5 command/workflow files

## [0.1.0] — 2026-06-01

### 🏗️ Architecture

- **Cargo Workspace**: Monolithic crate split into workspace with `synerix` (main) and `synerix-core` (core abstractions) sub-crates
- **Core crate (`synerix-core`)**: Extracted shared types including unified `Role` enum, `MutexExt` trait, and datetime utilities
- **Lint cleanup**: Removed `#![allow(dead_code, unused_imports, unused_variables)]` from lib root; all warnings now treated as errors via clippy CI
- **CI pipeline**: Release CI with matrix build (linux/macos/windows), uploaded artifacts, auto-generated release notes
- **Binary size**: 3.9MB release build with `lto=true`, `panic="abort"`, `strip=true`

### ✂️ Module Splitting

- **`tui/frame.rs`** → `layout.rs` (pure layout computation) + `renderer.rs` (orchestration) + 5 dedicated widget files
- **`agent/agloop.rs`** → `agent_loop.rs` (core loop) + `tool_dispatcher.rs` (timeout-aware tool dispatch)
- **`app/`** → reorganized into `state.rs`, `events.rs`, `message.rs`, `input_handler.rs`, `runner.rs`, `actions.rs`

### 📛 Naming Conventions

- Unified all `.rs` filenames to `snake_case` (7 renames: `trait_def` → `traits`, `watcher` → `config_watcher`, `command_preview` → `risk_classifier`, `atomic_replace` → `atomic_writer`, `loader` → `skill_loader`, `client` → `mcp_client`)
- CI now enforces naming conventions in PR checks

### 🎨 Code Quality

- **TUI style constants**: Added 10 color aliases and 7 helper functions in `theme.rs`; replaced ~45 inline style constructions across all widgets
- **Token budget**: Introduced `TokenBudget::from_config()` to eliminate hardcoded magic numbers (2000/3000); added `system_prompt_tokens` and `tools_schema_tokens` to `LlmConfig`
- **Mutex handling**: Unified via `MutexExt::lock_or_err()` trait

### 🧪 Testing

- Added 4 new unit tests for datetime parsing utility
- **1275 tests total**, all passing with zero clippy warnings

### 🔧 Configuration

- `panic = "abort"` in release profile for smaller binaries
- Features renamed to kebab-case (`startup-bench`)
- `tokio` features narrowed from `full` to precise list

### 🚀 Release

- Added CI Release pipeline (`release.yml`) for automated GitHub Releases on tags
- Binary size: **3.9 MB** (under 5 MB target)
- Zero `unsafe` blocks, zero nightly features

### ⚡ Performance

- **SessionStore locking**: Consolidate to `Mutex<Connection>` with WAL mode for minimal contention (`rusqlite::Connection` is `Send` but not `Sync`)
- **TUI dirty-flag rendering**: Per-widget dirty flags skip unchanged widgets (sidebar, chat, diff, input, status) — 60fps CPU reduction
- **Configurable tool timeout**: `SandboxConfig.tool_timeout_secs` (default 120s) controls tool execution timeout, configurable via `config.toml`
- **Benchmark suite**: `criterion` benchmarks for token estimation, context push/trim, session CRUD — run with `cargo bench`
