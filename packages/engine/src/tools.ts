import { type ToolDef, ToolError, type ToolResult } from '@zeno/core';
import * as sandbox from '@zeno/sandbox';

export class ToolRegistry {
  private tools = new Map<string, ToolDef>();

  register(tool: ToolDef): void {
    if (this.tools.has(tool.name)) throw new ToolError(`duplicate tool: ${tool.name}`);
    this.tools.set(tool.name, tool);
  }

  get(name: string): ToolDef | undefined {
    return this.tools.get(name);
  }

  list(): ToolDef[] {
    return [...this.tools.values()];
  }

  async run(name: string, args: Record<string, unknown>): Promise<ToolResult> {
    const tool = this.tools.get(name);
    if (!tool) return { ok: false, output: '', error: `unknown tool: ${name}` };
    try {
      return await tool.run(args);
    } catch (err) {
      return { ok: false, output: '', error: err instanceof Error ? err.message : String(err) };
    }
  }
}

export function builtinTools(
  cwd: string,
  opts: { networkAllowed?: boolean; harden?: boolean } = {}
): ToolRegistry {
  const reg = new ToolRegistry();
  reg.register({
    name: 'read_file',
    description: '读取文件内容（沙箱内）',
    parameters: [{ name: 'path', type: 'string', description: '相对或绝对路径', required: true }],
    run: async (args) => {
      const path = String(args.path ?? '');
      return sandbox.readText(path, cwd);
    }
  });
  reg.register({
    name: 'write_file',
    description: '写入文件内容（沙箱内）',
    parameters: [
      { name: 'path', type: 'string', description: '目标路径', required: true },
      { name: 'content', type: 'string', description: '文件内容', required: true }
    ],
    run: async (args) => sandbox.writeText(String(args.path ?? ''), String(args.content ?? ''), cwd)
  });
  reg.register({
    name: 'run_shell',
    description: '在沙箱中执行 shell 命令',
    parameters: [{ name: 'command', type: 'string', description: '要执行的命令', required: true }],
    run: async (args) =>
      sandbox.runCommand(String(args.command ?? ''), {
        cwd,
        networkAllowed: opts.networkAllowed ?? true,
        harden: opts.harden
      })
  });
  reg.register({
    name: 'list_files',
    description: '浏览项目目录树（沙箱内，自动跳过 node_modules/.git/dist 等噪音目录）',
    parameters: [
      { name: 'path', type: 'string', description: '目录路径（默认项目根）', required: false },
      { name: 'depth', type: 'number', description: '最大递归深度（默认 4）', required: false }
    ],
    run: async (args) =>
      sandbox.listFiles(
        String(args.path ?? '.'),
        cwd,
        args.depth !== undefined ? Number(args.depth) : 4
      )
  });
  reg.register({
    name: 'grep_search',
    description: '全局内容搜索（沙箱内）。支持字面量子串或正则，返回 path:line 命中列表',
    parameters: [
      { name: 'pattern', type: 'string', description: '搜索模式', required: true },
      {
        name: 'regex',
        type: 'boolean',
        description: '是否按正则解析（默认字面量）',
        required: false
      },
      {
        name: 'caseSensitive',
        type: 'boolean',
        description: '大小写敏感（默认否）',
        required: false
      },
      { name: 'include', type: 'string', description: '扩展名过滤，如 .ts', required: false }
    ],
    run: async (args) =>
      sandbox.grepSearch(String(args.pattern ?? ''), cwd, {
        regex: Boolean(args.regex),
        caseSensitive: Boolean(args.caseSensitive),
        include: args.include !== undefined ? String(args.include) : undefined
      })
  });
  reg.register({
    name: 'create_file',
    description: '创建新文件（含父目录）。文件已存在时拒绝，覆盖请用 write_file',
    parameters: [
      { name: 'path', type: 'string', description: '目标路径', required: true },
      { name: 'content', type: 'string', description: '文件内容', required: true }
    ],
    run: async (args) =>
      sandbox.createFile(String(args.path ?? ''), String(args.content ?? ''), cwd)
  });
  return reg;
}
