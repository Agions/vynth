import { describe, expect, test } from 'bun:test';
import { resolve } from 'node:path';
import { MCP_PROTOCOL_VERSION, McpClient, type McpTool, mcpToolToToolDef } from './mcp-client';

const ECHO_SERVER = resolve(import.meta.dir, '../examples/echo-server.ts');

describe('mcpToolToToolDef', () => {
  test('maps JSON Schema properties to ToolParam with required flags', () => {
    const tool: McpTool = {
      name: 'mcp_echo',
      description: 'echo',
      inputSchema: {
        type: 'object',
        properties: {
          message: { type: 'string', description: 'msg' },
          count: { type: 'integer', description: 'n' },
          flag: { type: 'boolean', description: 'f' }
        },
        required: ['message']
      }
    };
    const def = mcpToolToToolDef(tool, async () => ({ ok: true, output: '' }));
    expect(def.name).toBe('mcp_echo');
    expect(def.description).toBe('echo');
    expect(def.parameters).toHaveLength(3);

    expect(def.parameters.find((p) => p.name === 'message')?.type).toBe('string');
    expect(def.parameters.find((p) => p.name === 'message')?.required).toBe(true);
    expect(def.parameters.find((p) => p.name === 'count')?.type).toBe('number'); // integer → number
    expect(def.parameters.find((p) => p.name === 'count')?.required).toBe(false);
    expect(def.parameters.find((p) => p.name === 'flag')?.type).toBe('boolean');
  });

  test('run delegates to the injected call', async () => {
    const tool: McpTool = {
      name: 't',
      inputSchema: { properties: { a: { type: 'string' } }, required: ['a'] }
    };
    const def = mcpToolToToolDef(tool, async (n, a) => ({ ok: true, output: `${n}:${a.a}` }));
    const r = await def.run({ a: 'x' });
    expect(r.ok).toBe(true);
    expect(r.output).toBe('t:x');
  });
});

describe('McpClient (stdio JSON-RPC 2024-11-05)', () => {
  test('connects to echo server, lists tools, and calls them', async () => {
    const client = new McpClient(process.execPath, [ECHO_SERVER]);
    try {
      await client.connect();
      expect(MCP_PROTOCOL_VERSION).toBe('2024-11-05');

      const defs = client.getToolDefs();
      expect(defs.length).toBeGreaterThan(0);
      const echo = defs.find((d) => d.name === 'mcp_echo');
      expect(echo).toBeDefined();
      expect(echo?.parameters.find((p) => p.name === 'message')?.required).toBe(true);

      const res = await client.callTool('mcp_echo', { message: 'hi' });
      expect(res.ok).toBe(true);
      expect(res.output).toContain('echo: hi');
    } finally {
      client.close();
    }
  });

  test('callTool surfaces server errors via ToolResult (no throw)', async () => {
    const client = new McpClient(process.execPath, [ECHO_SERVER]);
    try {
      await client.connect();
      const res = await client.callTool('mcp_echo', {});
      expect(res.ok).toBe(true);
    } finally {
      client.close();
    }
  });
});
