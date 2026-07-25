# 测试指南

Vynth 使用 `bun test` 作为测试框架，覆盖单元测试、集成测试与端到端测试。

---

## 测试策略

| 测试类型 | 框架 | 位置 | 说明 |
|----------|------|------|------|
| 单元测试 | `bun:test` | `packages/*/src/*.test.ts` | 单包级测试，mock 外部依赖 |
| 集成测试 | `bun:test` | `packages/harness/src/e2e.test.ts` | 驱动真实 CLI 二进制 |
| 基准测试 | 自定义脚本 | `scripts/bench-*.ts` | 冷启动、体积等非功能指标 |

---

## 运行测试

```bash
# 运行所有测试
bun test packages

# 运行特定包
bun test packages/core
bun test packages/engine

# 监听模式
bun test --watch packages

# 覆盖率（bun 内置）
bun test --coverage packages
```

---

## 单元测试示例

### 配置加载测试

```typescript
// packages/core/src/core.test.ts
import { loadConfig } from './config';

describe('loadConfig 默认值', () => {
  it('默认模型 deepseek-v4-pro', () => {
    const c = loadConfig();
    expect(c.model).toBe('deepseek-v4-pro');
  });
});
```

### 沙箱守卫测试

```typescript
// packages/sandbox/src/sandbox.test.ts
import { safeResolve } from './sandbox';

describe('safeResolve 越界守卫', () => {
  it('拒绝 ../ 路径穿越', () => {
    expect(() => safeResolve('/cwd', '../etc/passwd')).toThrow();
  });
});
```

---

## 集成测试示例

集成测试通过 `execFileSync` 驱动真实编译后的二进制：

```typescript
// packages/harness/src/e2e.test.ts
import { execFileSync } from 'node:child_process';

it('CLI 无头模式输出', () => {
  const out = execFileSync(process.execPath, [
    'apps/cli/src/main.ts',
    '-g', 'hello'
  ], { env: { ...process.env, VYNTH_API_KEY: '' } });
  
  expect(out.toString()).toContain('Hello');
});
```

> **坑**：bun 的 `spawnSync.exitCode` 在子进程是 bun 时返回 `undefined`，改用 `execFileSync` 经 `.status` 取码。

---

## 基准测试

### 冷启动 P95

```bash
bun run scripts/bench-cold-start.ts
```

输出：

```
采样 1: 28.3ms
采样 2: 31.2ms
...
P50 = 29.1ms
P95 = 30.5ms
max = 817ms（偶发磁盘冷读）
```

### 体积门禁

```bash
bun run scripts/check-binary-size.ts
```

输出：

```
当前体积: 60.51 MB
上限（MVP）: 61 MB
状态: ✅ PASS
```

---

## 测试约定

### 文件命名

- 单元测试：`*.test.ts`（与源码同目录）
- 集成测试：`e2e.test.ts`（harness 包）

### 测试结构

```typescript
describe('模块名', () => {
  describe('子功能', () => {
    it('具体行为', () => {
      // arrange
      // act
      // assert
    });
  });
});
```

### Mock 策略

- **外部依赖**：mock LLM 请求（使用 `scripts/mock-llm.ts`）
- **文件系统**：使用临时目录（`tmpdir()`）
- **环境变量**：`beforeEach` 保存，`afterEach` 恢复

---

## 覆盖率目标

| 包 | 当前覆盖率 | 目标 |
|----|-----------|------|
| `@vynth/core` | ~95% | ≥ 90% |
| `@vynth/engine` | ~85% | ≥ 80% |
| `@vynth/sandbox` | ~90% | ≥ 85% |
| `@vynth/plugins` | ~80% | ≥ 75% |
| `@vynth/tui` | ~70% | ≥ 60% |

> TUI 渲染层（ansi-escapes 集成）难以单元测试，以集成测试补充。

---

## 持续集成

PR 合并前自动运行：

1. `bun test packages` —— 58 例全绿
2. `bun run lint` —— 0 error
3. `bun run compile` —— 单二进制成功
4. `bun run check-binary-size` —— ≤ 61MB
5. `gitleaks` —— 无密钥泄露

---

## 相关文档

- [开发规范](../development/dev-guide.md) —— 分支模型与代码规范
- [贡献指南](../development/contributing.md) —— PR 流程与行为准则
- [架构总览](../architecture/index.md) —— 模块关系与数据流
