---
name: code-review
description: 代码审查技能
trigger:
  auto_match:
    keywords: [review, 审查, code quality]
    threshold: 0.5
required_tools: [file_read, search]
---

## 代码审查指南
当审查代码时，关注以下方面：
1. 安全性 — SQL注入、XSS、CSRF
2. 性能 — 不必要的分配、O(n²) 算法
3. 可维护性 — 命名、注释、复杂度
