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
    let reasoningContent = '';
    const pendingTools: ToolCall[] = [];

    for await (const ev of opts.provider.chat(messages, toolDefs)) {
      if (ev.type === 'token') {
        assistantText += ev.text;
        yield ev;
      } else if (ev.type === 'reasoning') {
        reasoningContent += ev.text;
      } else if (ev.type === 'tool') {
        pendingTools.push(ev.call);
      } else if (ev.type === 'done') {
        yield ev;
      }
    }

    if (pendingTools.length === 0) break;

    // 构建 assistant 消息：携带 tool_calls + reasoning_content（DeepSeek V4 thinking 模式要求）
    messages.push({
      role: 'assistant',
      content: assistantText,
      reasoning_content: reasoningContent || undefined,
      tool_calls: pendingTools.map((t) => ({
        id: t.id,
        type: 'function' as const,
        function: {
          name: t.name,
          arguments: t.rawArgs ?? JSON.stringify(t.args)
        }
      }))
    });

    // 执行每个工具并构建 tool 角色消息（必须用 tool_call_id 关联）
    for (const tc of pendingTools) {
      yield { type: 'tool', call: tc };
      const result = await opts.tools.run(tc.name, tc.args);
      messages.push({
        role: 'tool',
        tool_call_id: tc.id,
        content: result.ok ? result.output : `ERROR: ${result.error}`
      });
    }
  }
}
