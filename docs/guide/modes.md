# Coding Modes / 编程模式

Synerix adapts to how you work. Choose the mode that fits the task.

Synerix 根据你的工作方式自适应。选择适合当前任务的模式。

## Act Mode / 执行模式

**Direct execution. No context switches. / 直接执行，无需上下文切换。**

Run commands, build projects, run tests, and manage files — all inside the terminal with AI-assisted safety.

运行命令、构建项目、运行测试和管理文件 —— 全部在终端内完成，并借助 AI 安全保障。

| Feature / 特性 | Detail / 详情 |
|---|---|
| Sandbox / 沙箱 | Auto-approves safe ops, previews risky ones / 自动批准安全操作，预览风险操作 |
| Commands / 命令 | Full shell access with AI context / 完整的 shell 访问权限，带 AI 上下文 |
| Use when / 适用场景 | Building, testing, debugging, Git workflows / 构建、测试、调试、Git 工作流 |

```
❯ Build the project and run tests
   ✓ cargo build --release
   ✓ 142 tests passed
```

## Vibe Mode / 沉浸模式

**Immersive flow state. / 沉浸式心流状态。**

Describe what you want. Synerix handles the rest: generate, compile, test, and fix — automatically iterating until it works.

描述你想要什么。Synerix 处理剩下的事：生成、编译、测试和修复 —— 自动迭代直到成功。

| Feature / 特性 | Detail / 详情 |
|---|---|
| Auto-approve / 自动批准 | Low-risk file edits and builds / 低风险文件编辑和构建 |
| Auto-fix / 自动修复 | Compilation errors fed back to AI / 编译错误自动反馈给 AI |
| Use when / 适用场景 | Implementing features, prototyping, refactoring / 实现功能、原型开发、重构 |

```
❯ Add user preferences to the API
   ✓ Code generated
   ✓ cargo check passes
   ✓ Tests pass
   ✓ Done in 1.8s
```

## Chat Mode / 对话模式

**Conversational AI assistant. / 对话式 AI 助手。**

Ask questions, get explanations, brainstorm ideas, and review code — like pairing with a senior engineer.

提出问题、获取解释、头脑风暴和审查代码 —— 就像与高级工程师结对编程一样。

| Feature / 特性 | Detail / 详情 |
|---|---|
| Context aware / 上下文感知 | Understands your codebase / 理解你的代码库 |
| Streaming / 流式响应 | Real-time responses / 实时响应 |
| Use when / 适用场景 | Learning, explaining, debugging, planning / 学习、解释、调试、规划 |

```
❯ Explain how the authorization middleware works
   AI: The middleware sits between the router and handlers...
```

## Architect Mode / 架构模式

**Design and review at scale. / 大规模设计与审查。**

Focus on architecture, module boundaries, design patterns, and long-term maintainability.

关注架构、模块边界、设计模式和长期可维护性。

| Feature / 特性 | Detail / 详情 |
|---|---|
| Analysis / 分析 | Reads full file structure and dependencies / 读取完整文件结构和依赖 |
| Review / 审查 | Code quality, patterns, performance / 代码质量、模式、性能 |
| Use when / 适用场景 | Designing systems, reviews, planning migrations / 系统设计、审查、迁移规划 |

```
❯ Review the auth module for potential improvements
   AI: Consider separating concerns into...
```

## Plan Mode / 规划模式

**Break it down before building. / 先规划再构建。**

Decompose complex tasks into actionable, prioritized steps with estimated effort.

将复杂任务分解为可执行的、有优先级的步骤，并估算工作量。

| Feature / 特性 | Detail / 详情 |
|---|---|
| Decomposition / 分解 | Hierarchical task breakdown / 分层任务分解 |
| Dependencies / 依赖 | Identifies blockers and ordering / 识别阻塞和顺序 |
| Use when / 适用场景 | Large features, migrations, unknowns / 大型功能、迁移、未知问题 |

```
❯ Plan a migration from REST to GraphQL
   1. Schema design (2h)
   2. Query resolver stubs (3h)
   3. Client migration (4h)
   ...
```

## Switching Modes / 切换模式

| Command / 命令 | Mode / 模式 |
|---|---|
| `/mode act` | Act / 执行 |
| `/mode vibe` | Vibe / 沉浸 |
| `/mode chat` | Chat / 对话 |
| `/mode architect` | Architect / 架构 |
| `/mode plan` | Plan / 规划 |

Press `Tab` to cycle through modes without typing.

按 `Tab` 键循环切换模式，无需输入。

## Next Steps / 下一步

- [Configuration](/guide/configuration) — Fine-tune mode behavior / 微调模式行为
- [Troubleshooting](/guide/troubleshooting) — Common issues / 常见问题
