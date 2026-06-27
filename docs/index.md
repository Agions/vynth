---
layout: home

hero:
  name: Synerix
  text: AI-Native Coding Terminal
  tagline: Think, write, review, and fix code — without leaving the terminal.
  image:
    src: /banner.svg
    alt: Synerix
  actions:
    - theme: brand
      text: Get Started
      link: /guide/getting-started
    - theme: alt
      text: View on GitHub
      link: https://github.com/Agions/synerix

features:
  - icon: ⚡
    title: Act Mode
    details: Execute commands directly in the terminal with an intelligent sandbox that auto-approves low-risk operations.
  - icon: 🎵
    title: Vibe Mode
    details: Immersive AI coding experience — describe, generate, compile, test, and fix in one seamless loop.
  - icon: 🗣
    title: Chat Mode
    details: Have a conversation with AI to get code suggestions, explanations, and debugging help.
  - icon: 📐
    title: Architect Mode
    details: High-level design and code reviews to help you build better software architecture.
  - icon: 📋
    title: Plan Mode
    details: Break down complex problems into executable, prioritized steps.
  - icon: 🛡
    title: Security Sandbox
    details: Atomic writes, risk-aware approvals, and per-tool permission boundaries.
---

## Why Synerix?

AI coding assistants are powerful — but **workflow friction** kills flow:

| The Problem | Synerix's Answer |
|---|---|
| Switch between editor, browser, and terminal 50× an hour | **One terminal** — type, generate, review, run, and fix in one place. |
| Approve every file write, command, and keystroke manually | **Smart sandbox** — auto-approves safe ops in Vibe mode, full preview for risky ones. |
| Paste code into chat, copy back, compile, paste errors back | **Auto-iteration** — `described → generated → compiled → tested → fixed` in a single loop. |

Synerix collapses your AI coding workflow from **6 tools × 3 contexts** into **1 terminal window**.

---

## Quick Start

```bash
# Linux / macOS
curl -fsSL https://raw.githubusercontent.com/Agions/synerix/main/install.sh | bash

# Windows (PowerShell 5.1+)
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
irm https://raw.githubusercontent.com/Agions/synerix/main/install.ps1 | iex
```

See the [Getting Started](/guide/getting-started) guide for detailed setup instructions.
