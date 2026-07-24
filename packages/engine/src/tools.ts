import { type ToolDef, ToolError, type ToolResult } from '@vynth/core';
import * as sandbox from '@vynth/sandbox';

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

export function builtinTools(cwd: string, opts: { networkAllowed?: boolean } = {}): ToolRegistry {
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
        networkAllowed: opts.networkAllowed ?? true
      })
  });
  return reg;
}
