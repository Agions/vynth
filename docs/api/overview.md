# API 总览（CLI）

Vynth 的对外接口是单二进制 `dist/vynth`（bin 名 `vynth`）。以下为完整 CLI 表面。

## 子命令 / 参数表

| 参数 | 等价长名 | 作用 | 备注 |
|------|----------|------|------|
| `-g <目标>` | `--goal <目标>` | 无头 agent 模式，把 `<目标>` 流式输出到 stdout | 无需 TTY；需设置 `VYNTH_API_KEY` |
| `-m <mode>` | `--mode <mode>` | 指定模式 `plan` \| `vibe` | 覆盖 `VYNTH_MODE`，默认 `vibe` |
| `--plugin <path>` | — | 加载本地插件入口（示例 / 扩展工具） | **见「实现状态」** |
| `-v` | `--version` | 输出版本号（`0.1.0`）并退出 | 退出码 0 |
| `-h` | `--help` | 输出用法说明并退出 | 退出码 0 |
| （无参数） | — | 启动交互式 TUI | 需真实 TTY |

> 用法示例：
> ```bash
> vynth                                  # 交互 TUI
> vynth -g '给当前目录写个 README'       # 无头 agent
> vynth -m plan -g '重构 src 下的工具函数' # 指定 plan 模式
> vynth --plugin ./my-plugin.ts          # 加载插件（实现状态见下）
> vynth --version
> vynth --help
> ```

## 环境变量

| 变量 | 作用 | 默认值 | 读取位置 |
|------|------|--------|----------|
| `VYNTH_API_KEY` | LLM API key（必填） | 空 | `core` `loadConfig` |
| `VYNTH_MODEL` | 模型名 | `deepseek-v4-pro` | `core` `loadConfig` |
| `VYNTH_LLM_BASE_URL` | OpenAI 兼容端点 | `https://api.deepseek.com/v1` | `core` `loadConfig` |
| `VYNTH_MODE` | `plan` \| `vibe` | `vibe` | `core` `loadConfig` |
| `VYNTH_THEME` | `mocha` \| `latte`（Catppuccin） | `mocha` | `core` `loadConfig` |
| `VYNTH_NET` | 沙箱网络开关；`'0'` = 禁止联网 | 开启（非 `'0'`） | `core` `loadConfig` → `sandbox` |
| `VYNTH_DATA_DIR` | 数据目录 | `~/.vynth` | `core` `loadConfig` |

> 注：配置**仅通过环境变量注入**（`loadConfig` 只读 `process.env`，不读取任何 `config.toml` 文件）。前 5 项为产品主配置；`VYNTH_NET` / `VYNTH_DATA_DIR` 为代码内已实现的附加开关。

## 退出码

| 退出码 | 含义 |
|--------|------|
| `0` | 正常结束（`--version` / `--help` / TUI 正常退出 / 无头任务完成） |
| `2` | 用法错误（参数非法 / 必要输入缺失） |
| 非 0（≠2） | 运行期错误（LLM 调用失败、工具执行失败等未捕获异常） |

> 约定：`0` 成功、`2` 用法错误、其余非 0 为运行错误。详见 [ADR 0003](../adr/0003-pure-typescript-build.md) 的分发约束。

## 实现状态 {#实现状态}

以下为当前 `apps/cli/src/main.ts` 与产品决策之间的差异，便于贡献者对齐：

- **`--plugin <path>`（CLI 加载）**：✅ 已实现。`main.ts` 的 `parseArgs` 已解析 `-p/--plugin`，`runHeadless` 经 `loadPlugin(abs)` + `plugin.activate(tools)` 动态加载插件并向 agent 工具集注册工具（无头模式 `-g` 下生效）。
- **退出码 `2`**：✅ 已实现。非 TTY 环境启动 TUI 时 `main.ts` 已显式 `process.exit(2)` 并提示改用无头模式；运行期错误仍以未捕获异常冒泡至 Bun 默认非 0 退出。
- **`mcp` 接入**：`packages/mcp` 的 `McpClient` 可用，但同样尚未并入 `apps/cli` 的 agent 工具集；接入后 MCP 工具将经由同一 `ToolRegistry` 暴露。

如需扩展工具或接入 MCP，参考 [Package 职责详解](../architecture/packages.md) 的 `plugins` / `mcp` 章节与 [快速开始](../guide/getting-started.md)。
