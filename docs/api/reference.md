# API 参考（详细）

本文档提供 Zeno 对外接口的完整参考，包括 CLI 参数、环境变量、退出码、工具调用协议。

---

## CLI 参数

| 参数 | 等价长名 | 作用 | 备注 |
|------|----------|------|------|
| `-g <目标>` | `--goal <目标>` | 无头 agent 模式，把 `<目标>` 流式输出到 stdout | 无需 TTY；需设置 `ZENO_API_KEY` |
| `-m <mode>` | `--mode <mode>` | 指定模式 `plan` \| `vibe` | 覆盖 `ZENO_MODE`，默认 `vibe` |
| `--plugin <path>` | — | 加载本地插件入口 | 仅无头模式 `-g` 下生效 |
| `-v` | `--version` | 输出版本号并退出 | 退出码 0 |
| `-h` | `--help` | 输出用法说明并退出 | 退出码 0 |
| （无参数） | — | 启动交互式 TUI | 需真实 TTY |

### 用法示例

```bash
# 交互 TUI
zeno

# 无头 agent
zeno -g '给当前目录写个 README'

# 指定模式
zeno -m plan -g '重构 src 下的工具函数'

# 加载插件
zeno --plugin ./my-plugin.ts -g '使用自定义工具'

# 查看版本
zeno --version

# 查看帮助
zeno --help
```

---

## 环境变量

| 变量 | 作用 | 默认值 | 读取位置 |
|------|------|--------|----------|
| `ZENO_API_KEY` | LLM API Key（必填） | 空 | `core` `loadConfig` |
| `ZENO_MODEL` | 模型名 | `deepseek-v4-pro` | `core` `loadConfig` |
| `ZENO_LLM_BASE_URL` | OpenAI 兼容端点 | `https://api.deepseek.com/v1` | `core` `loadConfig` |
| `ZENO_MODE` | `plan` \| `vibe` | `vibe` | `core` `loadConfig` |
| `ZENO_THEME` | `mocha` \| `latte`（Catppuccin） | `mocha` | `core` `loadConfig` |
| `ZENO_NET` | 沙箱网络开关；`'0'` = 禁止联网 | 开启（非 `'0'`） | `core` `loadConfig` → `sandbox` |
| `ZENO_DATA_DIR` | 数据目录 | `~/.zeno` | `core` `loadConfig` |

> 注：配置**仅通过环境变量注入**（`loadConfig` 只读 `process.env`，不读取任何配置文件）。

---

## 退出码

| 退出码 | 含义 | 触发条件 |
|--------|------|----------|
| `0` | 正常结束 | `--version` / `--help` / TUI 正常退出 / 无头任务完成 |
| `2` | 用法错误 | 参数非法 / 必要输入缺失 / 非 TTY 无 `-g` |
| 非 0（≠2） | 运行期错误 | LLM 调用失败、工具执行失败等未捕获异常 |

### 退出码契约

```typescript
// 用法错误（F11 契约）
- 未知参数 → exit 2
- -g 缺目标 → exit 2
- -m 非法值 → exit 2
- 非 TTY 无 -g → exit 2

// 正常结束
- --version → exit 0
- --help → exit 0
- 无头任务完成 → exit 0

// 运行期错误
- LLM 网络失败 → 未捕获异常 → 非 0
- 工具执行未捕获异常 → 非 0
```

---

## 工具调用协议

Agent 循环中，LLM 可通过 `tool_calls` 请求执行工具。工具结果以 `ToolResult` 回灌。

### ToolResult 结构

```typescript
interface ToolResult {
  ok: boolean;      // 是否成功
  output: string;   // 成功时的输出
  error?: string;   // 失败时的错误信息（可选）
}
```

### 工具调用示例

```json
{
  "id": "call_abc123",
  "type": "function",
  "function": {
    "name": "read_file",
    "arguments": "{\"path\": \"README.md\"}"
  }
}
```

### 工具结果回灌

```json
{
  "role": "tool",
  "tool_call_id": "call_abc123",
  "content": "Zeno 是一个..."
}
```

---

## 实现状态

以��为当前 `apps/cli/src/main.ts` 与产品决策之间的差异：

- **`--plugin <path>`（CLI 加载）**：✅ 已实现。`main.ts` 的 `parseArgs` 已解析 `-p/--plugin`，`runHeadless` 经 `loadPlugin(abs)` + `plugin.activate(tools)` 动态加载插件并向 agent 工具集注册工具（无头模式 `-g` 下生效）。
- **退出码 `2`**：✅ 已实现。非 TTY 环境启动 TUI 时 `main.ts` 已显式 `process.exit(2)` 并提示改用无头模式；运行期错误仍以未捕获异常冒泡至 Bun 默认非 0 退出。
- **`mcp` 接入**：`packages/mcp` 的 `McpClient` 可用，但同样尚未并入 `apps/cli` 的 agent 工具集；接入后 MCP 工具将经由同一 `ToolRegistry` 暴露。

---

## 相关文档

- [快速开始](../guide/getting-started.md) —— 30 秒跑通真实链路
- [配置详解](../guide/configuration.md) —— 环境变量最佳实践
- [架构总览](../architecture/index.md) —— 模块关系与数据流
