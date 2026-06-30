# Synerix 重构实施路线图

> 本文档是 `refactor-analysis-report.md` 的执行计划，按 Phase 分阶段实施。

---

## Phase 1: 基础清理与标准化

**时间估计**: 1-2 周  
**风险等级**: 低  
**验收标准**: `cargo clippy` 零警告，`cargo test` 全绿

### 1.1 死代码清理

- [x] `error.rs` 的 `PluginEventPartialFailure` / `PluginInitPartialFailure` — **确认插件系统已接入**（`lib.rs` 声明 `pub mod plugins`，`plugins/manager.rs` 实际构造并匹配这两个变体），故保留变体并移除其 `#[allow(dead_code)]` 标记
- [x] 删除 `app/state/app.rs` 中 `#[allow(dead_code)] pub fn new()`（runner 使用 `new_with_settings`，确为死构造器）
- [x] 清理 `app/state/diff.rs` 中 `hunks` 字段（write-only 死字段，连同 `slash/mod.rs` 的初始化点一并移除；`DiffHunk` 类型保留供 `diff_renderer.rs` 使用）
- [x] 批量移除模块级 `#![allow(dead_code)]`（registry, traits, builtin, adapter, provider, mcp, risk_classifier, atomic_writer, roles, prompt, parsers, context, multi/*, skills/builtin/*, session/* 等）
- [x] 删除整模块死代码：`skills/external.rs`、`sandbox/audit.rs`、`synerix-core/utils/sync.rs`（均无悬空引用）
- [x] 修复批量删除引入的 3 处硬编译错误：`custom.rs` serde import、`error.rs` 插件变体、`slash/mod.rs` `hunks` 初始化（详见审计说明）
- [x] 审计剩余 29 个 per-item `#[allow(dead_code)]`：均为合理标记（部分使用的类型如 `CommandPreview`/`coding_modes`/`approval`，以及已打开但未读写的 `SessionStore` CRUD），属 Phase 2 接线范围，Phase 1 不删

### 1.2 命名规范化

- [x] 重命名 `app/state/slash_menu.rs` → `slash_menu_state.rs`，更新 `mod.rs` 引用（消除与 `slash/` 模块的混淆）
- [x] 统一 `clean_*` 函数名为 `sanitize_*` / `render_plain_text`（迁移至新 `tui::text` 模块：`clean_display_text`→`render_plain_text`、`clean_terminal_text`→`sanitize_terminal_text`、`clean_markdown_line`→`sanitize_markdown_line`）
- [x] `StatusBarState.sandbox_mode` 改为 `SandboxMode` 枚举（移除 `String` 冗余；`SandboxMode` 派生 `Default=Confirm`，启动时从 `settings.sandbox.mode` 初始化，修复此前启动时 sandbox 标签为空的缺陷）

### 1.3 工具函数抽取

- [x] 创建 `app::cursor` 模块，统一 `prev_char_pos` / `next_char_pos` / `move_cursor_left` / `move_cursor_right` / `delete_char_before_cursor` / `delete_char_after_cursor`（`actions.rs` 内联实现一并改为调用，消除三处重复）
- [x] 创建 `app::scroll` 模块，统一聊天滚动逻辑为 `scroll_chat_older` / `scroll_chat_newer` / `scroll_chat_to_bottom`（迁移 `actions.rs` 的 `Scroll*`、`input_handler.rs` 的 `j`/`k`/`G`、`event_loop.rs` 的鼠标滚轮三处调用点）
- [x] 创建 `tui::animation` 模块，统一 `animated_dots`（移除 `chat_area.rs` 与 `status_bar.rs` 的两份重复实现）
- [x] 创建 `tui::text` 模块，统一 HTML/Markdown 清洗管线（合并 `chat_area` 与 `slash::common` 两套清洗逻辑）

---

## Phase 2: 架构分层与性能优化

**时间估计**: 2-3 周  
**风险等级**: 中  
**验收标准**: 首响延迟降低 50%，并发吞吐量提升

### 2.1 领域层拆分

- [ ] 创建 `app::domain::ChatService` — 负责消息提交、Agent 调度
- [ ] 创建 `app::domain::ConfigService` — 负责配置加载、热重载
- [ ] 创建 `app::domain::SessionService` — 负责 Session 持久化
- [ ] `App` 只持有状态 + 服务引用，不包含业务逻辑

### 2.2 资源池化

- [ ] 实现 `LlmPool` — 单例 `reqwest::Client` + 按模型缓存 Adapter
- [ ] 实现 `McpPool` — 连接复用 + 自动重连
- [ ] `ContextManager` 增加 `summarize()` 方法，用 LLM 生成摘要替代截断

### 2.3 事件总线

- [ ] 引入 `tokio::sync::broadcast` 或 `flume` 替代零散 `mpsc`
- [ ] 定义统一 `AppEvent` enum

### 2.4 Session 异步化

- [ ] 将 `SessionStore` 的阻塞调用移至 `spawn_blocking`
- [ ] 或引入 `rusqlite::async`（如可用）

---

## Phase 3: DRY 极致精简

**时间估计**: 1-2 周  
**风险等级**: 低  
**验收标准**: 代码行数减少 15%，圈复杂度下降

### 3.1 字符串处理管线

- [ ] 实现 `TextSanitizer` pipeline
- [ ] 合并 `clean_display_text` 和 `clean_terminal_text`
- [ ] 用 `aho-corasick` 或 `lru` cache 优化关键词匹配

### 3.2 配置重载统一

- [ ] 合并 `watch_loop` 和 `handle_sighup` 为单一 `reload_task`

### 3.3 Diff 渲染统一

- [ ] 提取 `render_hunk_header`、`guess_extension`
- [ ] 统一 color mapping 到 `theme::diff_colors()`

### 3.4 构造器简化

- [ ] 删除 `App::new()`，保留 `App::new_with_settings`
- [ ] 删除 `App::new_with_channel`，将 channel 创建内联到 `new_with_settings`

---

## Phase 4: TUI 视觉与交互升级

**时间估计**: 2-3 周  
**风险等级**: 中  
**验收标准**: 视觉 Review 通过，交互流畅度提升

### 4.1 设计系统

- [ ] 引入 `tui::theme::DesignToken`
- [ ] 所有 widget 强制使用 theme，禁止硬编码 RGB
- [ ] 实现完整 Light/Dark 双主题

### 4.2 布局增强

- [ ] 响应式侧边栏宽度
- [ ] Diff 面板动态高度
- [ ] 面板拖拽调整大小（可选）

### 4.3 交互优化

- [ ] 实现 `InputHistory`
- [ ] 实现 `SearchHighlighter`
- [ ] Slash 菜单模糊匹配
- [ ] 进度条组件

### 4.4 动画系统

- [ ] 统一 `AnimationController`
- [ ] 光标/思考/加载共用时间轴
- [ ] 欢迎屏渐入动画

---

## 五、详细实施计划（Phase 1 展开）

### Week 1: 死代码清理 + 命名规范

#### Day 1-2: 死代码清理

**步骤**:
1. 运行 `cargo clippy` 收集所有 `dead_code` 警告
2. 逐文件移除 `#[allow(dead_code)]`
3. 对于真正需要的代码，添加 `#[cfg(test)]` 或集成到主流程
4. 对于未完成功能，创建 GitHub Issue 跟踪

**涉及文件**:
- `crates/synerix/src/error.rs`
- `crates/synerix/src/app/state/app.rs`
- `crates/synerix/src/app/state/diff.rs`
- `crates/synerix/src/tools/`
- `crates/synerix/src/llm/`
- `crates/synerix/src/mcp/`
- `crates/synerix/src/sandbox/`
- `crates/synerix/src/agent/`
- `crates/synerix/src/skills/`
- `crates/synerix/src/project/`

#### Day 3-4: 命名规范

**步骤**:
1. 重命名 `slash_menu.rs` → `slash_menu_state.rs`
2. 统一 `clean_*` → `sanitize_*`
3. 统一 `StatusBarState` 字段类型

#### Day 5: 工具函数抽取

**步骤**:
1. 创建 `app::cursor` 模块
2. 创建 `app::scroll` 模块
3. 创建 `tui::animation` 模块
4. 创建 `tui::text` 模块

### Week 2: 统一字符串清洗 + 动画

#### Day 6-7: 字符串清洗管线

**步骤**:
1. 分析 `clean_display_text` 和 `clean_terminal_text` 的异同
2. 设计统一的 `TextSanitizer` pipeline
3. 迁移所有调用点

#### Day 8-10: 动画系统 + 验证

**步骤**:
1. 实现 `tui::animation::AnimationController`
2. 迁移所有 `animated_dots` 调用
3. 运行完整测试套件
4. 运行 `cargo clippy` + `cargo fmt`

---

## 六、验收检查清单

- [ ] `cargo clippy --all-targets` 零警告
- [ ] `cargo test --workspace` 全绿
- [ ] `cargo fmt --check` 通过
- [ ] `cargo doc --no-deps` 无警告
- [ ] 代码行数减少 >= 10%
- [ ] `App` 结构体字段数 <= 20
- [ ] 零个 `#[allow(dead_code)]` 模块级标记
- [ ] 零个硬编码 RGB（除主题定义外）
