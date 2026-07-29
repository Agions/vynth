import { type ChatMessage, type Mode, type StreamEvent, type ToolCall, audit } from '@zeno/core';
import type { LLMProvider } from './llm';
import type { ToolRegistry } from './tools';

export interface AgentOpts {
  provider: LLMProvider;
  tools: ToolRegistry;
  system?: string;
  mode?: Mode;
  maxSteps?: number;
  repoMap?: string;
}

import { loadProjectAgents } from './agents';
import { loadProjectSkills } from './skills';

const DEFAULT_SYSTEM =
  '你是 Vynth，一个终端内的 AI 编程助手。请用简洁中文回复，必要时调用工具完成任务。';

const PLAN_SYSTEM =
  '你是 Vynth，一个终端内的 AI 编程助手。当前处于 Plan（规划）模式。请先进行需求与架构分析，输出清晰的分步实施计划与风险评估，而后按计划逐步调用工具执行。';

const AUTO_SYSTEM =
  '你是 Vynth，一个终端内的自主 AI 编程 Agent。当前处于 Auto（自主编程）模式。你拥有完全自主规划、多步拆解、自我纠错与工具调用的权限。请自动推进直到目标完美完成。';

function pruneContext(messages: ChatMessage[], maxMessages = 24): void {
  if (messages.length <= maxMessages) return;
  const keepEndCount = 12;
  const startIdx = 2;
  const endIdx = messages.length - keepEndCount;
  for (let i = startIdx; i < endIdx; i++) {
    const msg = messages[i];
    if (msg.role === 'tool' && msg.content && msg.content.length > 400) {
      msg.content = `${msg.content.slice(0, 200)}\n… [已截断 ${msg.content.length - 400} 字符以节省 context] …\n${msg.content.slice(-200)}`;
    }
  }
}

export async function* runAgent(goal: string, opts: AgentOpts): AsyncGenerator<StreamEvent> {
  const messages: ChatMessage[] = [];
  let systemContent: string =
    opts.system ??
    (opts.mode === 'plan' ? PLAN_SYSTEM : opts.mode === 'auto' ? AUTO_SYSTEM : DEFAULT_SYSTEM);
  try {
    const { readFileSync, existsSync } = await import('node:fs');
    const { join } = await import('node:path');
    const cwd = (opts.tools as unknown as { cwd?: string } | undefined)?.cwd || process.cwd();
    const candidates = [
      join(cwd, 'AGENTS.md'),
      join(cwd, '.vynth', 'AGENTS.md'),
      join(cwd, 'PROJECT.md')
    ];
    for (const candidate of candidates) {
      if (existsSync(candidate)) {
        const content = readFileSync(candidate, 'utf8').trim();
        if (content) {
          systemContent += `\n\n=== 项目专属 AI 开发规则 (AGENTS.md) ===\n${content}`;
          break;
        }
      }
    }
    const skills = loadProjectSkills(cwd);
    if (skills.length > 0) {
      const skillText = skills.map((s) => `- ${s.name}: ${s.description}`).join('\n');
      systemContent += `\n\n=== 可用 Skills 技能库 (.vynth/skills) ===\n${skillText}`;
    }
    const agents = loadProjectAgents(cwd);
    if (agents.length > 0) {
      const agentText = agents.map((a) => `- ${a.name} (${a.role}): ${a.systemPrompt}`).join('\n');
      systemContent += `\n\n=== 可用 Subagents 专家库 (.vynth/agents) ===\n${agentText}`;
    }
  } catch {}

  if (opts.repoMap && opts.repoMap.trim().length > 0) {
    systemContent += `\n\n${opts.repoMap}`;
  }
  messages.push({ role: 'system', content: systemContent });
  messages.push({ role: 'user', content: goal });

  const toolDefs = opts.tools.list();
  const maxSteps = opts.maxSteps ?? 8;

  for (let step = 0; step < maxSteps; step++) {
    pruneContext(messages);

    let assistantText = '';
    let reasoningContent = '';
    const pendingTools: ToolCall[] = [];

    for await (const ev of opts.provider.chat(messages, toolDefs)) {
      if (ev.type === 'token') {
        assistantText += ev.text;
        yield ev;
      } else if (ev.type === 'reasoning') {
        reasoningContent += ev.text;
        yield ev;
      } else if (ev.type === 'tool') {
        pendingTools.push(ev.call);
      } else if (ev.type === 'done') {
        yield ev;
      }
    }

    if (pendingTools.length === 0) break;

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

    for (const tc of pendingTools) {
      yield { type: 'tool', call: tc };
      const result = await opts.tools.run(tc.name, tc.args);
      audit().record('tool_exec', { name: tc.name, ok: result.ok }, result.ok);
      yield {
        type: 'tool_result',
        id: tc.id,
        name: tc.name,
        args: tc.args,
        ok: result.ok,
        output: result.output ?? '',
        error: result.error
      };
      messages.push({
        role: 'tool',
        tool_call_id: tc.id,
        content: result.ok ? result.output : `ERROR: ${result.error}`
      });
    }
  }
}
