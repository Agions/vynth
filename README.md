<div align="center">

  <img src="./assets/banner.svg" alt="Synerix — AI-Native Coding Terminal" style="max-width:100%">

  <p>
    <strong>Synerix</strong> is an AI-native coding terminal that <strong>thinks, writes, reviews, and fixes code</strong> — without pulling you out of the command line.
  </p>

  <p>
    <a href="https://github.com/Agions/synerix/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/Agions/synerix/ci.yml?branch=main&style=flat-square&label=CI" alt="CI"></a>
    <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-00D4FF.svg?style=flat-square" alt="License"></a>
    <a href="https://www.rust-lang.org"><img src="https://img.shields.io/badge/Rust-1.75%2B-FF6B3D.svg?style=flat-square&logo=rust&logoColor=white" alt="Rust"></a>
    <a href="https://github.com/Agions/synerix"><img src="https://img.shields.io/badge/GitHub-Agions%2Fsynerix-181717.svg?style=flat-square&logo=github" alt="GitHub"></a>
    <a href="https://gitee.com/Agions/synerix"><img src="https://img.shields.io/badge/Gitee-Agions%2Fsynerix-C71D23.svg?style=flat-square&logo=gitee" alt="Gitee"></a>
  </p>

  <p>
    <a href="#"><img src="https://img.shields.io/badge/Tests-1342-00FF88.svg?style=flat-square&logo=rust" alt="Tests"></a>
    <a href="#"><img src="https://img.shields.io/badge/Binary-3.8_MB-7B61FF.svg?style=flat-square" alt="Size"></a>
    <a href="#"><img src="https://img.shields.io/badge/Unsafe_Code-0-FF4444.svg?style=flat-square" alt="Unsafe"></a>
  </p>

</div>

---

## What is Synerix?

Synerix is a **Rust-native, AI-powered terminal** designed for developers who want to stay in flow.

Instead of jumping between your editor, browser, and terminal, Synerix brings AI-assisted coding directly into your CLI — with a security sandbox, multiple coding modes, and a plugin system that grows with you.

### Five modes. One terminal.

| Mode | Icon | Description |
|---|---|---|
| **Act** | ⚡ | Execute commands with an intelligent sandbox. Low-risk ops auto-approve; dangerous ops get a full preview. |
| **Vibe** | 🎵 | Describe, generate, compile, test, and fix — fully automated loops for rapid prototyping. |
| **Chat** | 🗣 | Converse with AI about your code. Get explanations, suggestions, and debugging help. |
| **Architect** | 📐 | High-level design reviews, architecture feedback, and refactoring guidance. |
| **Plan** | 📋 | Break down complex features into executable, prioritized plans. |

---

## Why Synerix?

| The Problem | Synerix |
|---|---|
| Switch between 6 tools constantly | **One terminal** — type, generate, review, run, fix |
| Manual approval for every operation | **Smart sandbox** — auto-approves safe ops, previews risks |
| Paste code back and forth | **Auto-iteration** — `described → generated → compiled → tested → fixed` |

---

## Quick Start

```bash
# macOS / Linux
curl -fsSL https://raw.githubusercontent.com/Agions/synerix/main/install.sh | bash

# Windows (PowerShell)
irm https://raw.githubusercontent.com/Agions/synerix/main/install.ps1 | iex
```

[Read the full guide →](https://github.com/Agions/synerix/blob/main/docs/guide/getting-started.md)

---

## Capabilities

| Feature | Implementation |
|---|---|
| ⚡ **Snappy performance** | Rust binary, <4 MB, 2ms startup |
| 🛡 **Security sandbox** | Atomic writes, risk classifier, MCP isolation |
| 🧩 **Plugin system** | Commands, skills, and MCP servers |
| 🎨 **Tokyo Night TUI** | Rounded borders, syntax-highlighted diffs, 35+ themes |
| 🔑 **Keymaps** | Vim, Emacs, and Default layouts |
| 💡 **Smart defaults** | DeepSeek powered out of the box |

---

## Architecture

```
synerix/
├── src/
│   ├── main.rs              # Startup + binary entry
│   ├── app/                 # State machine + event dispatch
│   ├── tui/                 # TUI rendering (ratatui)
│   │   └── widgets/         # 7 composable widgets
│   ├── agent/               # Agent loop + multi-agent swarm
│   ├── tools/               # Tool registry + builtins
│   ├── skills/              # Skills system
│   ├── mcp/                 # MCP protocol client
│   ├── sandbox/             # Security sandbox
│   ├── workflow/            # DAG workflow engine
│   └── session/             # SQLite persistence
├── docs/                    # VitePress documentation
└── tests/                   # 1300+ tests
```

---

## Community

| | |
|---|---|
| 📖 **Docs** | [github.com/Agions/synerix/tree/main/docs](https://github.com/Agions/synerix/tree/main/docs) |
| 🐛 **Issues** | [github.com/Agions/synerix/issues](https://github.com/Agions/synerix/issues) |
| 🥭 **Gitee Mirror** | [gitee.com/Agions/synerix](https://gitee.com/Agions/synerix) |

---

## License

MIT — free for personal and commercial use.

<img src="./assets/logo.svg" alt="Synerix" style="height:64px">
