---
name: P1 - PluginManager::emit_event 短路 Bug
about: emit_event 注释和实现矛盾，单个插件失败会阻断后续插件
title: '[P1] PluginManager::emit_event short-circuits on first plugin error'
labels: bug, P1
assignees: ''
---

## 描述

`plugins/manager.rs` 中 `emit_event` 方法的注释声明 "do NOT short-circuit on individual plugin errors"，但实际实现却在第一个错误处 `return Err(e)` 短路，导致后续插件收不到事件。

## 当前行为

```rust
// 注释说不要短路，但代码实现了短路
for plugin in &self.plugins {
    if let Err(e) = plugin.on_event(event).await {
        tracing::warn!("Plugin error during event: {}", e);
        return Err(e);  // ❌ 短路！后续插件收不到
    }
}
```

## 期望行为

收集所有插件的错误，全部处理完再返回聚合错误：

```rust
let mut errors = Vec::new();
for plugin in &self.plugins {
    if let Err(e) = plugin.on_event(event).await {
        tracing::warn!("Plugin '{}' failed: {:?}", plugin.name(), e);
        errors.push(e);
    }
}
// 返回聚合错误或 Ok
```

## 需要变更

1. `plugins/manager.rs` - 修复 emit_event 实现
2. `error.rs` - 新增 `PluginEventPartialFailure` 变体（可选，或用 Vec<AppError>）
3. 更新相关测试用例

## 影响范围

插件系统的事件分发可靠性。单个插件崩溃不应当影响其他插件的正常运行。
