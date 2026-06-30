# Synerix 深度技术分析与重构方案

> 文档版本: 0.1.0  
> 分析日期: 2026-06-29  
> 分析范围: `crates/synerix` + `crates/synerix-core` 全量源码

---

## 一、现状问题分析报告

### 1. 架构演进维度

#### 1.1 核心问题：巨型对象与职责耦合

**App 上帝对象** (`app/state/app.rs`)
- `App` 结构体包含 **25+ 个公开字段**，集成了配置、聊天状态、侧边栏、Diff、状态栏、输入缓冲区、按键绑定、Yank 缓冲区、Slash 菜单、布局状态、Agent 通道、Config 重载通道、技能注册表、Agent 注册表、Goal 状态、编码模式、项目上下文、Session 存储、审批状态等
- 违反单一职责原则，任何模块的变更都需触碰 `App`
- 测试构造极其繁琐（见 `slash/mod.rs` 中 50+ 行的 `make_app()`）

**层间边界模糊**
- `app/` 模块同时承担状态定义、事件循环、输入处理、动作执行、渲染入口
- TUI 层 (`tui/`) 直接访问 `App` 的全部字段，无视图模型隔离
- Agent 层在 `App::spawn_agent_response` 中直接构造 LLM、ContextManager、ToolRegistry、McpManager，业务逻辑侵入 UI 层

#### 1.2 并发与性能瓶颈

**重复创建昂贵资源**
- `App::spawn_agent_response` 每次用户提交都重新创建：
  - `reqwest::Client`（连接池丢失）
  - `ContextManager` + TokenBudget
  - `ToolRegistry` + 重新注册所有 builtin
  - `McpManager::connect_all`（重新连接所有 MCP 服务器）
- 未实现连接复用，首次响应延迟极高

**阻塞式 Session 存储**
- `SessionStore` 使用 `Mutex<Connection>`，rusqlite 连接非 `Sync`，导致所有数据库操作阻塞 Tokio 运行时
- 虽注释解释为 WAL 模式下 contention 低，但高并发写入时仍会阻塞

**上下文管理粗粒度**
- `ContextManager::trim_to_budget` 仅按 token 数截断，无语义摘要
- `compress_old_tool_results` 仅保留最后 4 条，压缩策略为简单截断，丢失关键信息
- 无长期记忆 / 向量检索机制

**MCP 传输层局限**
- `StdioTransport` 使用单通道 `mpsc::channel` 做请求/响应，**无请求 ID 关联**
- `send_and_wait` 仅取第一个响应，多并发请求时结果错乱
- HTTP transport 完全未实现（返回 `Err`）

#### 1.3 可扩展性缺陷

- 插件系统 (`plugins/`) 已定义 `Plugin` trait 和 `PluginManager`，但**未接入主应用生命周期**
- 多 Agent 编排 (`agent/multi/`) 的 `AgentSwarm` 有 `coordinate` 方法，但 `run_task` 是占位实现，未真正驱动 LLM
- Workflow 引擎的 `execute_step_with_retry` 是**模拟执行**，未调用实际 Agent
- `model_catalog.rs` 硬编码模型能力映射，无外部数据源

---

### 2. 代码标准化维度

#### 2.1 死代码泛滥

| 文件 | 死代码标记 | 说明 |
|------|-----------|------|
| `error.rs` | `PluginEventPartialFailure`, `PluginInitPartialFailure` | 标记 `#[allow(dead_code)]`，插件系统未接入 |
| `app/state/app.rs` | `new()` 方法 | 标记 `#[allow(dead_code)]`，但 runner 使用 `new_with_settings` |
| `app/state/diff.rs` | `hunks: Vec<DiffHunk>` | 字段存在但 `diff_renderer.rs` 自行解析 |
| `tui/diff_renderer.rs` | 整个模块 | `#![allow(dead_code)]`，但 `diff_view.rs` 已调用 |
| `tools/registry.rs` | 整个模块 | `#![allow(dead_code)]` |
| `tools/traits.rs` | 整个模块 | `#![allow(dead_code)]` |
| `tools/builtin/mod.rs` | 整个模块 | `#![allow(dead_code)]` |
| `llm/adapter.rs` | 整个模块 | `#![allow(dead_code)]` |
| `llm/provider.rs` | 整个模块 | `#![allow(dead_code)]` |
| `mcp/manager.rs` | 整个模块 | `#![allow(dead_code)]` |
| `mcp/mcp_client.rs` | `reconnect_count`, `reconnect` | 标记 `#[allow(dead_code)]` |
| `sandbox/risk_classifier.rs` | 整个模块 | `#![allow(dead_code)]` |
| `sandbox/audit.rs` | 整个模块 | `#![allow(dead_code)]` |
| `sandbox/atomic_writer.rs` | 整个模块 | `#![allow(dead_code)]` |
| `agent/roles.rs` | 整个模块 | `#![allow(dead_code)]` |
| `agent/prompt.rs` | 整个模块 | `#![allow(dead_code)]` |
| `skills/external.rs` | 整个模块 | `#![allow(dead_code)]` |
| `project/detector/parsers.rs` | 整个模块 | `#![allow(dead_code)]` |

#### 2.2 命名规范不一致

- **文件名**：`slash_menu.rs` (app/state/) vs `slash-menu` (概念名) vs `slash/` (模块目录)
- **模块名**：`app::state::app` 与 `app::App` 同名，遮蔽严重
- **字段名**：`status_bar.sandbox_mode: String` 与 `sandbox.mode: SandboxMode` 语义重复但类型不同
- **常量**：`STATUS_BG` 硬编码在 `status_bar.rs`，`COLOR_DARK_GRAY` 在 `theme.rs`，无统一常量管理
- **函数名**：`clean_terminal_text`、`clean_display_text`、`clean_markdown_line` — "clean" 语义模糊

#### 2.3 文档与注释问题

- 大量 `#[allow(dead_code)]` 掩盖了真实的未完成功能
- `TODO` 注释：`token_estimator.rs` 第 9 行 "TODO: Token estimator — not yet wired"
- 中文注释与英文注释混用，风格不统一
- `///` 文档注释与 `//` 行注释比例失衡

---

### 3. 逻辑极致精简维度

#### 3.1 重复逻辑

**LLM 初始化重复**
```rust
// app/state/app.rs:190-244 与 agent/agent_loop.rs:49-50
// 每次提交都重建：LLM adapter, ContextManager, ToolRegistry, McpManager
```

**字符导航逻辑重复**
- `app/state/app.rs`: `prev_char_pos()`, `next_char_pos()`
- `app/input_handler.rs`: `move_cursor_left()`, `move_cursor_right()`, `delete_char_before_cursor()`, `delete_char_after_cursor()`
- `app/actions.rs`: 内联相同逻辑
- 三处实现相同功能，维护困难

**滚动逻辑重复**
- `app/actions.rs`: `ScrollUp`, `ScrollDown`, `ScrollPageUp`, `ScrollPageDown`
- `app/input_handler.rs`: Normal 模式下的 `j`/`k`/`G`
- `app/event_loop.rs`: `handle_mouse_scroll_up/down`
- 三处滚动逻辑分散

**动画点重复**
- `tui/widgets/chat_area.rs`: `animated_dots()`
- `tui/widgets/status_bar.rs`: `animated_dots()`
- 完全相同的实现

**Markdown/HTML 清洗重复**
- `tui/widgets/chat_area.rs`: `clean_display_text` → `strip_html_tags` + `decode_html_entities` + `strip_markdown_markers` + `convert_markdown_tables`
- `slash/common.rs`: `clean_terminal_text` → 20+ 链式 `.replace()`
- 两套清洗逻辑，目标相同但实现不同

**Diff 渲染重复**
- `tui/diff_renderer.rs`: `render_unified` 和 `render_side_by_side` 都重复了 hunk header 渲染和 `guess_extension`

**Config 重载逻辑重复**
- `config/config_watcher.rs`: `watch_loop` 和 `handle_sighup` 都执行 `Settings::load()` → `tx.send(ConfigReload)`

#### 3.2 过度封装

- `App::new_with_channel` → `App::new` → `App::new_with_settings` 三层构造器，参数传递冗余
- `ToolRegistry` 包装 `HashMap<String, Arc<dyn Tool>>`，但仅提供 `get/register/all_schemas`，无额外逻辑
- `SkillRegistry` 类似，仅包装 `Vec<SkillDef>`
- `ContextManager` 的 `TokenBudget` 有 4 层字段，但 `available` 是派生值，可计算属性

#### 3.3 效率问题

- `slash/common.rs:29-52` `clean_terminal_text`：20+ 次 `String::replace`，每次分配新字符串
- `skills/registry.rs:56-68` `match_skills`：对每个 keyword 都调用 `to_lowercase()`，O(n*m) 且大量分配
- `project/detector/parsers.rs:13-110` `detect_languages`：根目录和子目录重复相同的 extension match 逻辑
- `risk_classifier.rs:78-104` `classify_command`：每个风险级别都 `to_ascii_lowercase()` 一次

---

### 4. TUI 交互升级维度

#### 4.1 视觉系统缺陷

**硬编码颜色散落**
- `approval_popup.rs`: `Color::Rgb(30, 30, 40)`, `Color::Yellow`
- `status_bar.rs`: `STATUS_BG: Color = Color::Rgb(40, 42, 58)`
- `diff_renderer.rs`: `Color::Rgb(22, 40, 22)` 等硬编码
- `chat_area.rs`: ASCII art 硬编码

**主题系统不完整**
- `theme.rs` 定义了 `ColorPalette`，但 widgets 未统一使用
- `BORDER_TYPE` 是 `Rounded`，但 `input_box.rs` 和 `slash_menu.rs` 使用 `Plain`
- 无焦点状态视觉强化（除 border 颜色外）

#### 4.2 布局与交互

**固定尺寸假设**
- `layout.rs`: 侧边栏宽度 `116` 是硬阈值，无平滑过渡
- Diff 面板高度固定 `9`，大 diff 内容显示受限
- Slash 菜单宽度固定 `78`，可能溢出

**滚动体验原始**
- `chat_area.rs` 使用 `split_off + truncate` 做滚动，**修改原始 Vec**，但函数签名是 `&App`（不可变）— 实际通过 `unsafe` 或内部可变性？不，这里是复制，但 `split_off` 消耗原始 Vec
- 无平滑滚动，无鼠标拖拽滚动条
- 无搜索高亮

**输入体验**
- `input_box.rs` 手动计算 cursor 位置，对宽字符（CJK）支持可能有问题
- 无自动补全（除 Slash 命令外）
- 无输入历史导航（上下箭头）

---

## 二、重构实施路线图

### Phase 1: 基础清理与标准化（1-2 周）

**目标**：消除死代码，建立命名规范，修复明显的逻辑重复

#### 1.1 死代码清理
- 移除所有 `#[allow(dead_code)]` 模块级标记，逐一确认是否真正需要
- 删除未实现的 `reconnect`、`reconnect_count`、`hunks` 等死字段
- 移除 `AppError` 中未使用的变体（`PluginEventPartialFailure`、`PluginInitPartialFailure`）
- 移除 `new()` 构造器，统一使用 `new_with_settings`

#### 1.2 命名规范化
- 重命名 `app/state/slash_menu.rs` → `app/state/slash_menu_state.rs`（避免与 `slash/` 模块混淆）
- 统一 `clean_*` 函数名为 `sanitize_*` 或 `render_*`
- `StatusBarState.sandbox_mode: String` → 移除或同步为 `SandboxMode` 枚举

#### 1.3 工具函数抽取
- 抽取 `char_pos_*` 光标导航到 `app::cursor` 子模块
- 抽取 `scroll_*` 滚动逻辑到 `app::scroll` 子模块
- 统一 `animated_dots` 到 `tui::animation` 模块

### Phase 2: 架构分层与性能优化（2-3 周）

**目标**：引入分层架构，消除重复初始化，提升并发性能

#### 2.1 引入领域层
- 创建 `app::domain` 模块，定义 `ChatService`、`SessionService`、`ConfigService`
- `App` 退化为纯状态容器 + 轻量协调器
- 服务层负责 LLM 初始化、上下文管理、工具调度

#### 2.2 资源池化
- 实现 `LlmPool`：复用 `reqwest::Client` 和 `OpenAICompatAdapter`
- 实现 `McpPool`：复用 MCP 连接，避免每次提交重建
- `ContextManager` 升级为支持语义压缩（摘要 + 关键信息保留）

#### 2.3 事件总线统一
- 用 `tokio::sync::broadcast` 或 `bus` crate 替代当前零散的 `mpsc` 通道
- 统一 `AgentEvent`、`ConfigReload`、`PluginEvent` 为 `AppEvent` enum

#### 2.4 Session 存储异步化
- 将 `SessionStore` 操作封装为 `spawn_blocking`，避免阻塞 Tokio 运行时
- 或迁移到 `sqlx` + `AsyncConnection`（如支持）

### Phase 3: DRY 极致精简（1-2 周）

**目标**：消除重复逻辑，统一字符串处理，优化热点路径

#### 3.1 字符串处理管线
- 用 `smart-default` + `once_cell` 统一 `clean_terminal_text` / `clean_display_text`
- 实现 `TextSanitizer` pipeline，支持配置化清洗步骤

#### 3.2 配置重载统一
- `config_watcher.rs` 的 `watch_loop` 和 `handle_sighup` 合并为单一 `reload_task`

#### 3.3 关键词匹配优化
- `skills/registry.rs` 预 lowercase 所有 keyword，避免每次匹配时重新转换

#### 3.4 Diff 渲染统一
- 提取 `render_hunk_header`、`guess_extension` 为共享函数
- 统一 side-by-side 和 unified 的 color mapping

### Phase 4: TUI 视觉与交互升级（2-3 周）

**目标**：现代化视觉设计，提升交互流畅度

#### 4.1 设计系统建立
- 引入 `tui::theme::DesignToken` 系统（间距、圆角、阴影、字体大小）
- 所有 widget 强制通过 `theme` 获取颜色，禁止硬编码 RGB
- 实现 `LightTheme` 和 `DarkTheme` 完整双主题

#### 4.2 布局增强
- 响应式侧边栏：宽度按 `area.width` 平滑计算（0/16/24/32）
- Diff 面板高度按内容动态调整（`min(9, max(3, content_lines))`）
- 支持面板拖拽调整大小

#### 4.3 交互优化
- 实现 `InputHistory`（上下箭头导航）
- 实现 `SearchHighlighter`（搜索高亮 + 下一条匹配）
- 优化 Slash 菜单：模糊匹配、图标、分组
- 实现进度条（Agent 思考中、Tool 执行中）

#### 4.4 动画系统
- 统一 `AnimationController`，管理所有动画帧
- 光标闪烁、思考指示器、加载 spinner 共用时间轴
- 可选：ASCII art 欢迎屏渐入动画

---

## 三、实施优先级矩阵

| 任务 | 影响 | 难度 | 优先级 |
|------|------|------|--------|
| 移除死代码 | 中 | 低 | P0 |
| 统一构造器 | 低 | 低 | P0 |
| 抽取光标/滚动工具函数 | 中 | 低 | P0 |
| 统一 animated_dots | 低 | 低 | P1 |
| 统一字符串清洗 | 高 | 中 | P1 |
| 资源池化 (LLM/MCP) | 高 | 高 | P1 |
| 事件总线统一 | 中 | 高 | P2 |
| 领域层拆分 | 高 | 高 | P2 |
| Session 异步化 | 中 | 中 | P2 |
| 设计系统建立 | 高 | 中 | P3 |
| 响应式布局 | 中 | 中 | P3 |
| 输入历史/搜索 | 中 | 中 | P3 |
| 动画系统 | 低 | 中 | P4 |

---

## 四、质量保障

- 每 Phase 完成后运行 `cargo clippy --all-targets -- -D warnings`
- 每 Phase 完成后运行 `cargo test --workspace`
- 建立 `CONTRIBUTING.md` 规范：
  - 禁止新增 `#[allow(dead_code)]`
  - 禁止硬编码 RGB 值（必须通过 `theme`）
  - 新功能必须包含测试
- 引入 `cargo-deny` 做依赖审计
- 引入 `cargo-audit` 做安全审计
