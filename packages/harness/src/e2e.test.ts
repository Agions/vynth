import { describe, expect, test } from 'bun:test';
import { execFileSync } from 'node:child_process';
import { mkdtempSync, rmSync, symlinkSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { resolve } from 'node:path';
import { join } from 'node:path';
import type { ChatMessage, StreamEvent, ToolCall, ToolDef } from '@zeno/core';
import { loadConfig } from '@zeno/core';
import {
  type LLMProvider,
  ToolRegistry,
  builtinTools,
  createProvider,
  runAgent
} from '@zeno/engine';
import { McpClient } from '@zeno/mcp';
import * as samplePlugin from './fixtures/sample-plugin';

class MockProvider implements LLMProvider {
  private calls = 0;
  async *chat(_messages: ChatMessage[], tools: ToolDef[]): AsyncIterable<StreamEvent> {
    this.calls++;
    yield { type: 'token', text: 'plan: ' };
    if (tools.length && this.calls === 1) {
      yield { type: 'tool', call: { id: 'c1', name: tools[0].name, args: { x: 1 } } as ToolCall };
    }
    yield { type: 'done' };
  }
}

test('agent streams tokens and runs a tool', async () => {
  const reg = new ToolRegistry();
  let called = false;
  reg.register({
    name: 'echo_tool',
    description: 'echo',
    parameters: [{ name: 'x', type: 'number', description: 'n', required: true }],
    run: (args) => {
      called = true;
      return { ok: true, output: `got ${args.x}` };
    }
  });

  const events: StreamEvent[] = [];
  let toolName = '';
  for await (const ev of runAgent('echo run', {
    provider: new MockProvider(),
    tools: reg,
    maxSteps: 4
  })) {
    events.push(ev);
    if (ev.type === 'tool') toolName = ev.call.name;
  }

  expect(events.some((e) => e.type === 'token')).toBe(true);
  expect(toolName).toBe('echo_tool');
  expect(called).toBe(true);
});

test('OpenAiProvider parses SSE tokens and tool calls', async () => {
  const sse =
    'data: {"choices":[{"delta":{"content":"hi"}}]}\n' +
    'data: {"choices":[{"delta":{"tool_calls":[{"index":0,"function":{"name":"read_file","arguments":"{\\"path\\":\\"x\\"}"}}]}}]}\n' +
    'data: {"choices":[{"finish_reason":"tool_calls"}]}\n' +
    'data: [DONE]\n';
  const fakeRes = new Response(sse, { status: 200 });
  const orig = globalThis.fetch;
  globalThis.fetch = (async () => fakeRes) as unknown as typeof fetch;
  try {
    const provider = createProvider({ ...loadConfig(), apiKey: 'test' });
    const events: StreamEvent[] = [];
    for await (const ev of provider.chat([{ role: 'user', content: 'go' }], [])) events.push(ev);
    const token = events.find((e) => e.type === 'token');
    const tool = events.find((e) => e.type === 'tool');
    expect(token?.text).toBe('hi');
    expect(tool?.call.name).toBe('read_file');
  } finally {
    globalThis.fetch = orig;
  }
});

test('plugin activates and registers a tool', async () => {
  const reg = new ToolRegistry();
  samplePlugin.activate(reg);
  expect(reg.get('sample_tool')).toBeDefined();
  const r = await reg.run('sample_tool', { x: 2 });
  expect(r.ok).toBe(true);
  expect(r.output).toContain('x=2');
});

test('createProvider throws LlmError when API key is empty (v0.1.0 demo removed)', () => {
  expect(() => createProvider({ ...loadConfig(), apiKey: '' })).toThrow();
  const e = (() => {
    try {
      createProvider({ ...loadConfig(), apiKey: '' });
      return null;
    } catch (err) {
      return err as Error & { code?: string; numericCode?: string };
    }
  })();
  expect(e).not.toBeNull();
  expect(e?.name).toBe('LlmError');
  expect(e?.numericCode).toMatch(/^VC-\d{6}$/);
});

test('sandbox read_file reads within cwd and rejects escape', async () => {
  const dir = mkdtempSync(join(tmpdir(), 'zeno-test-'));
  try {
    writeFileSync(join(dir, 'note.txt'), 'hello-sandbox');
    const reg = builtinTools(dir);
    const ok = await reg.run('read_file', { path: 'note.txt' });
    expect(ok.ok).toBe(true);
    expect(ok.output).toContain('hello-sandbox');

    const escaped = await reg.run('read_file', { path: '../escaped.txt' });
    expect(escaped.ok).toBe(false);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test('sandbox rejects symlink escape', async () => {
  const dir = mkdtempSync(join(tmpdir(), 'zeno-sym-'));
  const outside = mkdtempSync(join(tmpdir(), 'zeno-out-'));
  try {
    writeFileSync(join(outside, 'secret.txt'), 'TOPSECRET');
    symlinkSync(outside, join(dir, 'escape'));
    const reg = builtinTools(dir);
    const r = await reg.run('read_file', { path: 'escape/secret.txt' });
    expect(r.ok).toBe(false);
  } finally {
    rmSync(dir, { recursive: true, force: true });
    rmSync(outside, { recursive: true, force: true });
  }
});

test('VYNTH_NET=off blocks run_shell networking', async () => {
  const dir = mkdtempSync(join(tmpdir(), 'zeno-net-'));
  try {
    const reg = builtinTools(dir, { networkAllowed: false });
    const r = await reg.run('run_shell', { command: 'echo hi' });
    expect(r.ok).toBe(false);
    expect(r.error ?? '').toContain('network blocked');
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test('OpenAiProvider refuses plaintext http to non-local endpoint', async () => {
  expect(() =>
    createProvider({ ...loadConfig(), apiKey: 'x', llmBaseUrl: 'http://evil.example.com/v1' })
  ).toThrow();
});

const REPO = resolve(import.meta.dir, '../../../');
const CLI = resolve(REPO, 'apps/cli/src/main.ts');

function runCli(
  args: string[],
  env: Record<string, string> = { ...process.env, VYNTH_API_KEY: 'test-key-for-cli-tests' }
): { code: number; out: string; err: string } {
  try {
    const out = execFileSync(process.execPath, [CLI, ...args], {
      cwd: REPO,
      env,
      timeout: 30000,
      encoding: 'utf8'
    });
    return { code: 0, out: String(out), err: '' };
  } catch (e) {
    const er = e as { status?: number; stdout?: string; stderr?: string };
    return {
      code: er.status ?? 1,
      out: String(er.stdout ?? ''),
      err: String(er.stderr ?? '')
    };
  }
}

describe('CLI 退出码契约（F11）', () => {
  test('--version 退出 0 且打印版本', () => {
    const r = runCli(['--version']);
    expect(r.code).toBe(0);
    expect(r.out).toContain('0.1.1');
  });

  test('--help 退出 0 且打印用法', () => {
    const r = runCli(['--help']);
    expect(r.code).toBe(0);
    expect(r.out).toContain('terminal');
  });

  test('未知参数 → 退出 2 + 6 位码 + 提示', () => {
    const r = runCli(['--bogus']);
    expect(r.code).toBe(2);
    expect(r.err).toContain('未知参数');
    expect(r.err).toMatch(/VC-\d{6}/);
  });

  test('-g 缺目标值 → 退出 2 + 6 位码', () => {
    const r = runCli(['-g']);
    expect(r.code).toBe(2);
    expect(r.err).toContain('目标参数');
    expect(r.err).toMatch(/VC-\d{6}/);
  });

  test('-m 非法模式 → 退出 2 + 6 位码', () => {
    const r = runCli(['-m', 'bad']);
    expect(r.code).toBe(2);
    expect(r.err).toContain('非法模式');
    expect(r.err).toMatch(/VC-\d{6}/);
  });

  test('空 VYNTH_API_KEY 跑 headless → 退出 1 + LLM 6 位码', () => {
    const r = runCli(['-g', 'echo run'], { ...process.env, VYNTH_API_KEY: '' });
    expect(r.code).toBe(1);
    expect(r.err).toMatch(/\[VC-\d{6}\]\s+missing VYNTH_API_KEY/);
  });
});

describe('CLI TUI 分流契约（F2/F3）', () => {
  test('无 -g 且非 TTY（stdin/stdout 都不可交互）→ 退出 2 提示用无头模式', () => {
    const r = runCli([]);
    expect(r.code).toBe(2);
    expect(r.err).toContain('无头模式');
  });

  test('-g 在非 TTY 环境下也能跑通（headless 不依赖 TTY）', () => {
    const r = runCli(['-g', 'echo run']);
    expect(r.code).not.toBe(0);
    expect(r.code).not.toBe(2);
  });
});

describe('MCP CLI 接入（F12）', () => {
  const ECHO_SERVER = resolve(REPO, 'packages/mcp/examples/echo-server.ts');

  test('MCP tools 并入 agent 工具集并被 agent 调用', async () => {
    const client = new McpClient(process.execPath, [ECHO_SERVER]);
    await client.connect();
    try {
      const reg = new ToolRegistry();
      for (const d of client.getToolDefs()) reg.register(d);
      expect(reg.get('mcp_echo')).toBeDefined();

      let invoked = '';
      for await (const ev of runAgent('use the mcp tool', {
        provider: new MockProvider(),
        tools: reg,
        maxSteps: 4
      })) {
        if (ev.type === 'tool') invoked = ev.call.name;
      }
      expect(invoked).toBe('mcp_echo');
    } finally {
      client.close();
    }
  });

  test('-s/--mcp 缺值 → 退出 2 + 6 位码', () => {
    const r = runCli(['-g', 'x', '-s']);
    expect(r.code).toBe(2);
    expect(r.err).toContain('服务器命令参数');
    expect(r.err).toMatch(/VC-\d{6}/);
  });

  test('-s 接入 MCP server（fake key 下不崩，退出码非 2）', () => {
    const r = runCli(['-g', 'x', '-s', `${process.execPath} ${ECHO_SERVER}`]);
    expect(r.code).not.toBe(2);
  });
});
