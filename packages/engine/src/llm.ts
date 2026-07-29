import {
  type ChatMessage,
  LlmError,
  type StreamEvent,
  type ToolCall,
  type ToolDef,
  type ZenoConfig
} from '@zeno/core';

export interface LLMProvider {
  chat(messages: ChatMessage[], tools: ToolDef[]): AsyncIterable<StreamEvent>;
}

interface OpenAiDelta {
  choices?: Array<{
    delta?: {
      content?: string;
      reasoning_content?: string;
      tool_calls?: Array<{
        index: number;
        id?: string;
        function?: { name?: string; arguments?: string };
      }>;
    };
    finish_reason?: string | null;
  }>;
  usage?: { prompt_tokens?: number; completion_tokens?: number };
}

export function createProvider(config: ZenoConfig): LLMProvider {
  if (!config.apiKey) throw new LlmError('missing VYNTH_API_KEY; set it to use a real LLM');
  return new OpenAiProvider(config);
}

function isLocalHost(host: string): boolean {
  if (host === 'localhost' || host === '::1' || host === '0.0.0.0') return true;
  if (/^127\./.test(host)) return true;
  if (/^10\./.test(host)) return true;
  if (/^192\.168\./.test(host)) return true;
  if (/^172\.(1[6-9]|2\d|3[01])\./.test(host)) return true;
  if (host.startsWith('::ffff:127.')) return true;
  return false;
}

function assertSafeEndpoint(raw: string): void {
  let url: URL;
  try {
    url = new URL(raw);
  } catch {
    throw new LlmError(`invalid LLM base URL: ${raw}`);
  }
  const host = url.hostname;
  const local = isLocalHost(host);
  if (url.protocol === 'http:' && !local) {
    throw new LlmError(
      'refusing to send API key over plaintext http; use https or a localhost endpoint for local dev'
    );
  }
  if (!local && host !== 'api.openai.com') {
    console.warn(`⚠ Sending API key to non-default LLM endpoint: ${host} — ensure it is trusted.`);
  }
}

class OpenAiProvider implements LLMProvider {
  constructor(private readonly config: ZenoConfig) {
    assertSafeEndpoint(this.config.llmBaseUrl);
  }

  async *chat(messages: ChatMessage[], tools: ToolDef[]): AsyncIterable<StreamEvent> {
    const body = {
      model: this.config.model,
      messages,
      stream: true,
      tools: tools.length
        ? tools.map((t) => ({
            type: 'function',
            function: {
              name: t.name,
              description: t.description,
              parameters: {
                type: 'object',
                properties: toJsonProps(t),
                required: t.parameters.filter((p) => p.required).map((p) => p.name)
              }
            }
          }))
        : undefined
    };

    const res = await fetch(`${this.config.llmBaseUrl}/chat/completions`, {
      method: 'POST',
      headers: {
        'content-type': 'application/json',
        authorization: `Bearer ${this.config.apiKey}`
      },
      body: JSON.stringify(body)
    });
    if (!res.ok || !res.body) {
      const errBody = await res.text().catch(() => '');
      const detail = errBody ? ` — ${errBody.slice(0, 500)}` : '';
      throw new LlmError(`LLM HTTP ${res.status}${detail}`);
    }

    const acc = new Map<number, { id: string; name: string; args: string }>();
    let promptTokens = 0;
    let completionTokens = 0;
    const reader = res.body.getReader();
    const decoder = new TextDecoder();
    let buffer = '';

    while (true) {
      const { value, done } = await reader.read();
      if (done) break;
      buffer += decoder.decode(value, { stream: true });
      const lines = buffer.split('\n');
      buffer = lines.pop() ?? '';
      for (const raw of lines) {
        const line = raw.trim();
        if (!line.startsWith('data:')) continue;
        const data = line.slice(5).trim();
        if (data === '[DONE]') continue;
        let json: OpenAiDelta;
        try {
          json = JSON.parse(data);
        } catch {
          continue;
        }
        const choice = json.choices?.[0];
        const delta = choice?.delta;
        if (delta?.reasoning_content) yield { type: 'reasoning', text: delta.reasoning_content };
        if (delta?.content) yield { type: 'token', text: delta.content };
        for (const tc of delta?.tool_calls ?? []) {
          const slot = acc.get(tc.index) ?? { id: '', name: '', args: '' };
          slot.id ||= tc.id ?? `call-${tc.index}`;
          slot.name ||= tc.function?.name ?? '';
          slot.args += tc.function?.arguments ?? '';
          acc.set(tc.index, slot);
        }
        if (choice?.finish_reason) {
          promptTokens = json.usage?.prompt_tokens ?? promptTokens;
          completionTokens = json.usage?.completion_tokens ?? completionTokens;
        }
      }
    }

    for (const slot of acc.values()) {
      let args: Record<string, unknown> = {};
      try {
        args = JSON.parse(slot.args || '{}');
      } catch {
        args = {};
      }
      const call: ToolCall = { id: slot.id, name: slot.name, args, rawArgs: slot.args || '{}' };
      yield { type: 'tool', call };
    }
    yield { type: 'done', usage: { promptTokens, completionTokens } };
  }
}

function toJsonProps(tool: ToolDef): Record<string, { type: string; description: string }> {
  const props: Record<string, { type: string; description: string }> = {};
  for (const p of tool.parameters) props[p.name] = { type: p.type, description: p.description };
  return props;
}
