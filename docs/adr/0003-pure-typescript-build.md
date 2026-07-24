# ADR 0003: 纯 TypeScript 全量构建

- **状态（Status）**：已采纳（Accepted）
- **提出日期**：2025-07
- **主题**：由「Rust 混合架构」翻转为「纯 TypeScript 全量构建（Single-binary）」

---

## 背景（Context）

Vynth 早期（原 `synerix`）在技术选型上倾向「Rust 混合架构」：用 Rust 承担性能敏感路径（解析、流式缓冲、部分 I/O），再用 TypeScript / 胶水层对外暴露编程体验。这一思路的出发点是用 Rust 拿到接近原生的启动速度与运行时可控性。

但在实际推进中，我们重新审视了项目的核心约束：

1. **生态成熟度**：终端 TUI、AI/LLM SDK、OpenAI 兼容 SSE 客户端、MCP 协议、插件体系在 TypeScript 生态里有直接可用的基础（Bun 的原生 `fetch`、流式 `ReadableStream`、`node:*` 子进程与 fs），而 Rust 侧需要重复实现或绑定，收益不及预期。
2. **招人成本**：纯 TS 团队招聘面远大于「Rust + TS 双栈」团队；Vynth 的核心竞争力在「编程体验 / agent 循环 / 工具系统」，不在底层运行时性能。
3. **迭代速度**：引入 Rust 意味着额外的工具链（rustc / cargo / 跨平台目标）、原生模块编译等待、以及 `bun build --compile` 对 wasm / 原生库的打包限制（典型受害者是 `ink` 依赖的 `yoga.wasm`，无法被 Bun 稳定打包成单二进制）。

关键转折点是：**TUI 渲染层不得不放弃 ink（yoga.wasm 无法被 bun 打包），改用自研轻量 ANSI 渲染器**。一旦渲染层已经去 ink 化、去 wasm 化，Rust 混合架构剩余的「性能护城河」进一步收窄，纯 TS 全量构建的可行性显著提升。

---

## 决策（Decision）

采用**纯 TypeScript 全量构建**，具体约定如下：

- **运行时**：Bun（`engines.bun >= 1.1`），生产分发使用 `bun build --compile` 产出**单个原生二进制**。
- **构建命令**：根 `package.json` 的 `compile` 脚本 = `bun build --compile --target=bun apps/cli/src/main.ts --outfile dist/vynth`。
- **包管理**：pnpm workspace + turbo（`pnpm-workspace.yaml` 声明 `packages/*` 与 `apps/*`；`turbo.json` 管理 `build`/`test`/`lint` 依赖图）。
- **包组织**：以 `@vynth/*` 作用域拆分：`@vynth/{core,engine,tui,sandbox,mcp,plugins,harness}` + 应用 `@vynth/cli`（bin: `vynth`）。
- **TUI 渲染**：使用**轻量 ANSI 渲染器**（raw mode + `readline` + `StreamArea` 直写逃生舱），**明确不使用 ink**——避免 `yoga.wasm` 打包进单二进制失败。
- **配置加载**：仅通过环境变量（`process.env`）注入；代码内不读取 `config.toml`（见 `packages/core/src/config.ts` 的 `loadConfig`）。

---

## 后果（Consequences）

### 正面

- **单一语言、统一工具链**：从「写功能」到「编译出二进制」之间不再有 Rust 工具链摩擦，迭代周期更短。
- **分发简单**：一个二进制 `dist/vynth` 即可分发，无 node_modules、无外部 wasm 资源文件。
- **招人 / 协作面更宽**：贡献者只需懂 TypeScript 与 Bun。
- **启动快**：冷启动目标 **50–150ms**（Bun 启动 + 极少模块初始化）。

### 代价 / 性能取舍

- **单二进制体积**：目标 **20–40MB**；当前因 `tsconfig` 残留 `react`/`react-dom` 类型与 react-devtools 引用，体积约 **61MB**，待 `minify` 与依赖收敛优化（见工程任务 #1）。
- **运行时依赖 Bun 演进**：单二进制与 `bun build --compile` 的能力边界随 Bun 版本变化；若 Bun 在某平台缺失或行为变更，分发受影响。
- **放弃 Rust 的零成本抽象与系统级控制**：极端高吞吐 / CPU 密集路径（如大文件流式 diff）受 V8 调度限制；当前可接受的现实是 **V8 GC 偶发 <5ms 停顿**。
- **TUI 需自维护**：放弃 ink 后，布局 / 重绘 / 输入处理由 `tui` 包自建（目前为全屏重绘 + `StreamArea` 直写，未做局部 diff）。

### 兼容性约束

- `bun build --compile` 必须能打包所有 `@vynth/*` 源码与 `ansi-escapes` 等纯 JS 依赖；**禁止引入无法被 bun 打包的原生 / wasm 模块**（这是「不用 ink」的同一根因约束）。

---

## 回退（Rollback / 回退条件）

若出现以下任一情况，回退到 **Node + esbuild 分发**（保持 `@vynth/*` TypeScript 源码不变，仅替换「打包 / 分发」层）：

- **Bun 不支持关键原生模块**：某个必要的 npm 原生包 / Node 内置能力在 `bun build --compile` 下无法运行或无法打包成单二进制；
- **跨平台分发受阻**：目标平台（如某 Linux musl / Windows / 特定 arch）的 `bun build --compile` 产物不可用，且短期无修复；
- **性能缺口无法接受**：冷启动或二进制体积长期无法满足产品 SLA，且确认是 Bun 运行时层面的限制而非可优化项。

回退方案要点：

- **保留全部 `@vynth/*` 源码与 `loadConfig` / agent 循环 / 工具系统不变**；
- 用 **esbuild（或 tsup）** 打包为 ESM/CJS bundle，配合 `pkg` / `node` 运行时分发；
- TUI 的 ANSI 渲染器与 `StreamArea` 逃生舱协议保持不变（与运行时无关）。

> 回退是「分发层」回退，不是「架构层」回退：纯 TS 包结构、事件总线、工具注册表、agent 循环等核心设计均延续。
