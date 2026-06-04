# ADR-002: Workspace 多 Crate 结构

- **状态**：Accepted
- **日期**：2026-01-15
- **作者**：Synerix 架构团队

## 背景

初始原型将所有代码放在单个 crate 中。随着功能增长（MCP 客户端、Sandbox、Skills、多 Agent 协作），单 crate 结构暴露出以下问题：

1. **编译级联**：修改任何模块都触发全量重编
2. **依赖混乱**：测试工具（criterion）和生产依赖混在同一 Cargo.toml
3. **API 边界模糊**：`pub` 函数无处不在，无模块间契约
4. **复用困难**：核心类型（Role、ChatMessage）无法被外部工具或未来服务端复用

## 决策

采用 Cargo Workspace 多 crate 结构，分为三个成员：

```
synerix/
├── crates/
│   ├── synerix/            # 主应用 — TUI + 业务逻辑
│   │   ├── src/            # 15 个领域模块
│   │   └── Cargo.toml
│   └── synerix-core/       # 核心类型库 — 无运行时依赖
│       ├── src/
│       │   ├── types/      # 共享类型 (Role, ChatMessage 等)
│       │   ├── utils/      # 工具函数 (datetime, sync)
│       │   └── token_estimator.rs
│       └── Cargo.toml
├── benches/
│   └── bench_runner/       # 性能基准 (criterion)
└── Cargo.toml              # Workspace 根
```

### 依赖方向

```
synerix (主应用)  ──depends on──▶  synerix-core
                                         │
bench_runner  ──depends on──▶  synerix-core
                                         │
                synerix-core 不会反向依赖任何 crate
```

### 共享配置

所有 crate 共享统一的 `[workspace.package]` 版本号（v0.1.1）、edition（2021）、license（MIT）。

## 后果

### 正面

- **增量编译**：修改 core 只需重编 core + 主 crate 的依赖层，不影响 bench_runner
- **关注点分离**：core 只导出纯数据类型，无 runtime 依赖（无 tokio、无 reqwest）
- **复用潜力**：未来构建 synerix-server 可以直接依赖 synerix-core
- **测试隔离**：core 的测试不依赖主 crate 的异步运行时，运行更快

### 负面

- **依赖管理复杂度**：依赖项需同时出现在 workspace 和 crate 级 Cargo.toml
- **类型搬家**：曾经简单的 `use crate::llm::types::ChatMessage` 变成了跨 crate 引用
- **初始设计成本**：拆分哪些模块进 core 需要经验判断，过度拆分会导致碎片化

## 备选方案

| 方案 | 放弃原因 |
|------|----------|
| 单 crate | 编译级联无解，类型复用受限 |
| monorepo 无 workspace | cargo test 无法并行，缺乏共享版本控制 |
| 先 mono 再拆分 | 迁移成本高，API 已耦合；早期拆分更经济 |