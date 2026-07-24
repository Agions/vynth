import type { ChatMessage, StreamEvent, ToolCall } from '@vynth/core';
import type { LLMProvider } from './llm';
import type { ToolRegistry } from './tools';

export interface AgentOpts {
  provider: LLMProvider;
  tools: ToolRegistry;
  system?: string;
  maxSteps?: number;
}

const DEFAULT_SYSTEM =
  '你是 Vynth，一个终端内的 AI 编程助手。请用简洁中文回复，必要时调用工具完成任务。';

export async function* runAgent(goal: string, opts: AgentOpts): AsyncGenerator<StreamEvent> {
  const messages: ChatMessage[] = [];
  if (opts.system) messages.push({ role: 'system', content: opts.system });
  else messages.push({ role: 'system', content: DEFAULT_SYSTEM });
  messages.push({ role: 'user', content: goal });

  const toolDefs = opts.tools.list();
  const maxSteps = opts.maxSteps ?? 8;

  for (let step = 0; step < maxSteps; step++) {
    let assistantText = '';
    let pendingTool: ToolCall | null = null;

    for await (const ev of opts.provider.chat(messages, toolDefs)) {
      if (ev.type === 'token') {
        assistantText += ev.text;
        yield ev;
      } else if (ev.type === 'tool') {
        pendingTool = ev.call;
      } else if (ev.type === 'done') {
        yield ev;
      }
    }

    if (!pendingTool) break;

    yield { type: 'tool', call: pendingTool };
    const result = await opts.tools.run(pendingTool.name, pendingTool.args);
    messages.push({ role: 'assistant', content: assistantText });
    messages.push({
      role: 'tool',
      name: pendingTool.name,
      content: result.ok ? result.output : `ERROR: ${result.error}`
    });
  }
}
