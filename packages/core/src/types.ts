export type Mode = 'plan' | 'vibe' | 'auto';

export interface ToolCallRef {
  id: string;
  type: 'function';
  function: { name: string; arguments: string };
}

export interface ChatMessage {
  role: 'system' | 'user' | 'assistant' | 'tool';
  content: string;
  name?: string;
  reasoning_content?: string;
  tool_calls?: ToolCallRef[];
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
  rawArgs?: string;
}

export type StreamEvent =
  | { type: 'token'; text: string }
  | { type: 'reasoning'; text: string }
  | { type: 'tool'; call: ToolCall }
  | {
      type: 'tool_result';
      id: string;
      name: string;
      args: Record<string, unknown>;
      ok: boolean;
      output: string;
      error?: string;
    }
  | { type: 'done'; usage?: { promptTokens: number; completionTokens: number } };

export interface ZenoConfig {
  mode: Mode;
  llmBaseUrl: string;
  apiKey: string;
  model: string;
  theme: 'mocha' | 'latte' | 'neon';
  sandbox: { networkAllowed: boolean; cwd: string; harden: boolean };
  dataDir: string;
  audit: boolean;
  repomap: RepomapConfig;
}

export interface RepomapConfig {
  enabled: boolean;
  maxSymbols: number;
  includeTests: boolean;
}
