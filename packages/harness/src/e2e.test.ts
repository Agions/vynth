import { expect, test } from 'bun:test';
import { mkdtempSync, rmSync, symlinkSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import type { ChatMessage, StreamEvent, ToolCall, ToolDef } from '@vynth/core';
import { loadConfig } from '@vynth/core';
import {
  type LLMProvider,
  ToolRegistry,
  builtinTools,
  createProvider,
  runAgent
} from '@vynth/engine';
import * as samplePlugin from './fixtures/sample-plugin.ts';

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
  for await (const ev of runAgent('demo-tool please', {
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

test('demo EchoProvider streams tokens when no API key', async () => {
  const provider = createProvider({ ...loadConfig(), apiKey: '' });
  let tokenText = '';
  let toolCount = 0;
  for await (const ev of provider.chat([{ role: 'user', content: 'hello' }], [])) {
    if (ev.type === 'token') tokenText += ev.text;
    else if (ev.type === 'tool') toolCount++;
  }
  expect(tokenText.length).toBeGreaterThan(0);
  expect(tokenText).toContain('demo');
  expect(toolCount).toBe(0);
});

test('sandbox read_file reads within cwd and rejects escape', async () => {
  const dir = mkdtempSync(join(tmpdir(), 'vynth-test-'));
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
  const dir = mkdtempSync(join(tmpdir(), 'vynth-sym-'));
  const outside = mkdtempSync(join(tmpdir(), 'vynth-out-'));
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
  const dir = mkdtempSync(join(tmpdir(), 'vynth-net-'));
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
