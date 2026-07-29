# 配置详解

Zeno **仅通过环境变量** 配置，不读取任何配置文件（如 `.env`、`config.toml`、`yaml`）。这一设计决策确保单二进制无需额外配置即可运行，同时避免敏感配置被意外提交到仓库。

---

## 完整变量表

| 变量 | 作用 | 默认值 | 必填 | 读取位置 |
|------|------|--------|------|----------|
| `ZENO_API_KEY` | LLM API Key | 空 | **是** | `core` `loadConfig` |
| `ZENO_MODEL` | 模型名 | `deepseek-v4-pro` | 否 | `core` `loadConfig` |
| `ZENO_LLM_BASE_URL` | OpenAI 兼容端点 | `https://api.deepseek.com/v1` | 否 | `core` `loadConfig` |
| `ZENO_MODE` | 运行模式 | `vibe` | 否 | `core` `loadConfig` |
| `ZENO_THEME` | TUI 主题 | `mocha` | 否 | `core` `loadConfig` |
| `ZENO_NET` | 沙箱网络开关 | 开启 | 否 | `core` `loadConfig` → `sandbox` |
| `ZENO_DATA_DIR` | 数据目录 | `~/.zeno` | 否 | `core` `loadConfig` |

---

## 变量详解

### ZENO_API_KEY

LLM 提供商的 API Key。

- **必填**：未设置时将抛出 `LlmError`，不会进入 demo 模式。
- **非空值**：使用 `OpenAiProvider` 走 OpenAI 兼容 SSE 流式。

```bash
# 真实 LLM
export ZENO_API_KEY="sk-..."
./dist/zeno -g '重构 utils'
```

> **安全提示**：`ZENO_API_KEY` 通过环境变量注入，不会被写入日志或配置文件。插件可访问环境变量，请仅加载可信插件。

### ZENO_MODEL

指定 LLM 模型名。

- 默认：`deepseek-v4-pro`
- 支持任何 OpenAI 兼容端点支持的模型

```bash
export ZENO_MODEL="deepseek-v4-pro"
./dist/zeno -g '写一个快速排序'
```

### ZENO_LLM_BASE_URL

OpenAI 兼容 SSE 端点。

- 默认：`https://api.deepseek.com/v1`
- 支持本地服务（如 Ollama）

```bash
# 本地 Ollama
export ZENO_LLM_BASE_URL="http://localhost:11434/v1"
export ZENO_API_KEY="ollama"  # Ollama 通常忽略 key
./dist/zeno -g '解释量子纠缠'
```

> **安全红线**：非默认端点发送 API Key 时，`OpenAiProvider` 会发出告警。`http`（非 `https`）且非 `localhost` 会被拒绝。

### ZENO_MODE

运行模式。

| 值 | 行为 |
|----|------|
| `vibe`（默认） | 边聊边写，agent 自主决定工具调用 |
| `plan` | 先规划再动手，输出计划而非直接执行 |

```bash
./dist/zeno -m plan '设计用户认证模块'
```

### ZENO_THEME

TUI 主题。

| 值 | 说明 |
|----|------|
| `mocha`（默认） | 深色主题（Catppuccin Mocha） |
| `latte` | 浅色主题（Catppuccin Latte） |
| `neon` | 赛博青绿主题（新增，面向暗色终端 / 极客风，可通过 `ZENO_THEME=neon` 启用） |
| `midnight` / `forest` / `light` | 深蓝 / 深绿 / 纯白（TUI 内 `/theme` 循环可切换，未纳入 env 解析） |

### ZENO_NET

沙箱网络开关。

| 值 | 行为 |
|----|------|
| 未设置 / `1` / `true` / `yes` | 允许联网（默认） |
| `0` / `off` / `false` / `no` | 禁止联网 |

`run_shell` 工具执行时会检查此开关。关闭时，任何出站网络请求（除 `localhost` 外）都会被拦截。

```bash
# 禁用联网
export ZENO_NET="0"
./dist/zeno -g '分析本地代码'
```

### ZENO_HARDEN

OS 级硬隔离：把 `run_shell` 启动的子进程放入操作系统强制沙箱。

| 值 | 行为 |
|----|------|
| 未设置 / `0` | 关闭（默认，仅应用层 `safeResolve` 路径守卫生效） |
| `1` | 开启进程级隔离：Linux=bubblewrap（推荐）；macOS=seatbelt，但 macOS 15+ 非 root 会 Fail-Closed（详见平台要求） |

```bash
# 强制 OS 级隔离（Fail-Closed：后端不可用时直接拒绝，不静默降级）
export ZENO_HARDEN="1"
./dist/zeno -g '只改本地文件'
```

> **平台要求**：Linux 用 `bwrap`（bubblewrap，推荐，非 root 可用）。macOS 用 `sandbox-exec`，但 **macOS 15+ 已禁止非 root 进程 apply 任意 seatbelt 策略**（连纯允许策略也报 `Operation not permitted`），因此普通开发机（非 root）上 `ZENO_HARDEN=1` 会直接 Fail-Closed 返回 `VC-030006`——这是刻意的安全设计，不是缺陷。如需 macOS 上真正生效，请以 root 运行，或改用 Linux。其它平台（Windows 等）不支持，开启时同样返回 `VC-030006` 而非静默放行。

> 也可在配置文件 `sandbox.harden` 中开启（见下文）。环境变量优先级更高：`ZENO_HARDEN=0` 可强制关闭，覆盖文件的 `true`。

### ZENO_DATA_DIR

数据存储目录。

- 默认：`~/.zeno`
- 用于存储会话历史、审计日志（完整版功能）

```bash
export ZENO_DATA_DIR="/path/to/data"
./dist/zeno
```

---

## 最佳实践

### 开发环境

```bash
# .zshrc / .bashrc
export ZENO_API_KEY="sk-..."
export ZENO_MODEL="deepseek-v4-pro"
export ZENO_MODE="vibe"
export ZENO_THEME="mocha"
```

### CI / 脚本环境

```bash
#!/usr/bin/env bash
set -euo pipefail

# 无头模式 + 禁用联网（安全基线）
ZENO_NET="0" ./dist/zeno -g '运行测试并修复 lint 错误'
```

### 多项目隔离

```bash
# 项目级 .env（需配合 direnv / autoenv）
export ZENO_DATA_DIR="$(pwd)/.zeno"
export ZENO_API_KEY="sk-..."
```

---

## 配置优先级

`loadConfig` 的解析顺序（高优先级覆盖低优先级）：

1. **代码参数**（`overrides`，仅内部测试使用）
2. **环境变量**（`process.env`）
3. **默认值**

```typescript
const config = loadConfig({
  mode: 'plan',  // 最高优先级
});
// 等价于
const config = loadConfig();
// 但 ZENO_MODE 环境变量会被 mode 参数覆盖
```

---

## 可选配置文件（F14，ADR-0003 扩展）

Zeno **仍以环境变量为主配置源**。F14 增加了一个**可选**的 JSON 配置文件层，作为便利补充；环境变量始终拥有最高优先级，配置文件**不会**覆盖环境变量。

查找顺序：

1. `ZENO_CONFIG_FILE` 指向的显式路径；
2. 否则 `<ZENO_DATA_DIR>/config.json`（默认 `~/.zeno/config.json`）。

示例 `~/.zeno/config.json`：

```json
{
  "model": "deepseek-v4-pro",
  "theme": "latte",
  "audit": true,
  "sandbox": { "harden": true }
}
```

合并优先级（高 → 低）：**代码 overrides > 环境变量 > 配置文件 > 默认值**。

> **安全红线**：配置文件中**不得写入 `apiKey`**（违反将抛 `VC-010005`）。密钥只允许通过 `ZENO_API_KEY` 环境变量注入。包含未知键或非法 JSON 会抛 `VC-010006`，配置文件即停止生效——宁可启动失败，也不静默接受错误配置。

允许的键：`mode` / `model` / `llmBaseUrl` / `theme` / `sandbox.{networkAllowed,cwd,harden}` / `dataDir` / `audit`。

---

## 审计日志（F14）

Zeno 内置 **5 维审计**，用于完整版的合规留痕：

| 维度 | 触发点 |
|------|--------|
| `tool_exec` | agent 工具执行（成功 / 失败） |
| `file_access` | 文件读 / 写（沙箱，含越界失败） |
| `network_egress` | `run_shell` 联网出站（放行 / 被 `ZENO_NET` 拦截） |
| `config_change` | 配置文件加载 / 生效 |
| `plugin_load` | 插件 `import` + `activate`（成功 / 失败） |

审计落盘为 **append-only JSONL** 文件 `<ZENO_DATA_DIR>/audit.log`（零依赖，符合 ADR-0003 对单二进制的约束，不引入原生模块）。每条记录含 `ts`（ISO 8601）、`kind`、`ok`、`detail`。

**默认关闭**。启用方式（任选其一）：

```bash
# 环境变量
export ZENO_AUDIT=1

# 或配置文件
echo '{ "audit": true }' > ~/.zeno/config.json
```

启用后，高危操作即被 100% 策略留痕（详见验收闸门「完整版：高危操作 100% 策略留痕」）。

---

## 相关文档

- [快速开始](getting-started.md) —— 30 秒跑通真实链路
- [API 参考](../api/overview.md) —— CLI 参数与退出码
- [开发规范](../development/dev-guide.md) —— 安全红线与冻结值
