import { existsSync, readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';

export interface AgentDef {
  name: string;
  role: string;
  systemPrompt: string;
  modelTier?: 'inherit' | 'pro' | 'flash';
  source: 'builtin' | 'project';
}

const BUILTIN_AGENTS: AgentDef[] = [
  {
    name: 'reviewer',
    role: 'Code Review Agent',
    systemPrompt: '资深代码评审专家，审查代码逻辑缺陷、潜在 Bug、安全隐患与性能瓶颈。',
    modelTier: 'pro',
    source: 'builtin'
  },
  {
    name: 'researcher',
    role: 'Codebase Researcher',
    systemPrompt: '代码库深度调研专家，分析架构依赖关系、梳理模块调用与符号地图。',
    modelTier: 'flash',
    source: 'builtin'
  },
  {
    name: 'tester',
    role: 'Unit Test Engineer',
    systemPrompt: '单元测试开发专家，编写高覆盖率的边界条件单测与断言断定。',
    modelTier: 'inherit',
    source: 'builtin'
  },
  {
    name: 'refactor',
    role: 'Refactoring Specialist',
    systemPrompt: '架构重构专家，在保持既有 API 契约前提下优化模块设计与降低耦合。',
    modelTier: 'pro',
    source: 'builtin'
  }
];

export function loadProjectAgents(cwd: string): AgentDef[] {
  const agents: AgentDef[] = [...BUILTIN_AGENTS];
  const agentsDir = join(cwd, '.zeno', 'agents');
  if (!existsSync(agentsDir)) return agents;

  try {
    const files = readdirSync(agentsDir);
    for (const file of files) {
      const full = join(agentsDir, file);
      if (file.endsWith('.json')) {
        try {
          const raw = JSON.parse(readFileSync(full, 'utf8'));
          if (raw && raw.name && raw.systemPrompt) {
            agents.push({
              name: raw.name,
              role: raw.role || raw.name,
              systemPrompt: raw.systemPrompt,
              modelTier: raw.modelTier || 'inherit',
              source: 'project'
            });
          }
        } catch {}
      } else if (file.endsWith('.md')) {
        const content = readFileSync(full, 'utf8');
        const name = file.replace(/\.md$/, '');
        agents.push({
          name,
          role: `${name} Custom Subagent`,
          systemPrompt: content,
          modelTier: 'inherit',
          source: 'project'
        });
      }
    }
  } catch {}

  return agents;
}
