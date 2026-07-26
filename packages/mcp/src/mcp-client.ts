import { type ChildProcessWithoutNullStreams, spawn } from 'node:child_process';
import { McpError, type ToolDef, type ToolParam, type ToolResult } from '@vynth/core';

/** MCP stdio JSON-RPC 协议版本（F12 锁定 2024-11-05，见实施开发计划 §2.3 / 风险 U-05） */
export const MCP_PROTOCOL_VERSION = '2024-11-05';

interface JsonRpcReq {
  jsonrpc: '2.0';
  id: number;
  method: string;
  params?: unknown;
}

interface JsonRpcRes {
  jsonrpc: '2.0';
  id: number;
  result?: unknown;
  error?: { message: string };
}

/** MCP `tools/list` 返回的单个工具定义（2024-11-05 子集） */
export interface McpTool {
  name: string;
  description?: string;
  inputSchema?: {
    type?: string;
    properties?: Record<string, { type?: string; description?: string }>;
    required?: string[];
  };
}

type McpCall = (name: string, args: Record<string, unknown>) => Promise<ToolResult>;

/**
 * 将 MCP 工具定义转换为 engine 的 `ToolDef`，使 MCP 工具能并入 agent 工具集（F12）。
 * `call` 由 `McpClient` 注入，绑定到对应服务器的 `tools/call`，从而复用同一套沙箱/审计链路。
 */
export function mcpToolToToolDef(tool: McpTool, call: McpCall): ToolDef {
  const schema = tool.inputSchema ?? {};
  const props = schema.properties ?? {};
  const required = new Set(schema.required ?? []);
  const parameters: ToolParam[] = Object.entries(props).map(([name, p]) => ({
    name,
    type: mapJsonType(p?.type),
    description: p?.description ?? '',
    required: required.has(name)
  }));
  return {
    name: tool.name,
    description: tool.description ?? '',
    parameters,
    run: (args) => call(tool.name, args)
  };
}

function mapJsonType(t?: string): ToolParam['type'] {
  switch (t) {
    case 'number':
    case 'integer':
      return 'number';
    case 'boolean':
      return 'boolean';
    default:
      return 'string';
  }
}

export class McpClient {
  private proc: ChildProcessWithoutNullStreams | null = null;
  private buf = '';
  private nextId = 1;
  private pending = new Map<number, (res: JsonRpcRes) => void>();
  private readonly tools = new Map<string, McpTool>();

  constructor(
    private readonly command: string,
    private readonly args: string[] = []
  ) {}

  async connect(): Promise<void> {
    this.proc = spawn(this.command, this.args, { stdio: ['pipe', 'pipe', 'inherit'] });
    this.proc.stdout.on('data', (d) => this.onData(String(d)));
    // 服务器异常退出时，让所有挂起请求失败，避免调用方永久挂起
    this.proc.on('exit', (code) => {
      if (code === null || code === 0) return;
      for (const resolve of this.pending.values()) {
        resolve({ jsonrpc: '2.0', id: -1, error: { message: `mcp server exited (${code})` } });
      }
      this.pending.clear();
    });
    const res = await this.rpc('initialize', {
      protocolVersion: MCP_PROTOCOL_VERSION,
      capabilities: {},
      clientInfo: { name: 'vynth', version: '0.1.0' }
    });
    if (res.error) throw new McpError(`initialize failed: ${res.error.message}`, 'VC-060001');
    const listRes = await this.rpc('tools/list', {});
    if (listRes.error)
      throw new McpError(`tools/list failed: ${listRes.error.message}`, 'VC-060001');
    const list = (listRes.result as { tools?: McpTool[] })?.tools ?? [];
    for (const t of list) this.tools.set(t.name, t);
  }

  /** 返回并入 agent 工具集所需的 `ToolDef[]`（每个 MCP 工具绑定到本客户端的 tools/call） */
  getToolDefs(): ToolDef[] {
    return [...this.tools.values()].map((t) => mcpToolToToolDef(t, (n, a) => this.callTool(n, a)));
  }

  async callTool(name: string, args: Record<string, unknown>): Promise<ToolResult> {
    const res = await this.rpc('tools/call', { name, arguments: args });
    if (res.error) return { ok: false, output: '', error: res.error.message };
    const content = (res.result as { content?: Array<{ text?: string }> })?.content ?? [];
    return { ok: true, output: content.map((c) => c.text ?? '').join('\n') };
  }

  close(): void {
    this.proc?.kill();
    this.proc = null;
  }

  private rpc(method: string, params: unknown): Promise<JsonRpcRes> {
    const proc = this.proc;
    if (!proc) throw new McpError('not connected', 'VC-060002');
    const id = this.nextId++;
    const req: JsonRpcReq = { jsonrpc: '2.0', id, method, params };
    return new Promise((resolve) => {
      this.pending.set(id, resolve);
      proc.stdin.write(`${JSON.stringify(req)}\n`);
    });
  }

  private onData(chunk: string): void {
    this.buf += chunk;
    const lines = this.buf.split('\n');
    this.buf = lines.pop() ?? '';
    for (const raw of lines) {
      const line = raw.trim();
      if (!line) continue;
      let msg: JsonRpcRes;
      try {
        msg = JSON.parse(line);
      } catch {
        continue;
      }
      const handler = this.pending.get(msg.id);
      if (handler) {
        this.pending.delete(msg.id);
        handler(msg);
      }
    }
  }
}
