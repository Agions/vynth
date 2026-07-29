import { existsSync, readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';

export interface McpServerConfig {
  name: string;
  command: string;
  args?: string[];
  env?: Record<string, string>;
  source: 'builtin' | 'project';
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
  const mcpJson = join(cwd, '.zeno', 'mcp.json');
  const mcpDir = join(cwd, '.zeno', 'mcp');

  if (existsSync(mcpJson)) {
    try {
      const raw = JSON.parse(readFileSync(mcpJson, 'utf8'));
      if (raw && typeof raw === 'object') {
        const mcpServers = raw.mcpServers || raw;
        for (const [name, config] of Object.entries(mcpServers)) {
          const cfg = config as any;
          if (cfg && cfg.command) {
            servers.push({
              name,
              command: cfg.command,
              args: cfg.args || [],
              env: cfg.env || {},
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
            if (raw && raw.command) {
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
