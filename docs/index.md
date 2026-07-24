# Vynth

> **你 terminal 里的代码合成器。**

AI-Native Coding Terminal。把自然语言目标「合成」成代码——支持 **Plan**（先规划再动手）与 **Vibe**（边聊边写）双模式，纯 TypeScript 全量构建为单个 Bun 二进制，开箱即用。

## 为什么是 Vynth

- **terminal 原生**：不抢你的编辑器，不抢你的上下文。在已有 shell 工作流里直接「说一句话」让 agent 读写文件、跑命令。
- **单二进制分发**：`bun build --compile` 产出 `dist/vynth`，无 node_modules、无外部 wasm 资源。
- **轻量 TUI**：自研 ANSI 渲染器（非 ink），流式直写逃生舱，逐字符不卡顿。
- **demo 即开**：不设 API key 也能体验流式与工具循环；接上 `VYNTH_API_KEY` 即接真实 LLM。
- **可扩展**：工具注册表 + 插件加载（已就绪）+ MCP 客户端（已就绪），agent 能力可长可短。

## 30 秒上手

```bash
bun install
bun run compile
./dist/vynth -g '用一句话介绍 vynth'     # 无需 key，demo 模式
export VYNTH_API_KEY="sk-..."            # 接入真实 LLM
./dist/vynth -g '给当前目录写一份 README.md'
```

详见 [快速开始](guide/getting-started.md)。

## 文档导航

| 文档 | 内容 |
|------|------|
| [快速开始](guide/getting-started.md) | 安装、编译、demo、真实链路、环境变量 |
| [架构总览](architecture/index.md) | 架构图、Package 地图、端到端数据流 |
| [Package 职责详解](architecture/packages.md) | 各 `@vynth/*` 包职责与关键文件 |
| [API 总览](api/overview.md) | CLI 参数、环境变量、退出码 |
| [ADR 0003：纯 TypeScript 全量构建](adr/0003-pure-typescript-build.md) | 架构决策、后果与回退条件 |

## 命令速览

```
vynth                 交互式 TUI（需 TTY）
vynth -g '<目标>'     无头 agent 流式
vynth -m plan         指定 plan|vibe 模式
vynth --version       版本号
vynth --help          用法说明
```

> 配置经环境变量注入：`VYNTH_API_KEY` / `VYNTH_MODEL` / `VYNTH_LLM_BASE_URL` / `VYNTH_MODE` / `VYNTH_THEME` / `VYNTH_NET` / `VYNTH_DATA_DIR`。
