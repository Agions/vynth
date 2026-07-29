import { existsSync, readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';

export interface McpServerConfig {
  name: string;
  command: string;
  args?: string[];
  env?: Record<string, string>;
  source: 'builtin' | 'project';
}

interface RawMcpConfig {
  command?: string;
  args?: string[];
  env?: Record<string, string>;
  [key: string]: unknown;
}

const BUILTIN_MCP_SERVERS: McpServerConfig[] = [
  {
    name: 'chrome-devtools',
    command: 'npx',
    args: ['-y', '@modelcontextprotocol/server-chrome-devtools'],
    source: 'builtin'
  },
  {
    name: 'git-analyzer',
    command: 'npx',
    args: ['-y', '@modelcontextprotocol/server-git'],
    source: 'builtin'
  },
  {
    name: 'postgres-db',
    command: 'npx',
    args: ['-y', '@modelcontextprotocol/server-postgres'],
    source: 'builtin'
  },
  {
    name: 'fetch-web',
    command: 'npx',
    args: ['-y', '@modelcontextprotocol/server-fetch'],
    source: 'builtin'
  }
];

export function loadProjectMcpServers(cwd: string): McpServerConfig[] {
  const servers: McpServerConfig[] = [...BUILTIN_MCP_SERVERS];
  const mcpJson = join(cwd, '.vynth', 'mcp.json');
  const mcpDir = join(cwd, '.vynth', 'mcp');

  if (existsSync(mcpJson)) {
    try {
      const raw = JSON.parse(readFileSync(mcpJson, 'utf8'));
      if (raw && typeof raw === 'object') {
        const mcpServers = (raw.mcpServers || raw) as Record<string, RawMcpConfig>;
        for (const [name, config] of Object.entries(mcpServers)) {
          if (config?.command) {
            servers.push({
              name,
              command: config.command,
              args: config.args ?? [],
              env: config.env ?? {},
              source: 'project'
            });
          }
        }
      }
    } catch {}
  }

  if (existsSync(mcpDir)) {
    try {
      const files = readdirSync(mcpDir);
      for (const file of files) {
        if (file.endsWith('.json')) {
          try {
            const raw = JSON.parse(readFileSync(join(mcpDir, file), 'utf8'));
            if (raw?.command) {
              const name = file.replace(/\.json$/, '');
              servers.push({
                name,
                command: raw.command,
                args: raw.args || [],
                env: raw.env || {},
                source: 'project'
              });
            }
          } catch {}
        }
      }
    } catch {}
  }

  return servers;
}
