# ADR-003: LLM 适配器使用 Trait Object

- **状态**：Accepted
- **日期**：2026-02-10
- **作者**：Synerix 架构团队

## 背景

Synerix 需要支持多个 LLM 提供商（DeepSeek V4、MiMo v2.5、OpenAI 兼容 API、自定义模型）。每个提供商在以下方面存在差异：

1. API 端点格式（路径、请求体结构）
2. 认证方式（API Key header、Bearer token）
3. 流式格式（SSE、WebSocket）
4. Token 计数精度（tiktoken vs 启发式）
5. 模型参数（context window、温度、top_p）

需要一种统一的方式来抽象这些差异，同时：
- 运行时可选择提供商（运行时多态）
- 无需在类型系统层面枚举所有提供商
- 新提供商无需修改核心框架代码

## 决策

使用 **Trait Object**（`Box<dyn LlmAdapter>`）而非泛型约束（`impl LlmAdapter`）来实现 LLM 适配器。

```rust
#[async_trait::async_trait]
pub trait LlmAdapter: Send + Sync {
    async fn chat(&self, messages: &[ChatMessage], tools: &[ToolSchema])
        -> Result<ChatResponse, AppError>;
    async fn chat_stream(&self, messages: &[ChatMessage], tools: &[ToolSchema])
        -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk, AppError>> + Send>>, AppError>;
    fn model_id(&self) -> &str;
    fn context_window(&self) -> usize;
}
```

### 运行时用法

```rust
// 注册管理器（运行时多态）
let adapter: Box<dyn LlmAdapter> = match config.provider.as_str() {
    "deepseek" => Box::new(OpenAICompatAdapter::new("deepseek", config)),
    "mimo"     => Box::new(OpenAICompatAdapter::new("mimo", config)),
    "custom"   => Box::new(CustomAdapter::new(config)),
};
let result = adapter.chat(&messages, &tools).await?;
```

### 为何不是泛型

如果使用泛型：

```rust
pub async fn run_agent<A: LlmAdapter>(adapter: &A, ...) { ... }
```

- 所有使用 `LlmAdapter` 的函数都需要泛型参数，导致类型签名传染
- 无法将不同提供商放入同一个 `Vec` 或 `HashMap`
- 在 Agent 配置中动态切换提供商需要额外模式（如 enum dispatch）

## 后果

### 正面

- **运行时自由度**：配置文件中指定 `provider = "deepseek"` 即可在运行时切换
- **零侵入扩展**：新增提供商只需实现 trait，无需修改核心调用链
- **动态集合**：多 Agent 场景下可以为不同 Agent 分配不同提供商
- **测试友好**：`MockLlm` 实现 `LlmAdapter` 即可注入测试

### 负面

- **vtable 间接调用**：每次方法调用多一层指针解引用（微秒级，可忽略）
- **堆分配开销**：`Box<dyn LlmAdapter>` 需要堆分配（启动时一次，后续不复用）
- **动态分派限制**：无法在编译期进行特化优化（如内联 `count_tokens`）
- **对象安全约束**：trait 方法不能有泛型参数或返回 `Self`（本 trait 已满足）

## 备选方案

| 方案 | 放弃原因 |
|------|----------|
| 泛型 `<A: LlmAdapter>` | 类型签名感染所有上游函数；无法在运行时切换；多 Agent 场景无法用 Vec 管理 |
| Enum 分发 `enum LlmProvider { DeepSeek, MiMo, ... }` | 新增提供商需修改核心枚举定义，违反开闭原则；match 语句繁琐 |
| 插件系统动态加载 | 复杂度太高，q 初期不需要动态链接加载；trait object 足够 |