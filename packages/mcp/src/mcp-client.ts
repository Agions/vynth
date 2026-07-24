import { type ChildProcessWithoutNullStreams, spawn } from 'node:child_process';
import { McpError, type ToolResult } from '@vynth/core';

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

export class McpClient {
  private proc: ChildProcessWithoutNullStreams | null = null;
  private buf = '';
  private nextId = 1;
  private pending = new Map<number, (res: JsonRpcRes) => void>();
  readonly tools = new Map<string, unknown>();

  constructor(
    private readonly command: string,
    private readonly args: string[] = []
  ) {}

  async connect(): Promise<void> {
    this.proc = spawn(this.command, this.args, { stdio: ['pipe', 'pipe', 'inherit'] });
    this.proc.stdout.on('data', (d) => this.onData(String(d)));
    const res = await this.rpc('initialize', {
      protocolVersion: '2024-11-05',
      capabilities: {},
      clientInfo: { name: 'vynth', version: '0.1.0' }
    });
    if (res.error) throw new McpError(res.error.message);
    await this.rpc('tools/list', {}).then((r) => {
      const list = (r.result as { tools?: Array<{ name: string }> })?.tools ?? [];
      for (const t of list) this.tools.set(t.name, t);
    });
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
    if (!proc) throw new McpError('not connected');
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
