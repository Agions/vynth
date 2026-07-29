# 常见问题（FAQ）

---

## 安装与运行

### Q: 为什么需要 Bun？不能只用 Node.js 吗？

A: Zeno 依赖 `bun build --compile` 打包为单二进制，这是核心分发形态。开发时可以使用 Node.js 运行辅助工具（如 `biome`、`turbo`），但运行时必须用 Bun。

### Q: 编译后的二进制很大（60MB），可以优化吗？

A: 当前体积 60.51MB 包含 Bun 运行时 + 所有依赖。完整版目标 ≤ 40MB，通过以下方式优化：
- 移除未使用的依赖
- 压缩 TUI 资源（当前已用轻量 ANSI，非 ink）
- 延迟加载非核心模块

### Q: Windows 支持如何？

A: 当前 CI 仅覆盖 macOS / Linux。Windows 上 `run_shell` 会回退到 `cmd /c`，但未充分测试。完整版将改善跨平台支持。

### Q: 为什么 `zeno` 命令找不到？

A: 编译后的二进制位于 `./dist/zeno`。临时使用：

```bash
./dist/zeno --help
```

永久使用：

```bash
bun link  # 或手动加入 PATH
```

---

## 配置

### Q: 环境变量配置文件放在哪里？

A: Zeno **不支持**配置文件，仅通过环境变量配置。你可以：
- 在 shell 配置文件中设置（`~/.zshrc`、`~/.bashrc`）
- 使用 `direnv` 等工具按项目自动加载
- 在 CI 脚本中直接 export

### Q: 无 API Key 能做什么？

A: `ZENO_API_KEY` 为**必填项**。未设置时 `createProvider` 会抛出 `LlmError`，不会进入 demo 模式。请先设置 API Key 再运行。

### Q: 如何接入 OpenAI / 其他兼容端点？

```bash
export ZENO_API_KEY="sk-..."
export ZENO_LLM_BASE_URL="https://api.deepseek.com/v1"
export ZENO_MODEL="deepseek-v4-pro"
./dist/zeno -g '你的目标'
```

> 提示：默认端点已指向 DeepSeek，若使用 OpenAI 或其他兼容服务，需同时修改 `ZENO_LLM_BASE_URL` 与 `ZENO_MODEL`。

### Q: `ZENO_NET='0'` 会影响插件吗？

A: 会的。`run_shell` 工具会检查 `ZENO_NET`，关闭时禁止出站网络请求。但插件注册的自定义工具不受此约束（插件可自行发起网络请求）。

---

## 使用

### Q: TUI 卡住 / 无响应？

A: 确认在真实终端（TTY）中运行。管道 / CI 环境请用无头模式：

```bash
./dist/zeno -g '你的目标'
```

### Q: 如何调试工具调用？

在插件中打印日志：

```typescript
execute: async (args) => {
  console.error('[DEBUG] args:', args);
  return { ok: true, output: '...' };
}
```

### Q: Agent 循环次数太多 / 太少？

A: `maxSteps` 默认 8。完整版将支持通过环境变量配置。

### Q: 如何保存会话历史？

A: 当前版本（v0.1.0）不持久化会话历史。完整版（F14）将支持 SQLite 存储与审计日志。

---

## 插件开发

### Q: 插件可以调用其他插件吗？

A: 可以。插件注册的工具统一进入 `ToolRegistry`，agent 可按需调用。

### Q: 插件可以修改配置吗？

A: 当前不支持。配置仅通过环境变量注入，插件无法修改 `loadConfig` 返回值。

### Q: 插件支持热重载吗？

A: 不支持。每次运行需重新加载。

---

## 安全

### Q: 插件安全吗？

A: **不安全**。插件在当前进程中执行任意代码，拥有与 Zeno 同等的权限。仅加载可信插件。

### Q: 数据会发送到第三方吗？

A: LLM 请求发送到 `ZENO_LLM_BASE_URL` 指定的端点（默认 DeepSeek）。其他数据（文件内容、环境变量）不会自动上传。

### Q: 如何审计工具调用？

A: F14 已内置 5 维审计日志（`tool_exec` / `file_access` / `network_egress` / `config_change` / `plugin_load`），通过 `ZENO_AUDIT=1` 或配置文件 `audit:true` 启用，落盘 `<ZENO_DATA_DIR>/audit.log`。插件也可在自身逻辑中手动记录：

```typescript
execute: async (args) => {
  console.log('[AUDIT]', new Date().toISOString(), args);
  return { ok: true, output: '...' };
}
```

---

## 故障排查

### Q: 编译失败：`bun: command not found`

A: 安装 Bun：

```bash
curl -fsSL https://bun.sh/install | bash
```

### Q: 测试失败：`Cannot find module '@zeno/core'`

A: 确保已安装依赖：

```bash
bun install
```

### Q: LLM 请求超时？

A: 检查网络连接与 `ZENO_LLM_BASE_URL` 可达性。可增加超时时间（完整版支持）。

### Q: 体积门禁失败？

A: 检查是否引入大体积依赖。当前 MVP 上限 61MB，完整版目标 40MB。

---

## 相关文档

- [快速开始](../guide/getting-started.md) —— 30 秒跑通真实链路
- [配置详解](../guide/configuration.md) —— 环境变量最佳实践
- [插件开发](../guide/plugins.md) —— 插件 manifest 与生命周期
- [开发规范](../development/dev-guide.md) —— 安全红线与冻结值
