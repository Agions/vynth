---
layout: home

hero:
  name: Synerix
  text: AI-Native Coding Terminal
  tagline: Think, write, review, and fix code — without leaving the terminal. / 思考、编写、审查和修复代码 —— 无需离开终端。
  image:
    src: /banner.svg
    alt: Synerix
  actions:
    - theme: brand
      text: Get Started
      link: /guide/getting-started
    - theme: alt
      text: 快速开始
      link: /guide/getting-started
    - theme: alt
      text: View on GitHub
      link: https://github.com/Agions/synerix

features:
  - icon: 📋
    title: Plan Mode / 规划模式
    details: Analyze first, propose a plan, then execute step-by-step with approval. / 先分析，提出方案，再逐步骤审批执行。
  - icon: 🎵
    title: Vibe Mode / 沉浸模式
    details: Immersive AI coding — describe, generate, compile, test, and fix in one seamless loop. / 沉浸式 AI 编程 —— 描述、生成、编译、测试、修复，一气呵成。
  - icon: 🛡
    title: Security Sandbox / 安全沙箱
    details: Atomic writes, risk-aware approvals, and per-tool permission boundaries. / 原子写入、风险感知审批和逐工具权限边界。
---

## Why Synerix? / 为什么选择 Synerix？

AI coding assistants are powerful — but **workflow friction** kills flow:

AI 编程助手很强大 —— 但**工作流摩擦**会打断心流：

| The Problem / 问题 | Synerix's Answer / Synerix 的答案 |
|---|---|
| Switch between editor, browser, and terminal 50× an hour / 每小时在编辑器、浏览器和终端之间切换 50 次 | **One terminal** — type, generate, review, run, and fix in one place. / **一个终端** —— 打字、生成、审查、运行和修复都在一个地方完成。 |
| Approve every file write, command, and keystroke manually / 手动批准每个文件写入、命令和按键 | **Smart sandbox** — auto-approves safe ops in Vibe mode, full preview for risky ones. / **智能沙箱** —— 在沉浸模式下自动批准安全操作，风险操作完整预览。 |
| Paste code into chat, copy back, compile, paste errors back / 把代码粘贴到聊天中，复制回来，编译，再把错误粘贴回去 | **Auto-iteration** — `described → generated → compiled → tested → fixed` in a single loop. / **自动迭代** —— `描述 → 生成 → 编译 → 测试 → 修复` 在一个循环中完成。 |
| Bloated IDEs with slow startup / 臃肿的 IDE 启动缓慢 | **2.8 MB binary, <50ms startup** — native Rust performance / **2.8 MB 二进制文件，<50ms 启动** —— 原生 Rust 性能 |
| Context switching kills flow / 上下文切换打断心流 | **Persistent sessions** — SQLite-backed conversation history / **持久化会话** —— SQLite 支持的对话历史 |

Synerix collapses your AI coding workflow from **6 tools × 3 contexts** into **1 terminal window**.

Synerix 将你的 AI 编程工作流从 **6 个工具 × 3 个上下文** 压缩为 **1 个终端窗口**。

---

## Quick Start / 快速开始

```bash
# Linux / macOS
curl -fsSL https://raw.githubusercontent.com/Agions/synerix/main/install.sh | bash

# Windows (PowerShell 5.1+)
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
irm https://raw.githubusercontent.com/Agions/synerix/main/install.ps1 | iex
```

See the [Getting Started](/guide/getting-started) guide for detailed setup instructions.

查看 [快速开始](/guide/getting-started) 指南了解详细设置说明。

---

## Performance / 性能

| Metric / 指标 | Value / 值 |
|--------|-------|
| Binary Size / 二进制大小 | 2.8 MB |
| Startup Time / 启动时间 | <50ms |
| Memory Usage / 内存占用 | ~15 MB idle |
| Test Coverage / 测试覆盖 | 1342+ tests |
| Build Time / 构建时间 | ~30s (release) |
| Unsafe Code / 非安全代码 | 0% |
