import { describe, expect, test } from 'bun:test';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import type { ChatMessage, StreamEvent, ToolCall, ToolDef } from '@zeno/core';
import { audit, initAudit, loadConfig, resetAudit } from '@zeno/core';
import { type LLMProvider, ToolRegistry, createProvider, runAgent } from './index';

class MockProvider implements LLMProvider {
  private calls = 0;
  async *chat(_m: ChatMessage[], tools: ToolDef[]): AsyncIterable<StreamEvent> {
    this.calls++;
    yield { type: 'token', text: 'thinking…' };
    if (tools.length && this.calls === 1) {
      yield { type: 'tool', call: { id: 'c1', name: tools[0].name, args: { x: 1 } } as ToolCall };
    }
    yield { type: 'done' };
  }
}

test('createProvider → 空 apiKey 抛出 LlmError（v0.1.0 起 demo 模式已移除）', () => {
  expect(() => createProvider({ ...loadConfig(), apiKey: '' })).toThrow();
});

test('runAgent 流式 token + 单个工具调用后终止（F4）', async () => {
  const reg = new ToolRegistry();
  let called = 0;
  reg.register({
    name: 'echo_tool',
    description: 'echo',
    parameters: [{ name: 'x', type: 'number', description: 'n', required: true }],
    run: (a) => {
      called++;
      return { ok: true, output: `got ${a.x}` };
    }
  });
  const events: StreamEvent[] = [];
  for await (const ev of runAgent('echo run', {
    provider: new MockProvider(),
    tools: reg,
    maxSteps: 4
  })) {
    events.push(ev);
  }
  expect(events.some((e) => e.type === 'token')).toBe(true);
  expect(events.filter((e) => e.type === 'tool').length).toBe(1);
  expect(called).toBe(1);
  expect(events[events.length - 1].type).toBe('done');
});

class OneToolProvider implements LLMProvider {
  constructor(private toolName: string) {}
  async *chat(_m: ChatMessage[], tools: ToolDef[]): AsyncIterable<StreamEvent> {
    yield { type: 'token', text: 'go' };
    const t = tools.find((x) => x.name === this.toolName) ?? tools[0];
    yield { type: 'tool', call: { id: 'c1', name: t.name, args: {} } as ToolCall };
    yield { type: 'done' };
  }
}

test('runAgent 工具执行后 yield tool_result（ok 路径带 output）', async () => {
  const reg = new ToolRegistry();
  reg.register({
    name: 'ok_tool',
    description: 'ok',
    parameters: [],
    run: () => ({ ok: true, output: 'fine result' })
  });
  const events: StreamEvent[] = [];
  for await (const ev of runAgent('run ok', { provider: new OneToolProvider('ok_tool'), tools: reg })) {
    events.push(ev);
  }
  const r = events.find((e) => e.type === 'tool_result') as Extract<StreamEvent, { type: 'tool_result' }>;
  expect(r).toBeDefined();
  expect(r.ok).toBe(true);
  expect(r.output).toBe('fine result');
  expect(r.name).toBe('ok_tool');
});

test('runAgent 工具失败 yield tool_result（error 带诊断码）', async () => {
  const reg = new ToolRegistry();
  reg.register({
    name: 'fail_tool',
    description: 'fail',
    parameters: [],
    run: () => ({ ok: false, output: '', error: '[VC-030006] sandbox backend unavailable' })
  });
  const events: StreamEvent[] = [];
  for await (const ev of runAgent('run fail', { provider: new OneToolProvider('fail_tool'), tools: reg })) {
    events.push(ev);
  }
  const r = events.find((e) => e.type === 'tool_result') as Extract<StreamEvent, { type: 'tool_result' }>;
  expect(r).toBeDefined();
  expect(r.ok).toBe(false);
  expect(r.error).toContain('VC-030006');
});

test('runAgent 遵守 maxSteps 上限（F4）', async () => {
  class LoopProvider implements LLMProvider {
    async *chat(_m: ChatMessage[], tools: ToolDef[]): AsyncIterable<StreamEvent> {
      yield { type: 'token', text: '.' };
      yield { type: 'tool', call: { id: 'c', name: tools[0]?.name ?? 'echo_tool', args: {} } };
      yield { type: 'done' };
    }
  }
  const reg = new ToolRegistry();
  reg.register({
    name: 'echo_tool',
    description: 'e',
    parameters: [],
    run: () => ({ ok: true, output: 'ok' })
  });
  let steps = 0;
  for await (const ev of runAgent('loop', {
    provider: new LoopProvider(),
    tools: reg,
    maxSteps: 3
  })) {
    if (ev.type === 'tool') steps++;
  }
  expect(steps).toBe(3);
});

test('OpenAiProvider 解析 SSE token + tool_calls + usage（F6/F7）', async () => {
  const sse =
    'data: {"choices":[{"delta":{"content":"hi"}}]}\n' +
    'data: {"choices":[{"delta":{"tool_calls":[{"index":0,"id":"call_1","function":{"name":"read_file","arguments":"{\\"path\\":\\"x\\"}"}}]}}]}\n' +
    'data: {"choices":[{"finish_reason":"tool_calls"}],"usage":{"prompt_tokens":10,"completion_tokens":5}}\n' +
    'data: [DONE]\n';
  const fakeRes = new Response(sse, { status: 200 });
  const orig = globalThis.fetch;
  globalThis.fetch = (async () => fakeRes) as unknown as typeof fetch;
  try {
    const p = createProvider({ ...loadConfig(), apiKey: 'k' });
    const events: StreamEvent[] = [];
    for await (const ev of p.chat([{ role: 'user', content: 'go' }], [])) events.push(ev);
    const token = events.find((e) => e.type === 'token');
    const tool = events.find((e) => e.type === 'tool');
    const done = events.find((e) => e.type === 'done');
    expect(token?.text).toBe('hi');
    expect(tool?.call.name).toBe('read_file');
    expect(done && 'usage' in done && done.usage?.promptTokens).toBe(10);
    expect(done && 'usage' in done && done.usage?.completionTokens).toBe(5);
  } finally {
    globalThis.fetch = orig;
  }
});

test('OpenAiProvider 拒绝向非本地端点发送明文 http（安全红线）', () => {
  expect(() =>
    createProvider({ ...loadConfig(), apiKey: 'x', llmBaseUrl: 'http://evil.example.com/v1' })
  ).toThrow();
});

test('OpenAiProvider 解析 reasoning_content（DeepSeek V4 thinking 模式）', async () => {
  const sse =
    'data: {"choices":[{"delta":{"reasoning_content":"let me think"}}]}\n' +
    'data: {"choices":[{"delta":{"content":"answer"}}]}\n' +
    'data: [DONE]\n';
  const fakeRes = new Response(sse, { status: 200 });
  const orig = globalThis.fetch;
  globalThis.fetch = (async () => fakeRes) as unknown as typeof fetch;
  try {
    const p = createProvider({ ...loadConfig(), apiKey: 'k' });
    const events: StreamEvent[] = [];
    for await (const ev of p.chat([{ role: 'user', content: 'go' }], [])) events.push(ev);
    const reasoning = events.find((e) => e.type === 'reasoning');
    const token = events.find((e) => e.type === 'token');
    expect(reasoning && 'text' in reasoning && reasoning.text).toBe('let me think');
    expect(token?.text).toBe('answer');
  } finally {
    globalThis.fetch = orig;
  }
});

test('OpenAiProvider 400 错误时读取响应体（调试可见性）', async () => {
  const fakeRes = new Response('{"error":{"message":"model not found"}}', { status: 400 });
  const orig = globalThis.fetch;
  globalThis.fetch = (async () => fakeRes) as unknown as typeof fetch;
  try {
    const p = createProvider({ ...loadConfig(), apiKey: 'k' });
    let thrown: Error | null = null;
    try {
      for await (const _ of p.chat([{ role: 'user', content: 'go' }], [])) {
        // consume
      }
    } catch (e) {
      thrown = e instanceof Error ? e : new Error(String(e));
    }
    expect(thrown).not.toBeNull();
    expect(thrown?.message).toContain('400');
    expect(thrown?.message).toContain('model not found');
  } finally {
    globalThis.fetch = orig;
  }
});

test('runAgent 正确构建 tool_calls + reasoning_content + tool_call_id 消息', async () => {
  let secondCallMessages: ChatMessage[] | null = null;

  class CaptureProvider implements LLMProvider {
    private calls = 0;
    async *chat(messages: ChatMessage[], tools: ToolDef[]): AsyncIterable<StreamEvent> {
      this.calls++;
      if (this.calls === 2) secondCallMessages = [...messages];
      yield { type: 'reasoning', text: 'thinking...' };
      yield { type: 'token', text: 'checking' };
      if (this.calls === 1 && tools.length) {
        yield {
          type: 'tool',
          call: {
            id: 'call_1',
            name: tools[0].name,
            args: { x: 1 },
            rawArgs: '{"x":1}'
          } as ToolCall
        };
      }
      yield { type: 'done' };
    }
  }

  const reg = new ToolRegistry();
  reg.register({
    name: 'echo_tool',
    description: 'echo',
    parameters: [{ name: 'x', type: 'number', description: 'n', required: true }],
    run: (a) => ({ ok: true, output: `got ${a.x}` })
  });

  for await (const _ of runAgent('test', {
    provider: new CaptureProvider(),
    tools: reg,
    maxSteps: 2
  })) {
    // consume
  }

  expect(secondCallMessages).not.toBeNull();
  const msgs: ChatMessage[] = secondCallMessages ?? [];

  const assistant = msgs.find((m) => m.role === 'assistant');
  expect(assistant).toBeDefined();
  expect(assistant?.tool_calls).toBeDefined();
  expect(assistant?.tool_calls?.[0]?.id).toBe('call_1');
  expect(assistant?.tool_calls?.[0]?.type).toBe('function');
  expect(assistant?.tool_calls?.[0]?.function?.name).toBe('echo_tool');
  expect(assistant?.tool_calls?.[0]?.function?.arguments).toBe('{"x":1}');
  expect(assistant?.reasoning_content).toBe('thinking...');

  const tool = msgs.find((m) => m.role === 'tool');
  expect(tool).toBeDefined();
  expect(tool?.tool_call_id).toBe('call_1');
  expect(tool?.content).toContain('got 1');
});

test('runAgent 工具执行被 F14 审计记录（tool_exec 维度）', async () => {
  const dir = mkdtempSync(join(tmpdir(), 'zeno-eng-audit-'));
  try {
    initAudit({ dataDir: dir, audit: true });
    const reg = new ToolRegistry();
    reg.register({
      name: 'echo_tool',
      description: 'echo',
      parameters: [{ name: 'x', type: 'number', description: 'n', required: true }],
      run: (a) => ({ ok: true, output: `got ${a.x}` })
    });
    for await (const _ of runAgent('audit me', {
      provider: new MockProvider(),
      tools: reg,
      maxSteps: 2
    })) {
      // consume
    }
    const records = audit().readAllSync();
    expect(
      records.some(
        (r) =>
          r.kind === 'tool_exec' && (r.detail as { name?: string }).name === 'echo_tool' && r.ok
      )
    ).toBe(true);
  } finally {
    resetAudit();
    rmSync(dir, { recursive: true, force: true });
  }
});
