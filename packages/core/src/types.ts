export type Mode = 'plan' | 'vibe';

export interface ChatMessage {
  role: 'system' | 'user' | 'assistant' | 'tool';
  content: string;
  name?: string;
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
}

export type StreamEvent =
  | { type: 'token'; text: string }
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
}
