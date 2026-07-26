/**
 * 最小 stdio JSON-RPC 2024-11-05 MCP server（仅用于 vynth 测试与文档示例）。
 *
 * 暴露一个 `mcp_echo` 工具：把传入的 `message` 原样回显。
 * 通过标准输入按行读取 JSON-RPC 请求，向标准输出按行写回响应。
 *
 * 用法（配合 vynth）： vynth -g "<目标>" -s "bun run packages/mcp/examples/echo-server.ts"
 */
interface JsonRpcReq {
  jsonrpc: '2.0';
  id?: number;
  method: string;
  params?: unknown;
}

interface JsonRpcRes {
  jsonrpc: '2.0';
  id: number;
  result?: unknown;
  error?: { message: string };
}

const ECHO_TOOL = {
  name: 'mcp_echo',
  description: '回显传入的消息（MCP 示例工具，用于验证 -s/--mcp 接入）',
  inputSchema: {
    type: 'object',
    properties: {
      message: { type: 'string', description: '要回显的内容' }
    },
    required: ['message']
  }
};

function send(res: JsonRpcRes): void {
  process.stdout.write(`${JSON.stringify(res)}\n`);
}

function handle(req: JsonRpcReq): void {
  const id = req.id ?? -1;
  switch (req.method) {
    case 'initialize':
      send({
        jsonrpc: '2.0',
        id,
        result: {
          protocolVersion: '2024-11-05',
          capabilities: { tools: {} },
          serverInfo: { name: 'vynth-echo', version: '0.1.0' }
        }
      });
      break;
    case 'tools/list':
      send({ jsonrpc: '2.0', id, result: { tools: [ECHO_TOOL] } });
      break;
    case 'tools/call': {
      const args = ((req.params as { arguments?: Record<string, unknown> })?.arguments ?? {}) as {
        message?: string;
      };
      send({
        jsonrpc: '2.0',
        id,
        result: { content: [{ type: 'text', text: `echo: ${args.message ?? ''}` }] }
      });
      break;
    }
    default:
      // 通知类（无 id）忽略；未知方法回错
      if (req.id !== undefined) {
        send({ jsonrpc: '2.0', id, error: { message: `unknown method: ${req.method}` } });
      }
  }
}

let buf = '';
process.stdin.setEncoding('utf8');
process.stdin.on('data', (chunk: string) => {
  buf += chunk;
  const lines = buf.split('\n');
  buf = lines.pop() ?? '';
  for (const raw of lines) {
    const line = raw.trim();
    if (!line) continue;
    try {
      handle(JSON.parse(line) as JsonRpcReq);
    } catch {
      // 忽略无法解析的行
    }
  }
});
