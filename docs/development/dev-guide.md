# zeno 开发规范（工程规矩）

> 本文件是 `docs/实施开发计划.md §1` 的落地细则，所有开发前置闸门。
> 对齐已交付架构：`delivery/部署设计.md §4`（CI/CD 8 阶段）、`delivery/安全设计.md §1/§6`（密钥红线）、`delivery/系统设计.md`（错误码 6 位体系）。

## 1. 包边界（M1–M7 ↔ @zeno/*）

| 模块 | 包 | 职责 |
|---|---|---|
| M1 接入层 | `@zeno/cli` (apps/cli) | 单二进制入口、参数解析、退出码、TUI/无头分发 |
| M2 业务能力 | `@zeno/engine` | agent loop、LLM 客户端、内置工具 |
| M3 基础能力 | `@zeno/core` | 类型/配置/错误码/事件/日志（跨域唯一共享契约） |
| M4 基础能力 | `@zeno/sandbox` | 工具执行沙箱 + 越界守卫 |
| M5 扩展 | `@zeno/plugins` | 插件加载与生命周期 |
| M6 扩展 | `@zeno/tui` | 自研 ANSI 渲染 + 双模式 + 主题 |
| M7 扩展 | `@zeno/mcp` | MCP stdio 客户端（F12，尚未并入 CLI） |
| — 测试 | `@zeno/harness` | e2e 夹具与测试 |

**铁律**：跨包仅经 `@zeno/core` 导出的契约类型（`ZenoConfig` / `StreamEvent` / `ToolDef` / `ZenoError` / `Emitter`）。禁止包间直接 import 实现。

## 2. 代码质量

- 格式化/校验：**biome**（`biome check .` 必须 0 error）。规则：`single` 引号、`semicolons: always`、`noExplicitAny: error`、`noDefaultExport: error`、`lineWidth: 100`。
- TypeScript：`strict: true`，`verbatimModuleSyntax: false`。
- 错误表达：统一 `ZenoError` 子类；**v0.1.0 起落地 6 位全局错误码 `VC-XXXXXX`**（旧字符串域 `config/llm/tool/...` 由 `fromLegacy()` 兼容映射，详见 `packages/core/src/error-codes.ts`）。

### 错误码权威表（`packages/core/src/error-codes.ts`）

**编号规则**：`VC-AABBCC`，AA = 族、BB = 子类、CC = 实例。

| 码 | 语义 | 抛出位置 |
|---|---|---|
| `VC-010001` | `CONFIG_MISSING_KEY` | （预留） |
| `VC-010002` | `CONFIG_INVALID_MODE` | `apps/cli` 非法 `-m` |
| `VC-010003` | `CONFIG_UNKNOWN_FLAG` | `apps/cli` 未知参数 |
| `VC-010004` | `CONFIG_VALUE_MISSING` | `apps/cli` `-g`/`-m`/`-p` 缺值 |
| `VC-020001` | `LLM_AUTH_FAILED` | （预留） |
| `VC-020002` | `LLM_RATE_LIMITED` | （预留） |
| `VC-020003` | `LLM_NETWORK` | （预留） |
| `VC-020004` | `LLM_INVALID_RESPONSE` | `OpenAiProvider` SSE 解析失败 |
| `VC-020005` | `LLM_PLAINTEXT_HTTP` | `OpenAiProvider` 拒绝明文 http |
| `VC-030001` | `SANDBOX_PATH_ESCAPE` | `safeResolve` 拒绝 `../` 与绝对路径 |
| `VC-030002` | `SANDBOX_SYMLINK_ESCAPE` | `safeResolve` realpath 后越界 |
| `VC-030003` | `SANDBOX_NETWORK_BLOCKED` | `runCommand` `ZENO_NET=0` 阻断 |
| `VC-030004` | `SANDBOX_READ_FAILED` | `runCommand` 命令超时 |
| `VC-030005` | `SANDBOX_WRITE_FAILED` | `runCommand` 非 0 exit |
| `VC-040001` | `TOOL_NOT_FOUND` | `ToolRegistry.run` 未知工具 |
| `VC-040002` | `TOOL_EXECUTION_FAILED` | （预留） |
| `VC-040003` | `TOOL_INVALID_ARGS` | （预留） |
| `VC-050001` | `PLUGIN_LOAD_FAILED` | `loadPlugin` 动态 import 失败 |
| `VC-050002` | `PLUGIN_MISSING_ACTIVATE` | `loadPlugin` 缺导出 |
| `VC-050003` | `PLUGIN_MISSING_NAME` | `loadPlugin` 缺导出 |
| `VC-060001` | `MCP_NOT_IMPLEMENTED` | （F12 接入占位） |
| `VC-060002` | `MCP_PROTOCOL_PARSE` | （F12） |
| `VC-060003` | `MCP_REQUEST_TIMEOUT` | （F12） |

> 抛出约定：`new SandboxError('msg', 'VC-030001')` —— 第二个参数为 6 位码。
> CLI / 工具结果中的错误字符串前缀格式：`"[VC-030001] path escapes sandbox: ../x"`，
> 可被 grep / 日志聚合 / 监控告警按码字符串直接定位。


- 依赖纪律：**禁止引入大依赖**（直接威胁单二进制体积 ≤61MB 门禁）。新增依赖须 PR 说明体积影响。

## 3. CI/CD 8 阶段（`.github/workflows/ci.yml`）

1. install（`pnpm install --frozen-lockfile` —— 见下方说明）
2. lint & typecheck（`bun run lint`）
3. build packages（`bun run turbo run build`）
4. compile single binary（`bun run compile` → `dist/zeno`）
5. tests（`bun test packages`，含 harness e2e）
6. **安全扫描 — 密钥硬编码红线（gitleaks）**：提交真实密钥即阻断
7. **体积门禁**（`bun scripts/check-binary-size.ts`，MVP ≤61MB）
8. sign/notarize + publish（仅 tag/release 或 main 推送）

本地等价校验：`pnpm install && bun run lint && bun run compile && bun test packages && bun scripts/check-binary-size.ts`。

### 3.1 为什么 install 用 pnpm 而不是 bun install

bun 1.3.14 **不会** 自动给 monorepo workspace 建立 `node_modules/@zeno/*` 软链接，导致 `bun test packages` 在执行 `import { McpClient } from '@zeno/mcp'` 时报 `Cannot find module '@zeno/mcp'`。

项目已有 `pnpm-workspace.yaml`，CI 改用 pnpm 安装，并通过根目录 `.npmrc` 设置：

```ini
shamefully-hoist=true
```

pnpm 在严格模式下默认不把 workspace 依赖 hoist 到顶层；`shamefully-hoist=true` 强制把所有 `@zeno/*` 软链到顶层 `node_modules/@zeno/`，bun 即可正常解析。锁文件同时维护两份可能引起混淆，**项目仅维护 `pnpm-lock.yaml`**，提交策略：

- CI / 本地均用 `pnpm install --frozen-lockfile`
- 锁文件更新：用 `pnpm install` 后再 `pnpm install --lockfile-only` 校验
- 历史 `bun.lock` 已不再使用

## 4. 提交与分支模型

- 分支：`main` 受保护；功能分支 `feat/<fxx>` / `fix/<xxx>`；发布分支 `release/<ver>`。
- 提交：**Conventional Commits**，必须关联功能/模块编号：
  - `feat(F4): agent loop 支持 maxSteps 上限`
  - `fix(X3): sandbox safeResolve 修正符号链接逃逸`
  - `chore: 引入体积门禁脚本`
- PR：需 ≥1 审批 + CI 全绿 + e2e 通过；合并至 `main` 即触发 beta 发布闸门。
- 退出码语义（CLI）：`0`=正常；`2`=非交互终端却未给 `-g`（见 `apps/cli/src/main.ts`）；其它非 0=运行错误。

## 5. 安全红线（MVP 即生效，对齐 安全设计.md）

- **密钥绝不硬编码**；`ZENO_API_KEY` 仅经环境变量/OS keychain 入参（见 `ZENO_NET` 与密钥分级 §6）。
- **提示注入防护**：系统提示完整性保护、工具结果沙箱化、危险操作二次确认。
- **命令注入防护**：shell 工具禁用动态拼接，参数化白名单数组。
- **路径穿越防护**：`sandbox.safeResolve` 规范化 + 符号链接守卫（对抗 X3 sandbox symlink bug）。
- **SSRF 防护**：工具出站仅允许 `ZENO_NET` 白名单（默认 `api.deepseek.com:443`）。
- **沙箱策略（方案 A）**：MVP 维持设计信任模型（软 `ZENO_NET`），OS 级硬隔离推迟至 F15。

## 6. 默认配置冻结值（X1–X5，以代码为准）

| 项 | 值 | 来源 |
|---|---|---|
| LLM 端点 | `https://api.deepseek.com/v1` | `packages/core/src/config.ts` |
| 默认模型 | `deepseek-v4-pro` | `packages/core/src/config.ts` |
| 插件 CLI | `--plugin` 无头已接入 | 高层架构 O1/F9 |
| sandbox symlink | 已由 `safeResolve` 守卫修复 | 冲突 X3 |
| workspace 声明 | pnpm 双声明历史兼容 | 冲突 X5 |
