# Changelog / 更新日志

All notable changes to Synerix are documented here.

Synerix 的所有重要更改都会记录在这里。

Synerix follows [Semantic Versioning](https://semver.org/):

Synerix 遵循[语义化版本控制](https://semver.org/)：

- **Major**: Breaking API changes / 重大 API 变更
- **Minor**: New features, backward compatible / 新功能，向后兼容
- **Patch**: Bug fixes, backward compatible / 错误修复，向后兼容

---

## [0.2.3] — 2026-07-03

### Fixed / 修复

- **Dead loop fix** — Sending a message while the agent is busy caused state corruption. Added `agent_busy` guard to `submit_message()` entry point, reset on `AgentEvent::Done/Error` / 修复用户在处理中再次发送消息导致的死循环
- **"Device not configured" exit error** — `tui::init()` failure in non-interactive context was propagated as fatal via `?`. Changed to `match` for clean `Ok(())` exit / 修复非交互环境下的退出报错
- **UTF-8 safety** — `truncate_preview()` in `approval_popup.rs` used `&text[..max_chars]` which panics on multi-byte chars. Changed to `char_indices().nth()` / 修复多字节字符截断 panic
- **Alpha blend bug** — `FocusRing::alpha_color()` used wrong variable for green channel, making alpha ineffective. Fixed variable naming / 修复 alpha 混合绿色通道失效

### Changed / 变更

- **Single-column layout** — Removed sidebar, pure vertical: chat → diff(optional) → input → status / 单列垂直布局，移除侧边栏
- **Terminal-native scrolling** — Chat area uses terminal-native scroll with mouse wheel support (ScrollUp/ScrollDown) / 终端原生滚动，支持鼠标滚轮
- **Cached gradient separator** — `cached_gradient_separator()` uses Mutex cache to avoid rebuilding `width` spans per frame / 缓存梯度分隔线减少每帧分配
- **Static lookup tables** — Status bar token progress bar and sidebar tab labels use static strings, eliminating per-frame `format!()` allocations / 静态查找表消除每帧字符串分配
- **Documentation bilingual** — README, installation guide, mode docs, config docs all updated to Chinese + English / 文档全面中英双语
- **Modes reduced** — Coding modes from 5 (Plan/Act/Chat/Architect/Vibe) to 2 (Plan/Vibe), docs updated / 编码模式从 5 种缩减为 2 种

### Added / 新增

- **SVG assets** — Banner (1200×400) with gradient `>S` monogram + TUI mockup; Icon (64×64) with animated cursor blink / SVG 视觉资产
- **Easing functions** — `ease_in_out_cubic`, `ease_out_cubic`, `ease_in_out_quad`, `ease_out_quad`, `linear` in `animation.rs` / 缓动函数
- **Smooth animations** — `fade_in_alpha`, `fade_out_alpha`, `blink_alpha`, `pulse_sine`, `lerp_color` / 平滑动画效果

## [0.2.2] — 2026-06-11

### Added / 新增

- Vibe mode auto-iteration: generate → compile → test → fix / 沉浸模式自动迭代：生成 → 编译 → 测试 → 修复
- Smart sandbox with risk-aware approval flows / 带风险感知审批流的智能沙箱
- Multi-provider LLM support (DeepSeek, custom OpenAI-compatible) / 多提供商 LLM 支持（DeepSeek、自定义 OpenAI 兼容）
- Plugin system with command, skill, and MCP types / 插件系统（命令、技能、MCP 类型）
- Slash command framework with autocomplete / 带自动补全的斜杠命令框架
- Activity labels with animated status indicators / 带动画状态指示器的活动标签
- Model catalog for discoverable presets / 可发现预设的模型目录

### Changed / 变更

- Token count removed from status bar for a cleaner UI / 从状态栏移除令牌计数，界面更简洁
- Welcome screen replaced with ASCII art branding / 欢迎屏幕替换为 ASCII 艺术品牌
- Activity labels now use accent color for active states / 活动标签现在使用强调色表示活动状态
- TUI widgets refactored into composable components / TUI 组件重构为可组合组件

### Fixed / 修复

- Terminal display corruption on resize / 调整大小时的终端显示损坏
- Config hot reload reliability / 配置热重载可靠性
- Memory leak in streaming pipeline / 流管道中的内存泄漏

## [0.2.1] — 2026-05-15

### Added / 新增

- Chat mode with streaming responses / 带流式响应的对话模式
- Code review workflow / 代码审查工作流
- Git integration for session management / 用于会话管理的 Git 集成

### Changed / 变更

- Response latency reduced by 30% / 响应延迟降低 30%
- Error messaging improved across all modes / 所有模式的错误消息改进

### Fixed / 修复

- AI connection retry logic / AI 连接重试逻辑
- Command history persistence / 命令历史持久化

## [0.2.0] — 2026-04-01

### Added / 新增

- Act mode with sandboxed command execution / 带沙箱命令执行的执行模式
- Core AI integration layer / 核心 AI 集成层
- TOML-based configuration system / 基于 TOML 的配置系统
- Tokyo Night theme with light/dark variants / Tokyo Night 主题，支持浅色/深色变体

## [0.1.0] — 2026-03-01

### Added / 新增

- Project initialization / 项目初始化
- Core Rust workspace structure / 核心 Rust 工作空间结构
- Basic terminal UI with ratatui / 基于 ratatui 的基础终端 UI
