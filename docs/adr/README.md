# Architecture Decision Records (ADR)

Synerix 架构决策记录。

| ADR | 标题 | 状态 |
|-----|------|:----:|
| [ADR-001](./ADR-001-rust-ratatui.md) | 技术栈选择：Rust + Ratatui | Accepted |
| [ADR-002](./ADR-002-workspace-structure.md) | Workspace 多 Crate 结构 | Accepted |
| [ADR-003](./ADR-003-llm-adapter-trait-object.md) | LLM 适配器使用 Trait Object | Accepted |
| [ADR-004](./ADR-004-mcp-json-rpc.md) | MCP 协议使用 JSON-RPC 2.0 | Accepted |
| [ADR-005](./ADR-005-unbounded-channel.md) | Agent 事件通道使用 UnboundedChannel | Accepted |

---

### 模板

新 ADR 请使用 [Michael Nygard 格式](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions)：

```markdown
# ADR-NNN: 标题

- 状态：[Proposed | Accepted | Deprecated | Superseded]
- 日期：YYYY-MM-DD
- 作者：

## 背景

## 决策

## 后果

## 备选方案
```