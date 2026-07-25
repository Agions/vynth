import { describe, expect, test } from 'bun:test';
import type { ChatMessage, StreamEvent, ToolCall, ToolDef } from '@vynth/core';
import { loadConfig } from '@vynth/core';
import { type LLMProvider, ToolRegistry, createProvider, runAgent } from './index';

// 一个可控的 Mock Provider：第一次返回 token + 一个工具调用，之后只返回 token。
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

test('createProvider → 空 apiKey 抛出 LlmError（v0.2.1 起 demo 模式已移除）', () => {
  expect(() => createProvider({ ...loadConfig(), apiKey: '' })).toThrow();
  expect(() => createProvider({ ...loadConfig(), apiKey: undefined })).toThrow();
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

test('runAgent 遵守 maxSteps 上限（F4）', async () => {
  // 一个每次都要求调用工具的 Provider，必须被 maxSteps 截断。
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
