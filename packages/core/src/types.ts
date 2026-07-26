export type Mode = 'plan' | 'vibe';

/**
 * 工具调用引用（assistant 消息中的 tool_calls 数组元素）。
 * 对齐 OpenAI / DeepSeek V4 chat-completions 格式。
 */
export interface ToolCallRef {
  id: string;
  type: 'function';
  function: { name: string; arguments: string };
}

export interface ChatMessage {
  role: 'system' | 'user' | 'assistant' | 'tool';
  content: string;
  name?: string;
  /** DeepSeek V4 thinking 模式：assistant 的推理链，工具调用轮次必须原样回传 */
  reasoning_content?: string;
  /** assistant 消息携带的工具调用列表（与 tool 角色消息配对） */
  tool_calls?: ToolCallRef[];
  /** tool 角色消息：对应 tool_calls[].id，API 要求必填 */
  tool_call_id?: string;
}

export interface ToolParam {
  name: string;
  type: 'string' | 'number' | 'boolean';
  description: string;
  required: boolean;
}

export interface ToolResult {
  ok: boolean;
  output: string;
  error?: string;
}

export interface ToolDef {
  name: string;
  description: string;
  parameters: ToolParam[];
  run: (args: Record<string, unknown>) => Promise<ToolResult> | ToolResult;
}

export interface ToolCall {
  id: string;
  name: string;
  args: Record<string, unknown>;
  /** 原始 JSON 参数字符串，用于回传 assistant 消息的 tool_calls 字段 */
  rawArgs?: string;
}

export type StreamEvent =
  | { type: 'token'; text: string }
  | { type: 'reasoning'; text: string }
  | { type: 'tool'; call: ToolCall }
  | { type: 'done'; usage?: { promptTokens: number; completionTokens: number } };

export interface VynthConfig {
  mode: Mode;
  llmBaseUrl: string;
  apiKey: string;
  model: string;
  theme: 'mocha' | 'latte';
  sandbox: { networkAllowed: boolean; cwd: string };
  dataDir: string;
  /** F14 配置合规层：是否启用 5 维审计落盘（opt-in，默认 false） */
  audit: boolean;
}
