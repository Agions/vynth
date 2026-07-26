# Vynth

![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)
![Bun](https://img.shields.io/badge/Bun-%E2%89%A51.1-orange)
![TypeScript](https://img.shields.io/badge/TypeScript-5.x-blue)
![Platform](https://img.shields.io/badge/Platform-macOS%20%7C%20Linux%20%7C%20Windows-lightgrey)

> **你 terminal 里的代码合成器** —— 本地优先的单二进制 TUI AI 编程工具，多智能体协作 + 安全沙箱 + 插件可扩展。

Vynth 是一个 AI-Native Coding Terminal，支持 **Plan**（先规划再动手）与 **Vibe**（边聊边写）双模式，把自然语言目标「合成」为代码改动。纯 TypeScript 全量构建，通过 `bun build --compile` 打包为单个 `dist/vynth` 二进制，开箱即用。

---

## 核心功能

| 功能             | 说明                                                    | 状态      |
| ---------------- | ------------------------------------------------------- | --------- |
| **单二进制启动** | Bun compile 打包，无 node_modules，无外部 wasm          | ✅ MVP    |
| **双模式 TUI**   | 交互式 ANSI 终端 / 无头 agent 流式输出                  | ✅ MVP    |
| **Agent 循环**   | 流式 token + tool_calls + 回填，默认 maxSteps=8         | ✅ MVP    |
| **内置工具**     | read_file / write_file / run_shell，经 sandbox 越界守卫 | ✅ MVP    |
| **LLM 默认对齐** | DeepSeek 端点 + `deepseek-v4-pro`，OpenAI 兼容 SSE      | ✅ MVP    |
| **插件无头接入** | `-p/--plugin` 动态加载，工具注册表热扩展                | ✅ MVP    |
| **沙箱守卫**     | safeResolve 路径越界拦截 + VYNTH_NET 联网开关           | ✅ MVP    |
| **MCP CLI 接入** | `-s/--mcp` 接入 stdio JSON-RPC 2024-11-05 server，工具并入 agent 工具集 | ✅ v0.1.0 |
| **TUI 内插件**   | 交互界面内插件加载与渲染                                | ⏳ 完整版 |
| **OS 级硬隔离**  | bubblewrap/seatbelt 进程级沙箱                          | ⏳ 完整版 |

---

## 快速上手

```bash
# 1. 克隆仓库
git clone git@github.com:Agions/vynth.git && cd vynth

# 2. 安装依赖
bun install

# 3. 编译单二进制
bun run compile

# 4. 设置 API Key 并运行
export VYNTH_API_KEY="sk-..."
./dist/vynth -g '给当前目录写一份 README.md'

# 5. 启动交互 TUI（需真实终端）
./dist/vynth
```

> **实测基线**：单二进制 **60.52 MB**；冷启动 **P95 = 30.5 ms**（远低于 150 ms 基线）。

---

## 安装

### 前置要求

- **Bun >= 1.1**（运行时 + 打包）
- **Node.js >= 18**（仅用于 `biome` / `turbo` 等辅助工具，非运行时依赖）
- **Git**（克隆仓库）

### 从源码编译

```bash
git clone git@github.com:Agions/vynth.git
cd vynth
bun install
bun run compile
```

### 直接使用二进制

```bash
# 下载预编译二进制（即将发布）
# 或通过 bun link 全局安装
bun link
```

---

## 配置

Vynth **仅通过环境变量** 配置，不读取任何配置文件。

| 变量                 | 作用            | 默认值                        | 必填   |
| -------------------- | --------------- | ----------------------------- | ------ |
| `VYNTH_API_KEY`      | LLM API Key     | 空                            | **是** |
| `VYNTH_MODEL`        | 模型名          | `deepseek-v4-pro`             | 否     |
| `VYNTH_LLM_BASE_URL` | OpenAI 兼容端点 | `https://api.deepseek.com/v1` | 否     |
| `VYNTH_MODE`         | 运行模式        | `vibe`                        | 否     |
| `VYNTH_THEME`        | TUI 主题        | `mocha`                       | 否     |
| `VYNTH_NET`          | 联网开关        | `开启`                        | 否     |
| `VYNTH_DATA_DIR`     | 数据目录        | `~/.vynth`                    | 否     |

```bash
# 示例：接入 DeepSeek + 禁用联网
export VYNTH_API_KEY="sk-..."
export VYNTH_MODEL="deepseek-v4-pro"
export VYNTH_NET="0"
./dist/vynth -g '重构 utils 目录下的工具函数'
```

---

## 使用示例

### 无头模式（CI / 管道 / 脚本）

```bash
# 直接输出到 stdout，适合管道
./dist/vynth -g '给 src/core 写单元测试' > output.md
```

### 交互 TUI

```bash
# 启动全屏 TUI（需 TTY）
./dist/vynth

# 指定模式
./dist/vynth -m plan '设计用户认证模块'
```

### 加载插件

```bash
# 加载本地插件
./dist/vynth -g '使用自定义工具' -p packages/plugins/examples/hello-plugin.ts
```

### 接入 MCP server

```bash
# 接入一个 stdio MCP server（可重复 -s 接入多个）
./dist/vynth -g '用 MCP 工具完成任务' -s "bun run packages/mcp/examples/echo-server.ts"
# 生产环境常见写法（任意 stdio 命令均可）：
./dist/vynth -g '查询天气' -s "npx -y @modelcontextprotocol/server-xxx"
```

> MCP server 以子进程启动（stdio JSON-RPC，协议版本锁定 **2024-11-05**），其工具会被自动转换为 agent 工具集并参与同一套沙箱 / 审计链路。

---

## 目录结构

```
vyntoh/
├── apps/
│   └── cli/                    # CLI 入口（bin: vynth）
├── packages/
│   ├── core/                   # 共享类型 / 配置 / 错误 / 事件总线 / 日志
│   ├── engine/                 # LLM 客户端 + 工具系统 + agent 循环
│   ├── tui/                    # 轻量 ANSI 渲染器 + 主题 + 逃生舱
│   ├── sandbox/                # fs 越界守卫 + 命令执行 + 网络开关
│   ├── mcp/                    # MCP 客户端（stdio JSON-RPC）
│   ├── plugins/                # 插件加载 / 生命周期
│   └── harness/                # 集成测试 / e2e 驱动
├── docs/                       # 项目文档
├── delivery/                   # 架构交付物归档
├── dist/                       # 编译输出
├── scripts/                    # 开发 / 基准测试脚本
├── package.json
├── pnpm-workspace.yaml
├── turbo.json
└── biome.json
```

---

## 贡献指南

欢迎贡献！请阅读 [贡献指南](docs/development/contributing.md) 了解：

- 开发环境搭建
- 分支模型与 Conventional Commits
- 测试策略（`bun test`）
- CI 流水线与质量闸门
- 安全红线（密钥扫描、体积门禁）

---

## 开源许可证

[MIT](./LICENSE) © 2026 Agions

---

## 文档导航

| 文档                                          | 面向人群          | 说明                              |
| --------------------------------------------- | ----------------- | --------------------------------- |
| [快速开始](docs/guide/getting-started.md)     | 新用户            | 安装、编译、真实链路、插件       |
| [配置详解](docs/guide/configuration.md)       | 新用户 / 高级用户 | 环境变量完整说明与最佳实践        |
| [插件开发](docs/guide/plugins.md)             | 插件开发者        | 插件 manifest、生命周期、工具注册 |
| [架构总览](docs/architecture/index.md)        | 架构师 / 贡献者   | 模块关系、数据流、关键不变量      |
| [Package 职责](docs/architecture/packages.md) | 贡献者            | 各 `@vynth/*` 包职责与关键文件    |
| [API 参考](docs/api/overview.md)              | 用户 / 开发者     | CLI 参数、环境变量、退出码        |
| [开发规范](docs/development/dev-guide.md)     | 贡献者            | 分支模型、代码规范、安全红线      |
| [贡献指南](docs/development/contributing.md)  | 贡献者            | 提交流程、PR 规范、评审标准       |
| [测试指南](docs/development/testing.md)       | 贡献者            | 测试策略、基准测试、覆盖率        |
| [FAQ](docs/faq/index.md)                      | 所有用户          | 常见问题与故障排查                |
| [变更日志](docs/changelog/v0.1.0.md)          | 所有用户          | 版本历史与迁移指南                |

---

## 阅读路径建议

### 🟢 新手入门（15 分钟）

1. [README 核心功能](#核心功能)
2. [快速上手](#快速上手)
3. [使用示例](#使用示例)

### 🟡 进阶使用（30 分钟）

1. [配置详解](docs/guide/configuration.md)
2. [插件开发](docs/guide/plugins.md)
3. [FAQ](docs/faq/index.md)

### 🔴 深度参与（2 小时+）

1. [架构总览](docs/architecture/index.md)
2. [开发规范](docs/development/dev-guide.md)
3. [贡献指南](docs/development/contributing.md)
4. [测试指南](docs/development/testing.md)
