# ADR-001: 技术栈选择 — Rust + Ratatui

- **状态**：Accepted
- **日期**：2026-01-15
- **作者**：Synerix 架构团队

## 背景

Synerix 需要构建一个 AI 原生编程终端，核心需求包括：

1. **低延迟**：LLM 流式响应需在 TUI 中实时渲染，首 Token 延迟目标 < 50ms
2. **内存安全**：终端直接操作本地文件系统、执行命令，安全防线必须从语言层面开始
3. **并发能力**：Agent 循环需同时处理 LLM 流式响应、工具执行、UI 渲染三条并发流水线
4. **跨平台**：支持 macOS 和 Linux 开发环境
5. **二进制体积**：作为终端工具，启动速度和磁盘占用直接影响体验

## 决策

选择 **Rust** 作为实现语言，**Ratatui** 作为 TUI 框架。

### Rust 的理由

1. **零成本抽象**：所有权模型在编译期消除数据竞争和悬挂引用，无需 GC 暂停
2. **异步优先**：tokio 提供了生产级的 async 运行时，天然适合 LLM 流式 + 工具执行的并发模型
3. **编译期保证**：`unsafe` 零使用率（当前代码库 0 个 unsafe 块），内存安全由编译器保障
4. **产物体积**：LTO=fat + panic=abort 编译产物仅 3.8MB，远低于 Electron 方案的 50MB+
5. **生态成熟**：crates.io 拥有 Ratatui（TUI）、tokio（异步）、serde（序列化）、clap（CLI）等高质量生态

### Ratatui 的理由

1. **即时模式渲染**：每一帧重建 UI 状态，与 React/Vue 的响应式模式相比，更适合终端中频繁变化的流式输出
2. **无运行时**：纯 Rust 库，无 GC、无解释器，启动时间 2ms
3. **Flex/Constraint 布局**：内置的 `Layout::default().constraints([...])` 声明式布局，比 curses 的手动坐标计算更可维护
4. **Buffer 级 diff**：帧渲染只将变更部分写入终端，避免全屏闪烁

## 后果

### 正面

- 启动速度 2ms（目标 < 80ms），远超同业水平
- 二进制 3.8MB，可嵌入 Docker 镜像或 CI 管道
- 编译阶段即发现内存安全问题，减少运行时 crash
- async/await 天然适配 LLM 流式 + 工具并发的场景

### 负面

- Rust 学习曲线陡峭，新贡献者上手成本较高
- 编译时间长（增量 2-10s，clean 3-8min）
- Ratatui 的即时模式渲染在超大文件高亮时可能出现性能瓶颈（需 syntect 异步化）

## 备选方案

| 方案 | 放弃原因 |
|------|----------|
| Python + Textual | 性能不足，流式渲染帧率低；GIL 限制多 Agent 并发 |
| Go + Bubble Tea | Go 泛型缺失导致 LLM 适配器代码重复；无零成本抽象 |
| TypeScript + Ink | Node.js 启动 100ms+；Electron 体积 50MB+；不适合终端原生体验 |
| C + ncurses | 无内存安全保障，unsafe 代码覆盖率高；生态陈旧 |