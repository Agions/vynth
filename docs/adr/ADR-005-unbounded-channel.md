# ADR-005: Agent 事件通道使用 UnboundedChannel

- **状态**：Accepted
- **日期**：2026-03-05
- **作者**：Synerix 架构团队

## 背景

Synerix 的 Agent 循环需要在以下组件之间传递事件：

1. **Agent Loop**（生产者）：LLM 流式响应的每个 chunk、工具执行的每个中间结果
2. **TUI 渲染层**（消费者）：将事件渲染到终端界面
3. **Plugin 系统**（消费者/生产者）：插件响应事件可能产生新事件
4. **多 Agent 总线**（生产者/消费者）：多 Agent 协作时的消息路由

事件通道需要满足：

- **低延迟**：首 Token 显示延迟 < 50ms
- **高吞吐**：LLM 流式响应可能达到每秒 50+ 个 chunk
- **背压策略**：消费者慢于生产者时如何应对
- **生命周期**：Agent 循环结束后通道自动关闭

## 决策

使用 `tokio::sync::mpsc::unbounded_channel` 作为 Agent 事件通道。

```rust
// 通道建立
let (event_tx, event_rx) = mpsc::unbounded_channel::<AgentEvent>();

// Agent Loop 侧（生产者）
event_tx.send(AgentEvent::StreamChunk(chunk)).ok();

// TUI 侧（消费者）
while let Some(event) = event_rx.recv().await {
    match event {
        AgentEvent::StreamChunk(chunk) => renderer.append(chunk),
        AgentEvent::ToolResult(result) => renderer.show_result(result),
        AgentEvent::Done => break,
    }
}
```

### 为何不使用有界通道

1. **死锁风险**：有界通道 `channel(buffer)` 在 buffer 满时 `send().await` 会阻塞生产者
   - LLM 流式响应在事件循环中发送事件，阻塞 `send` 会阻塞整个 Agent 循环
   - 工具执行可能在等待 TUI 消费事件，而 TUI 在等待工具执行结果 → 循环死锁
2. **复杂度过高**：`try_send` + 丢弃/缓冲策略需要额外逻辑，增加 bug 面
3. **消费者控制**：TUI 渲染层天然有帧率限制，消费速度受终端 I/O 限制，不会被无界通道撑爆

### 内存安全边界

Umbounded 通道在消费者跟不上时的内存增长是可控的：
- 单次 Agent 循环产生的最大事件量 ≈ LLM response 的 chunk 数 + tool call 数
- LLM response 有 max_tokens 限制（默认 4096），对应不超过 4096 个 StreamChunk
- 每个 StreamChunk 约 100 字节 → 单次循环最坏情况 400KB
- 在事件消费完毕前，下一轮循环不会开始（Agent 循环是串行的）

## 后果

### 正面

- **零阻塞**：Agent Loop 发送事件从不等待，保持最小延迟
- **死锁免疫**：消除循环依赖场景下的死锁风险
- **简单可靠**：无背压策略的配置和维护负担
- **通道生命周期**：Agent 循环结束自动 `drop(sender)`，消费者 recv 返回 `None`

### 负面

- **无消费者限流**：极端情况下（LLM 超长输出 + TUI 渲染阻塞）内存增长不可预测
- **缺乏压力反馈**：Agent Loop 不知道 TUI 是否卡住，不会主动减速
- **测试困难**：无界通道无法模拟背压场景的测试

### 缓解措施

1. `max_tokens` 硬限制：LLM response 长度受模型 context window 约束
2. Agent 循环是单线程串行的：不会出现多个循环同时生产事件
3. 监控方案：未来在 `telemetry.rs` 中增加通道 pending 计数告警

## 备选方案

| 方案 | 放弃原因 |
|------|----------|
| `channel(buffer)` 有界通道 | send().await 阻塞 Agent Loop 导致死锁；try_send() 丢失事件 |
| `watch::channel` | 只保留最新值，丢失中间 chunk，不适合流式 LLM 响应 |
| `broadcast::channel` | 多消费者场景需要，但当前架构是单消费者（TUI） |
| 自定义有界 + 丢弃策略 | 复杂度与收益不成正比；Agent Loop 事件量天然有上界 |