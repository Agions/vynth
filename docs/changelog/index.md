# 变更日志

所有 Vynth 的 notable changes 都记录在此文件中。格式遵循 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)，版本号遵循 [Semantic Versioning](https://semver.org/lang/zh-CN/)。

---

## 版本列表

| 版本 | 日期 | 说明 |
|------|------|------|
| [v0.1.0](v0.1.0.md) | 2026-07-25 | MVP 完整闭环 + 错误码 6 位化 + demo 移除 + DeepSeek V4 thinking |

> **统一发布**：本次发布合并 v0.1.0（初版骨架）/ v0.2.0（MVP 闭环）/ v0.2.1（错误码 6 位化 + demo 移除 + 模型回滚）三段历史到单一 `v0.1.0` 版本。

---

## 版本说明

### v0.1.0（当前版本）

**统一发布** —— MVP 完整闭环 + 错误码 6 位化 + demo 模式移除 + DeepSeek V4 thinking 支持。

**新增：**
- 错误码 6 位码（VC-XXXXXX）权威表 + 单测（22 个已声明码 + 6 个族默认回退码）
- `VynthError.numericCode` 字段，旧 `code` 保留兼容（`fromLegacy()` 桥接）
- DeepSeek V4 thinking 模式支持：`reasoning_content` / `tool_calls` / `tool_call_id` 字段
- Agent 引擎 + LLM（F4 / F6 / F7 / F8）
- 内置工具 + 沙箱守卫（F5 / F10，含 symlink 二次校验）
- 插件无头接入（F9）
- TUI 双模式契约（F2 / F3）+ Catppuccin mocha/latte 主题
- 工程规矩闸门（CI 8 阶段含 gitleaks / 体积门禁）
- 冷启动基线测量（P95 ≤ 150ms）
- CLI 退出码契约（F11，含 6 位码前缀）
- MIT License + 完整文档体系 redesign
- 架构交付归档 `delivery/` + 实施开发计划

**变更：**
- 默认模型 `deepseek-v4-pro`（与冻结裁决 X1/X2 一致）
- 默认端点 `https://api.deepseek.com/v1`
- 体积实测 60.51 MB（门禁 61 MB）
- 文档结构：`docs/dev-guide.md` → `docs/development/dev-guide.md`；`docs/release-notes-v0.*.md` → `docs/changelog/v0.*.md`

**Breaking：**
- 移除 `EchoProvider` / demo 模式；`VYNTH_API_KEY` 为必填项（空时抛 `[VC-020099]`）

**安全：**
- 错误字符串前缀化（`[VC-XXXXXX] message`），便于日志聚合 / 监控告警
- 沙箱守卫覆盖路径越界、绝对路径越界、symlink 逃逸（F10 对抗 X3）
- `VYNTH_NET='0'` 联网开关
- `OpenAiProvider` 拒绝向远程明文 http 端点发 API Key
- gitleaks 密钥扫描入 CI 红线

**测试基线：**
- `bun test packages`: **69 pass / 0 fail**
- `bun run lint`: 0 error（59 文件）
- `bun run compile` + 体积门禁: **60.51 MB < 61 MB PASS**
- 冷启动 P95 ≤ 150 ms PASS

---

## 贡献

在 PR 中更新 `CHANGELOG.md`（根目录）以记录你的变更。

---

## 相关文档

- [快速开始](../guide/getting-started.md)
- [架构总览](../architecture/index.md)
- [开发规范](../development/dev-guide.md)