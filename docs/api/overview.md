# API 总览（CLI）

Zeno 的对外接口是单二进制 `dist/zeno`（bin 名 `zeno`）。以下为完整 CLI 表面，与 `apps/cli/src/main.ts` 同源。

## 参数表

| 参数 | 等价长名 | 作用 | 备注 |
| --- | --- | --- | --- |
| （无参数） | — | 启动交互式 TUI | 需真实 TTY；非 TTY 环境退出码 `2` |
| `-g <目标>` | `--goal <目标>` | 无头 agent 模式，流式输出到 stdout | 无需 TTY；需 `ZENO_API_KEY` |
| `-m <mode>` | `--mode <mode>` | 指定模式 `plan` \| `vibe` | 覆盖 `ZENO_MODE`，默认 `vibe` |
| `-p <路径>` | `--plugin <路径>` | 加载本地插件 | 无头模式直接加载；TUI 模式弹出信任确认 |
| `-s "<命令>"` | `--mcp "<命令>"` | 接入 MCP server（stdio JSON-RPC 2024-11-05） | 可重复指定多个；工具自动并入 agent 工具集 |
| `-v` | `--version` | 输出版本号（`0.1.1`）并退出 | 退出码 0 |
| `-h` | `--help` | 输出用法说明并退出 | 退出码 0 |

用法示例：

```bash
zeno                                          # 交互 TUI
zeno -g '给当前目录写个 README'                # 无头 agent
zeno -m plan -g '重构 src 下的工具函数'        # 指定 plan 模式
zeno -p ./my-plugin.ts                        # TUI + 插件（信任确认）
zeno -g '查天气' -s "npx -y @modelcontextprotocol/server-xxx"   # MCP
```

## 配置来源与优先级

**命令行参数 > 环境变量 > 配置文件**。

### 环境变量

| 变量 | 作用 | 默认值 |
| --- | --- | --- |
| `ZENO_API_KEY` | LLM API key（**必填**） | 空 |
| `ZENO_MODEL` | 模型名 | `deepseek-v4-pro` |
| `ZENO_LLM_BASE_URL` | OpenAI 兼容端点 | `https://api.deepseek.com/v1` |
| `ZENO_MODE` | `plan` \| `vibe` | `vibe` |
| `ZENO_THEME` | `mocha` / `latte` / `midnight` / `forest` / `light` / `neon` | `mocha` |
| `ZENO_NET` | 联网开关（`'0'` = 禁网） | 开启 |
| `ZENO_HARDEN` | OS 级硬隔离（`'1'` = 开启，不可用时 Fail-Closed） | 关闭 |
| `ZENO_AUDIT` | 5 维审计日志（`'1'` = 开启） | 关闭 |
| `ZENO_DATA_DIR` | 数据目录 | `~/.zeno` |
| `ZENO_REPOMAP` | 仓库地图（`'0'` = 关闭） | 开启 |

### 配置文件

- `~/.zeno/config.json` — 全局（TUI 内 `/model` `/config` 的写入目标）
- 项目根 `zeno.json` / `.zenorc` — 项目级，可配置 `mode` / `model` / `theme` / `sandbox` 等非敏感项

> **API Key 禁止写入项目配置文件**，只走环境变量或 `~/.zeno/config.json`。详见 [配置详解](../guide/configuration.md)。

## 退出码

| 退出码 | 含义 |
| --- | --- |
| `0` | 正常结束（`--version` / `--help` / TUI 正常退出 / 无头任务完成） |
| `2` | 用法错误（参数非法 / 非 TTY 启动 TUI / 必要输入缺失） |
| 非 0（≠2） | 运行期错误（LLM 调用失败、工具执行失败等） |

## 错误码体系

运行期错误统一带 `VC-XXXXXX` 错误码（如 `VC-010003` 未知参数、`VC-030006` 硬隔离不可用、`VC-060003` MCP 命令为空），TUI 中失败的工具块会附带错误码与可操作建议。

相关阅读：[TUI 使用指南](../guide/tui.md) · [Package 职责](../architecture/packages.md) · [详细 API 参考](reference.md)
