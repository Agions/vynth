---
name: P1 - 沙箱命令注入检测缺失
about: risk_classifier 仅做关键词匹配，无法检测命令注入
title: '[P1] Sandbox risk classifier lacks command injection detection'
labels: security, P1
assignees: ''
---

## 描述

`sandbox/risk_classifier.rs` 中的 `classify_command` 仅使用 `starts_with`/`contains` 做关键词匹配，不解析语法。形如 `ls; rm -rf /` 的命令仅匹配到 `ls` 被判定为 Safe。

## 当前风险

- `; rm -rf /` — 命令分隔符注入未被检测
- `| cat /etc/passwd` — 管道注入未被检测
- `` `malicious` `` — 反引号执行未被检测
- `$(malicious)` — 命令替换未被检测

## 修复方案

1. 增加 shell 元字符检测：`;`、`|`、`&`、`` ` ``、`$`、`\n`、`\r`
2. 检测到元字符时提升风险等级至 Critical
3. 考虑添加简单的语法解析来区分误报（如 `echo "safe;"` 不应触发）

## 建议实现

```rust
pub fn detect_injection(command: &str) -> Option<RiskLevel> {
    let injection_chars = [';', '|', '&', '`', '$', '\n', '\r'];
    // 简化检测：检查命令中是否包含元字符（忽略引号内）
    // 更精确的做法需要 shell 语法解析
    for ch in &injection_chars {
        if command.contains(*ch) {
            return Some(RiskLevel::Critical);
        }
    }
    None
}
```

## 相关

- `sandbox/risk_classifier.rs`
- `sandbox/mod.rs`

## 后续

命令注入检测应作为 `classify_command` 流程的第一步，优先级高于关键词匹配。
