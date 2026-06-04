# ADR-004: MCP 协议使用 JSON-RPC 2.0

- **状态**：Accepted
- **日期**：2026-02-20
- **作者**：Synerix 架构团队

## 背景

Synerix 需要通过 MCP（Model Context Protocol）与外部工具通信。MCP 协议需要满足：

1. **语言无关**：MCP 服务器可能由 Python、Rust、Go、TypeScript 等语言实现
2. **传输灵活**：支持本地进程通信（stdio）和远程服务（HTTP/SSE）
3. **请求-响应语义**：工具调用需要明确的 request/response 配对，支持超时和取消
4. **流式通知**：服务端可以主动推送状态更新
5. **类型安全**：工具参数需要有 Schema 约束（JSON Schema）

## 决策

采用 **JSON-RPC 2.0** 作为 MCP 协议的序列化格式，支持 **stdio** 和 **HTTP** 两种传输方式。

### 协议结构

```rust
// JSON-RPC 2.0 Request
pub struct JsonRpcRequest {
    pub jsonrpc: String,       // 固定 "2.0"
    pub id: u64,               // 请求 ID（用于配对 response）
    pub method: String,        // 方法名（如 "tools/call"）
    pub params: Option<Value>, // 参数（JSON object）
}
```

### 传输层设计

```
┌─────────────────────────────────────┐
│          Application Layer          │
│  JSON-RPC 2.0 (Request/Response)    │
├─────────────────────────────────────┤
│         Transport Layer             │
│  StdioTransport | HttpTransport     │
├─────────────────────────────────────┤
│         Connection Layer            │
│  MCP Client (connection pool)       │
└─────────────────────────────────────┘
```

- **StdioTransport**：通过子进程 stdin/stdout 通信，一行一个 JSON-RPC 消息，`\n` 分隔
- **HttpTransport**：通过 HTTP POST 发送请求，SSE 接收流式响应

### 为什么不使用 MessagePack/GrPC/自定义协议

1. **JSON-RPC 2.0** 是 MCP 规范的原生选择，所有 MCP SDK 均支持
2. 人类可读的调试体验：`curl` 可以直接发送请求
3. serde_json 序列化性能足够（单次调用微秒级）
4. MessagePack 的二进制压缩在终端通信场景节约有限（协议 header 占主导）

## 后果

### 正面

- **标准兼容**：完全遵循 MCP 规范，可与任意语言的 MCP 服务器互操作
- **零额外依赖**：serde_json 已在依赖树中，MessagePack 需要额外引入 rmp-serde
- **调试友好**：`tools/list` 和 `tools/call` 的 JSON 输出可直接用 jq 分析
- **传输透明**：stdio 和 HTTP 两种传输对上层应用层完全透明

### 负面

- **无 Schema 验证**：JSON-RPC 2.0 本身不强制参数验证，需在 MCP 应用层实现
- **无双向流原生支持**：服务端推送依赖 SSE 或 polling，不如 gRPC 的 bidi streaming 优雅
- **JSON 解析开销**：在极高吞吐场景（1000+ calls/sec）JSON 解析可能成为瓶颈

## 备选方案

| 方案 | 放弃原因 |
|------|----------|
| MessagePack | MCP 规范非原生支持；调试需额外工具；serde_json 性能已足够 |
| gRPC + Protobuf | 太重，需要 protoc 编译器和代码生成；MCP 社区不使用 |
| 自定义二进制协议 | 不兼容 MCP 生态；每个 MCP 服务器需实现自定义协议适配器 |