# 插件开发

Vynth 支持通过 `-p/--plugin` 加载本地 TypeScript 插件，动态扩展 agent 工具集。

---

## 插件结构

一个最小插件包含两个导出：

```typescript
// my-plugin.ts
export const pluginName = 'my-plugin';

export function activate(reg: ToolRegistry): void {
  // 注册自定义工具
  reg.add({
    name: 'hello',
    description: '向世界问好',
    schema: {
      type: 'object',
      properties: {},
      required: [],
    },
    execute: async () => {
      return { ok: true, output: 'Hello, Vynth!' };
    },
  });
}
```

---

## 完整示例

```typescript
// packages/plugins/examples/hello-plugin.ts
import type { ToolRegistry, ToolDefinition } from '@vynth/engine';

export const pluginName = 'hello-world';

export function activate(reg: ToolRegistry): void {
  const tool: ToolDefinition = {
    name: 'hello',
    description: '向指定对象问好',
    schema: {
      type: 'object',
      properties: {
        name: {
          type: 'string',
          description: '问候对象',
        },
      },
      required: ['name'],
    },
    execute: async (args: Record<string, unknown>) => {
      const name = args.name as string;
      return { ok: true, output: `Hello, ${name}!` };
    },
  };

  reg.add(tool);
}
```

---

## 工具定义规范

### ToolDefinition 接口

```typescript
interface ToolDefinition {
  name: string;           // 工具名，唯一标识
  description: string;    // 工具描述（LLM 用于判断何时调用）
  schema: {               // JSON Schema，描述参数结构
    type: 'object';
    properties: Record<string, {
      type: string;
      description?: string;
    }>;
    required?: string[];
  };
  execute: (args: Record<string, unknown>) => Promise<ToolResult>;
}
```

### ToolResult 结构

```typescript
interface ToolResult {
  ok: boolean;      // 是否成功
  output: string;   // 成功时的输出
  error?: string;   // 失败时的错误信息（可选）
}
```

> **设计原则**：工具执行不抛异常，而是通过 `ok: false` 回传错误。agent 根据 `ToolResult` 决定下一步行动。

---

## 加载与生命周期

### 加载方式

```bash
# 无头模式加载插件（脚本中 -p 即视为已授权，直接加载）
./dist/vynth -g '使用 hello 工具问好' -p ./my-plugin.ts

# 交互 TUI 加载插件（启动后弹出信任确认，确认后才加载）
./dist/vynth -p ./my-plugin.ts
```

### 生命周期

1. **发现**：CLI 解析 `-p/--plugin` 参数
2. **动态导入**：`loadPlugin(path)` 执行 `import(path)`
3. **校验**：检查 `pluginName`（字符串）和 `activate`（函数）导出
4. **激活**：调用 `activate(toolRegistry)` 注册工具
5. **运行**：agent 循环可通过 `tools.list()` 发现新工具

### 批量加载

```typescript
import { loadAll } from '@vynth/plugins';

const plugins = await loadAll([
  './plugins/hello.ts',
  './plugins/code-review.ts',
]);

// plugins = ['hello-world', 'code-review']
```

---

## 信任确认（F13 · 信任模型联动）

插件在本进程内执行任意代码，拥有与 Vynth 同等的权限。因此加载插件必须受信任门禁约束：

- **无头模式（`-g ... -p <路径>`）**：`-p` 是脚本/管道中的显式授权，启动即加载，不做交互确认。
- **交互 TUI（`-p <路径>`）**：进入 TUI 后会**在 `import` 之前**弹出信任确认，展示插件路径与信任边界警告，需用户输入 `y`/`yes` 才真正加载；输入其他（`n`/回车）则拒绝，插件代码不会被执行。

底层由 `@vynth/plugins` 的 `loadPluginsWithTrust(paths, reg, confirm)` 实现：确认回调返回 `true` 才 `import` + `activate`，故门禁在任意代码执行前生效。

```typescript
import { loadPluginsWithTrust } from '@vynth/plugins';

// TUI：确认回调弹出交互式信任提示
const res = await loadPluginsWithTrust(paths, tools, async ({ path }) => askUserTrust(path));
// res = { loaded: string[], declined: string[], errors: { path, error }[] }
```

---

## 信任边界

> ⚠ **安全警告**：插件在当前进程中执行**任意代码**，拥有与 Vynth 同等的权限。

- 文件系统：可读写 cwd 内任意文件（受 `safeResolve` 守卫约束）
- 环境���量：可访问 `VYNTH_API_KEY` 等敏感变量
- 网络：可发起任意出站请求（受 `VYNTH_NET` 开关约束）
- 命令执行：可调用 `run_shell` 执行宿主命令

**仅加载你完全信任的插件**。恶意插件可窃取凭���或破坏系统。

---

## 调试技巧

### 打印工具调用

```typescript
execute: async (args) => {
  console.error('[DEBUG] tool called with:', args);
  return { ok: true, output: '...' };
}
```

### 模拟 LLM 响应

使用本地 mock 服务器：

```bash
# 启动 mock LLM
bun run scripts/mock-llm.ts

# 指向 mock
VYNTH_LLM_BASE_URL=http://localhost:8787 ./dist/vynth -g '测试插件'
```

---

## 常见问题

**Q: 插件可以调用其他插件吗？**

A: 可以。插件注册的工具统一进入 `ToolRegistry`，agent 可按需调用。

**Q: 插件可以修改配置吗？**

A: 当前不支持。配置仅通过环境变量注入，插件无法修改 `loadConfig` 返回值。

**Q: 插件支持热重载吗？**

A: 不支持。每次运行需重新加载。

---

## 相关文档

- [API 参考](api/overview.md) —— CLI 参数与退出码
- [架构总览](architecture/index.md) —— 插件在系统中的位置
- [开发规范](development/dev-guide.md) —— 安全红线与冻结值
