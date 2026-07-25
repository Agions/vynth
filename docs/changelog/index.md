# 变更日志

所有 Vynth 的 notable changes 都记录在此文件中。格式遵循 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)，版本号遵循 [Semantic Versioning](https://semver.org/lang/zh-CN/)。

---

## 版本列表

| 版本 | 日期 | 说明 |
|------|------|------|
| [v0.2.1](v0.2.1.md) | 2026-07-25 | 错误码 6 位落地 + demo 下线 + 模型名回滚 |
| [v0.2.0](v0.2.0.md) | 2026-07-25 | MVP 完整闭环上线 |
| [v0.1.0](v0.1.0.md) | 2026-07-25 | 初始架构交付 |

---

## 版本说明

### v0.2.1（当前版本）

**Patch 发布** —— 错误码 6 位体系 + demo 模式下线 + 默认模型回滚。

**新增：**
- 错误码 6 位码（VC-XXXXXX）权威表 + 单测（22 个已声明码）
- `VynthError.numericCode` 字段，旧 `code` 保留兼容

**变更：**
- 默认模型回滚为 `deepseek-v4-pro`（与冻结裁决 X1/X2 一致）
- 所有包版本号 0.2.0 → 0.2.1

**Breaking：**
- 移除 `EchoProvider` / demo 模式；`VYNTH_API_KEY` 为必填项

### v0.2.0

**MVP 完整闭环上线** —— 包含 Sprint 1-6 全部功能。

**新增功能：**
- Agent 引擎 + LLM（F4 / F6 / F7 / F8）
- 内置工具 + 沙箱守卫（F5 / F10）
- 插件无头接入（F9）
- TUI 双模式契约（F2 / F3）
- 工程规矩闸门（CI 8 阶段）
- 冷启动基线测量（P95 = 30.5ms）
- CLI 退出码契约（F11）

**变更：**
- 默认模型修正为 `deepseek-v4-pro`
- 默认端点 `https://api.deepseek.com/v1`
- 体积优化至 60.51MB

**安全：**
- 沙箱守卫覆盖路径越界、symlink 逃逸
- `VYNTH_NET` 联网开关
- gitleaks 密钥扫描入 CI

### v0.1.0

**初始架构交付** —— 项目初始化、基础架构搭建。

---

## 贡献

在 PR 中更新 `CHANGELOG.md`（根目录）以记录你的变更。

---

## 相关文档

- [快速开始](../guide/getting-started.md)
- [架构总览](../architecture/index.md)
- [开发规范](../development/dev-guide.md)
