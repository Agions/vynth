# vynth 开发规范（工程规矩）

> 本文件是 `docs/实施开发计划.md §1` 的落地细则，所有开发前置闸门。
> 对齐已交付架构：`delivery/部署设计.md §4`（CI/CD 8 阶段）、`delivery/安全设计.md §1/§6`（密钥红线）、`delivery/系统设计.md`（错误码 6 位体系）。

## 1. 包边界（M1–M7 ↔ @vynth/*）

| 模块 | 包 | 职责 |
|---|---|---|
| M1 接入层 | `@vynth/cli` (apps/cli) | 单二进制入口、参数解析、退出码、TUI/无头分发 |
| M2 业务能力 | `@vynth/engine` | agent loop、LLM 客户端、内置工具 |
| M3 基础能力 | `@vynth/core` | 类型/配置/错误码/事件/日志（跨域唯一共享契约） |
| M4 基础能力 | `@vynth/sandbox` | 工具执行沙箱 + 越界守卫 |
| M5 扩展 | `@vynth/plugins` | 插件加载与生命周期 |
| M6 扩展 | `@vynth/tui` | 自研 ANSI 渲染 + 双模式 + 主题 |
| M7 扩展 | `@vynth/mcp` | MCP stdio 客户端（F12，尚未并入 CLI） |
| — 测试 | `@vynth/harness` | e2e 夹具与测试 |

**铁律**：跨包仅经 `@vynth/core` 导出的契约类型（`VynthConfig` / `StreamEvent` / `ToolDef` / `VynthError` / `Emitter`）。禁止包间直接 import 实现。

## 2. 代码质量

- 格式化/校验：**biome**（`biome check .` 必须 0 error）。规则：`single` 引号、`semicolons: always`、`noExplicitAny: error`、`noDefaultExport: error`、`lineWidth: 100`。
- TypeScript：`strict: true`，`verbatimModuleSyntax: false`。
- 错误表达：统一 `VynthError` 子类；**演进目标**为 6 位全局错误码 `VC-XXXXXX`（当前为字符串域 `config/llm/tool/...`，见 `packages/core/src/errors.ts`，需在 F 系列中迁移）。
- 依赖纪律：**禁止引入大依赖**（直接威胁单二进制体积 ≤61MB 门禁）。新增依赖须 PR 说明体积影响。

## 3. CI/CD 8 阶段（`.github/workflows/ci.yml`）

1. install（`bun install --frozen-lockfile`）
2. lint & typecheck（`bun run lint`）
3. build packages（`bun run turbo run build`）
4. compile single binary（`bun run compile` → `dist/vynth`）
5. tests（`bun test packages`，含 harness e2e）
6. **安全扫描 — 密钥硬编码红线（gitleaks）**：提交真实密钥即阻断
7. **体积门禁**（`bun scripts/check-binary-size.ts`，MVP ≤61MB）
8. sign/notarize + publish（仅 tag/release 或 main 推送）

本地等价校验：`bun run lint && bun run compile && bun test packages && bun scripts/check-binary-size.ts`。

## 4. 提交与分支模型

- 分支：`main` 受保护；功能分支 `feat/<fxx>` / `fix/<xxx>`；发布分支 `release/<ver>`。
- 提交：**Conventional Commits**，必须关联功能/模块编号：
  - `feat(F4): agent loop 支持 maxSteps 上限`
  - `fix(X3): sandbox safeResolve 修正符号链接逃逸`
  - `chore: 引入体积门禁脚本`
- PR：需 ≥1 审批 + CI 全绿 + e2e 通过；合并至 `main` 即触发 beta 发布闸门。
- 退出码语义（CLI）：`0`=正常；`2`=非交互终端却未给 `-g`（见 `apps/cli/src/main.ts`）；其它非 0=运行错误。

## 5. 安全红线（MVP 即生效，对齐 安全设计.md）

- **密钥绝不硬编码**；`VYNTH_API_KEY` 仅经环境变量/OS keychain 入参（见 `VYNTH_NET` 与密钥分级 §6）。
- **提示注入防护**：系统提示完整性保护、工具结果沙箱化、危险操作二次确认。
- **命令注入防护**：shell 工具禁用动态拼接，参数化白名单数组。
- **路径穿越防护**：`sandbox.safeResolve` 规范化 + 符号链接守卫（对抗 X3 sandbox symlink bug）。
- **SSRF 防护**：工具出站仅允许 `VYNTH_NET` 白名单（默认 `api.deepseek.com:443`）。
- **沙箱策略（方案 A）**：MVP 维持设计信任模型（软 `VYNTH_NET`），OS 级硬隔离推迟至 F15。

## 6. 默认配置冻结值（X1–X5，以代码为准）

| 项 | 值 | 来源 |
|---|---|---|
| LLM 端点 | `https://api.deepseek.com/v1` | `packages/core/src/config.ts` |
| 默认模型 | `deepseek-chat` | `packages/core/src/config.ts`（已修正，原 `deepseek-v4-pro` 与冻结冲突） |
| 插件 CLI | `--plugin` 无头已接入 | 高层架构 O1/F9 |
| sandbox symlink | 已由 `safeResolve` 守卫修复 | 冲突 X3 |
| workspace 声明 | pnpm 双声明历史兼容 | 冲突 X5 |
