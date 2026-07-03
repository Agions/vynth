<div align="center">

<!-- Logo -->
<img src="assets/logo.svg" alt="Synerix Logo" width="180" style="margin-bottom: 1.5rem;">

<!-- Title -->
<h1 style="font-size: 3.5rem; font-weight: 800; letter-spacing: -0.04em; margin: 0.5rem 0;">
  <span style="background: linear-gradient(135deg, #7DCFFF, #BB9AF7); -webkit-background-clip: text; -webkit-text-fill-color: transparent;">Synerix</span>
</h1>

<p style="font-size: 1.35rem; color: #9AA5CE; max-width: 640px; margin: 0.75rem auto 1.5rem; line-height: 1.6;">
  <strong style="color: #C0CAF5;">AI-Native Coding Terminal</strong> — Think, write, review, and fix code without leaving the terminal.
  <br>
  <strong style="color: #C0CAF5;">AI 原生编程终端</strong> —— 思考、编写、审查和修复代码，无需离开终端。
</p>

<!-- Badges -->
<p>
  <a href="https://github.com/Agions/synerix/actions/workflows/ci.yml">
    <img src="https://img.shields.io/github/actions/workflow/status/Agions/synerix/ci.yml?branch=main&style=flat-square&label=CI&color=7DCFFF" alt="CI">
  </a>
  <a href="LICENSE">
    <img src="https://img.shields.io/badge/License-MIT-00D4FF.svg?style=flat-square" alt="License">
  </a>
  <a href="https://www.rust-lang.org">
    <img src="https://img.shields.io/badge/Rust-1.75%2B-FF6B3D.svg?style=flat-square&logo=rust&logoColor=white" alt="Rust">
  </a>
  <a href="https://github.com/Agions/synerix">
    <img src="https://img.shields.io/github/stars/Agions/synerix?style=flat-square&label=Stars" alt="Stars">
  </a>
  <a href="https://gitee.com/Agions/synerix">
    <img src="https://img.shields.io/badge/Gitee-Agions%2Fsynerix-C71D23.svg?style=flat-square&logo=gitee" alt="Gitee">
  </a>
</p>

<p>
  <a href="https://github.com/Agions/synerix">
    <img src="https://img.shields.io/badge/Binary-2.8_MB-7B61FF.svg?style=flat-square" alt="Size">
  </a>
  <a href="https://github.com/Agions/synerix">
    <img src="https://img.shields.io/badge/Startup-&lt;50ms-00FF88.svg?style=flat-square" alt="Startup">
  </a>
  <a href="https://github.com/Agions/synerix">
    <img src="https://img.shields.io/badge/Safety-100%25-FF6B9D.svg?style=flat-square" alt="Safety">
  </a>
  <a href="https://github.com/Agions/synerix">
    <img src="https://img.shields.io/badge/Tests-1342-00D4FF.svg?style=flat-square" alt="Tests">
  </a>
</p>

</div>

---

## What is Synerix? / Synerix 是什么？

**Synerix** is a high-performance, AI-native coding terminal built in Rust that brings intelligent code assistance directly into your CLI workflow. It combines the power of modern LLMs with a security-first sandbox, multiple coding modes, and a plugin ecosystem — all in a beautiful, responsive TUI.

**Synerix** 是一款用 Rust 构建的高性能 AI 原生编程终端，将智能代码辅助直接带入你的 CLI 工作流。它将现代 LLM 的能力与安全优先的沙箱、多种编程模式和插件生态系统相结合 —— 全部集成在一个美观、响应迅速的 TUI 中。

Instead of juggling editors, browsers, and terminals, Synerix keeps you in flow: **type → generate → compile → test → fix**, all in one place.

无需在编辑器、浏览器和终端之间来回切换，Synerix 让你保持心流：**打字 → 生成 → 编译 → 测试 → 修复**，所有操作都在一个地方完成。

### Two Coding Modes / 两种编程模式

| Mode / 模式 | Badge / 图标 | Description / 描述 |
|------|-------|-------------|
| **Plan** / 规划 | `📋` | Analyze first, propose a plan, then execute step-by-step with approval. / 先分析，提出方案，再逐步骤审批执行。 |
| **Vibe** / 沉浸 | `🎵` | Immersive AI coding — describe, generate, compile, test, and fix in a fully automated loop. / 沉浸式 AI 编程 —— 描述、生成、编译、测试和修复，全自动化循环。 |

---

## Why Synerix? / 为什么选择 Synerix？

| The Problem / 问题 | Synerix's Answer / Synerix 的答案 |
|-------------|------------------|
| Switch between 6 tools constantly / 在 6 个工具之间不断切换 | **One terminal** — type, generate, review, run, fix / **一个终端** —— 打字、生成、审查、运行、修复 |
| Manual approval for every operation / 每个操作都需手动批准 | **Smart sandbox** — auto-approves safe ops, previews risks / **智能沙箱** —— 自动批准安全操作，预览风险 |
| Paste code back and forth / 来回粘贴代码 | **Auto-iteration** — `described → generated → compiled → tested → fixed` / **自动迭代** —— `描述 → 生成 → 编译 → 测试 → 修复` |
| Context switching kills flow / 上下文切换打断心流 | **Persistent sessions** — SQLite-backed conversation history / **持久化会话** —— SQLite 支持的对话历史 |
| Bloated IDE with slow startup / 臃肿 IDE 启动缓慢 | **2.8 MB binary, <50ms startup** — native Rust performance / **2.8 MB 二进制文件，<50ms 启动** —— 原生 Rust 性能 |

---

## Quick Start / 快速开始

```bash
# macOS / Linux
curl -fsSL https://raw.githubusercontent.com/Agions/synerix/main/install.sh | bash

# Windows (PowerShell)
irm https://raw.githubusercontent.com/Agions/synerix/main/install.ps1 | iex
```

> **From source / 从源码构建:** `git clone && cargo build --release && cargo install --path .`

[Read the full guide → / 阅读完整指南 →](https://github.com/Agions/synerix/blob/main/docs/guide/getting-started.md)

---

## Capabilities / 核心能力

| Feature / 特性 | Details / 详情 |
|---------|---------|
| ⚡ **Blazing Performance** / 极速性能 | Rust binary, 2.8 MB, <50ms startup, 1342+ tests / Rust 二进制文件，2.8 MB，<50ms 启动，1342+ 测试 |
| 🛡 **Security Sandbox** / 安全沙箱 | Atomic writes, risk classifier, MCP isolation, approval flows / 原子写入、风险分类器、MCP 隔离、审批流程 |
| 🧩 **Plugin System** / 插件系统 | Commands, skills, MCP servers, custom agents / 命令、技能、MCP 服务器、自定义代理 |
| 🎨 **Tokyo Night TUI** | Rounded borders, syntax-highlighted diffs, 35+ color themes / 圆角边框、语法高亮差异、35+ 色彩主题 |
| 🔑 **Keymaps** / 按键映射 | Vim, Emacs, and Default layouts / Vim、Emacs 和默认布局 |
| 💡 **Smart Defaults** / 智能默认 | DeepSeek powered out of the box, auto-detects project context / 开箱即用 DeepSeek，自动检测项目上下文 |
| 🔄 **Session Persistence** / 会话持久化 | SQLite-backed conversations, resume anywhere / SQLite 支持的对话， anywhere 恢复 |
| 🎯 **Goal-Driven Loops** / 目标驱动循环 | `/goal` auto-iteration until conditions are met / `/goal` 自动迭代直到条件满足 |

---

## Architecture / 架构

```
synerix/
├── src/
│   ├── main.rs                 # Startup + binary entry / 启动 + 二进制入口
│   ├── app/                    # State machine + event dispatch / 状态机 + 事件分发
│   │   ├── state/              # App state (bitflags, controllers) / 应用状态（位标志、控制器）
│   │   ├── event_loop.rs       # Main event loop (tokio::select!) / 主事件循环
│   │   ├── actions.rs          # Action dispatch / 动作分发
│   │   └── input_handler.rs    # Mode-specific input / 模式特定输入
│   ├── tui/                    # TUI rendering (ratatui) / TUI 渲染
│   │   ├── widgets/            # 7 composable widgets / 7 个可组合组件
│   │   ├── theme.rs            # Theme manager (runtime hot-swap) / 主题管理器（运行时热切换）
│   │   ├── layout.rs           # Responsive layout engine / 响应式布局引擎
│   │   └── renderer.rs         # Frame orchestration / 帧编排
│   ├── agent/                  # Agent loop + multi-agent swarm / 代理循环 + 多代理集群
│   ├── tools/                  # Tool registry + builtins / 工具注册表 + 内置工具
│   ├── skills/                 # Skills system / 技能系统
│   ├── mcp/                    # MCP protocol client / MCP 协议客户端
│   ├── sandbox/                # Security sandbox / 安全沙箱
│   ├── workflow/               # DAG workflow engine / DAG 工作流引擎
│   ├── llm/                    # LLM adapters + streaming / LLM 适配器 + 流式
│   └── session/                # SQLite persistence / SQLite 持久化
├── docs/                       # VitePress documentation / VitePress 文档
└── tests/                      # 1342+ tests / 1342+ 测试
```

---

## Community / 社区

| | |
|---|---|
| 📖 **Docs** / **文档** | [github.com/Agions/synerix/tree/main/docs](https://github.com/Agions/synerix/tree/main/docs) |
| 🐛 **Issues** / **问题反馈** | [github.com/Agions/synerix/issues](https://github.com/Agions/synerix/issues) |
| 🥭 **Gitee Mirror** / **Gitee 镜像** | [gitee.com/Agions/synerix](https://gitee.com/Agions/synerix) |

---

## License / 许可证

MIT — free for personal and commercial use. / MIT —— 免费用于个人和商业用途。

<div align="center">
  <img src="assets/logo.svg" alt="Synerix" width="100">
  <p style="color: #565F89; font-size: 0.85rem; margin-top: 0.5rem;">
    Built with 🦀 Rust · Designed for developers who love the terminal / 用 🦀 Rust 构建 · 为热爱终端的开发者设计
  </p>
</div>
