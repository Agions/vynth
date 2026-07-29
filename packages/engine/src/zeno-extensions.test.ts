import { describe, expect, it } from 'bun:test';
import { loadProjectSkills } from './skills';
import { loadProjectAgents } from './agents';
import { loadProjectMcpServers } from './mcp';

describe('.zeno Custom & Builtin Extensions (Skills, Agents, MCP)', () => {
  it('loads builtin and project skills cleanly', () => {
    const skills = loadProjectSkills(process.cwd());
    expect(skills.length).toBeGreaterThanOrEqual(4);
    expect(skills.some((s) => s.name === 'a11y-debugging')).toBe(true);
    expect(skills.some((s) => s.name === 'memory-leak-debugging')).toBe(true);
    expect(skills.some((s) => s.name === 'lcp-optimization')).toBe(true);
    expect(skills.some((s) => s.name === 'security-audit')).toBe(true);
  });

  it('loads builtin and project subagents cleanly', () => {
    const agents = loadProjectAgents(process.cwd());
    expect(agents.length).toBeGreaterThanOrEqual(4);
    expect(agents.some((a) => a.name === 'reviewer')).toBe(true);
    expect(agents.some((a) => a.name === 'researcher')).toBe(true);
    expect(agents.some((a) => a.name === 'tester')).toBe(true);
    expect(agents.some((a) => a.name === 'refactor')).toBe(true);
  });

  it('loads builtin and project MCP servers cleanly', () => {
    const mcps = loadProjectMcpServers(process.cwd());
    expect(mcps.length).toBeGreaterThanOrEqual(4);
    expect(mcps.some((m) => m.name === 'chrome-devtools')).toBe(true);
    expect(mcps.some((m) => m.name === 'git-analyzer')).toBe(true);
    expect(mcps.some((m) => m.name === 'postgres-db')).toBe(true);
    expect(mcps.some((m) => m.name === 'fetch-web')).toBe(true);
  });
});
