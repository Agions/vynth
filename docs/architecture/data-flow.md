# 数据流

> 理解 Vynth 的数据流是贡献代码的第一步。本文档描述从用户输入到渲染输出的完整链路。

---

## 端到端数据流

```
            ┌──────────────────────── 交互入口 ────────────────────────┐
            │  vynth (TUI, 需 TTY)        vynth -g '<goal>' (无头)      │
            └───────────────────────────┬─────────────────────────────┘
                                         │  goal (string) + loadConfig()
                                         ▼
            ┌─────────────────────────── agent-loop ───────────────────────────┐
            │  runAgent(goal):                                                  │
            │   1. 组装 messages = [system, user(goal)]                          │
            │   2. 取 toolDefs = tools.list()                                    │
            │   3. for step in 0..maxSteps(默认 8):                              │
            │        - provider.chat(messages, toolDefs) → AsyncIterable<StreamEvent>
            │        - 收集 token / pendingTool                                 │
            │        - 若 pendingTool: tools.run(name, args) → ToolResult       │
            │        - 回填 messages: [assistant, tool]                         │
            └───────┬───────────────────────────────┬──────────────────────────┘
                    │ StreamEvent {token|tool|done}  │ tool call
                    ▼                                ▼
        ┌─────────────── LLM 层 ──────────────┐   ┌───────��── sandbox ──────────┐
        │ createProvider(config):             │   │ readText / writeText:        │
        │  - 有 key → OpenAiProvider           │   │   safeResolve 越界守卫        │
        │    fetch POST {baseUrl}/chat/completions │ runCommand(sh -c):          │
        │    SSE 逐行解析 → token / tool_calls │   │   网络开关(networkAllowed)   │
        │  - 无 key → 抛出 LlmError            │   │   超时 30s(SIGKILL)          │
        └─────────────────────────────────────┘   └─────────────────────────────┘
                    │                                │ ToolResult {ok,output,error}
                    └───────────────┬────────────────┘
                                    ▼
                        ┌────────── 渲染层 ──────────┐
                        │ TUI:  StreamArea 直写逃生舱  │
                        │       (全屏重绘 + 行内覆盖)  │
                        │ 无头: process.stdout.write   │
                        └──────────────────────────────┘
```

---

## 关键阶段详解

### 1. 入口与配置加载

**���发点**：`apps/cli/src/main.ts`

- **TUI 模式**：`vynth`（需真实 TTY）
- **无头模式**：`vynth -g '<目标>'`
- **插件模式**：`vynth -g '<目标>' -p <路径>`

配置通过 `loadConfig()` 从环境变量加载：

```typescript
const config = loadConfig({
  mode: parsed.mode,  // CLI 参数覆盖环境变量
});
```

### 2. Agent 循环（Engine）

**核心**：`packages/engine/src/agent-loop.ts`

```
for step in 0..maxSteps:
  ├─ provider.chat(messages, toolDefs)
  │   └─ OpenAiProvider: fetch SSE → 解析 delta.content / delta.tool_calls
  │
  ├─ 收集 StreamEvent:
  │   ├─ token: 追加到当前 assistant 消息
  │   ├─ tool: 记录 pendingTool（name + args）
  │   └─ done: 本轮完成
  │
  ├─ 若有 pendingTool:
  │   ├─ tools.run(name, args) → ToolResult
  │   ├─ 回灌 messages: [assistant(tool_calls), tool(result)]
  │   └─ 继续下一轮
  │
  └─ 若 finish_reason=stop 或达到 maxSteps:
      └─ 返回最终 StreamEvent{done}
```

### 3. LLM 层（Provider）

**接口**：`packages/engine/src/llm.ts`

| Provider | 触发条件 | 行为 |
|----------|----------|------|
| `OpenAiProvider` | `VYNTH_API_KEY` 非空 | 真实 SSE 流式（v0.2.1 起移除 EchoProvider；空 key 抛 `LlmError`） |

#### SSE 解析流程

```
fetch POST {baseUrl}/chat/completions
  ├─ Content-Type: text/event-stream
  ├─ 逐行解析：
  │   ├─ data: {"choices": [{"delta": {"content": "Hello"}}]}
  │   │   → StreamEvent{type: 'token', text: 'Hello'}
  │   │
  │   ├─ data: {"choices": [{"delta": {"tool_calls": [...]}}]}
  │   │   → StreamEvent{type: 'tool', call: {name, args}}
  │   │
  │   └─ data: {"choices": [{"finish_reason": "stop"}]}
  │       → StreamEvent{type: 'done'}
  │
  └─ data: [DONE]
      → 流结束
```

### 4. 沙箱执行（Sandbox）

**核心**：`packages/sandbox/src/sandbox.ts`

所有工具执行统一经过 sandbox：

| 工具 | 沙箱约束 |
|------|----------|
| `read_file` | `safeResolve` 拒绝 `../`、绝对路径、symlink 逃逸 |
| `write_file` | 同上 |
| `run_shell` | `safeResolve` + `VYNTH_NET` 网络开关 + 30s 超时 |

#### safeResolve 算法

```typescript
function safeResolve(cwd: string, userPath: string): string {
  // 1. 规范化路径
  const resolved = path.resolve(cwd, userPath);
  
  // 2. 确保在 cwd 内
  if (!resolved.startsWith(cwd)) {
    throw new SandboxError('路径越界');
  }
  
  // 3. 检查 symlink 逃逸
  const real = fs.realpathSync(resolved);
  if (!real.startsWith(cwd)) {
    throw new SandboxError('symlink 逃逸');
  }
  
  return resolved;
}
```

### 5. 渲染层

| 模式 | 渲染器 | 行为 |
|------|--------|------|
| TUI | `StreamArea` | 全屏 ANSI，逐字符流式 + 工具调用回显 + 逃生舱（避免卡顿时清屏） |
| 无头 | `process.stdout.write` | 直接输出到 stdout，适合管道 / CI |

---

## 关键不变量（Invariants）

1. **StreamEvent 是唯一跨层协议**：`engine` 向 `tui` / `cli` 只暴露 `StreamEvent`（`token` / `tool` / `done`），渲染层与上层解耦。
2. **工具结果统一为 `ToolResult`**：`{ ok, output, error? }`；失败不抛异常，而是 `ok:false` 回灌 `messages`，由 LLM 决定下一步。
3. **沙箱是工具执行的唯一出口**：所有 fs / shell 访问经 `sandbox` 的 `safeResolve` 越界守卫与网络开关，agent 不能直接触达宿主机任意路径。
4. **无 key 即失败**：`createProvider` 在 `apiKey` 为空时抛出 `LlmError`，要求显式配置 `VYNTH_API_KEY`。

---

## 错误传播

```
LLM 错�� (网络 / 认证 / 限流)
  └─ LlmError → 未捕获 → 进程非 0 退出

工具执行失败
  └─ ToolResult{ok: false, error: '...'}
      └─ 回灌 messages → LLM 决定重试 / 放弃

配置错误
  └─ ConfigError → 未捕获 → 进程非 0 退出

沙箱拦截
  └─ SandboxError → ToolResult{ok: false}
      └─ 回灌 messages → LLM 决定下一步
```

---

## 相关文档

- [架构总览](index.md) —— 模块关系与构建分发
- [API 参考](../api/overview.md) —— CLI 参数与退出码
- [开发规范](../development/dev-guide.md) —— 安全红线与冻结值
