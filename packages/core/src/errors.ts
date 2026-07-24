export class VynthError extends Error {
  code: string;
  constructor(code: string, message: string) {
    super(message);
    this.code = code;
    this.name = 'VynthError';
  }
}

export class ConfigError extends VynthError {
  constructor(message: string) {
    super('config', message);
  }
}

export class LlmError extends VynthError {
  constructor(message: string) {
    super('llm', message);
  }
}

export class ToolError extends VynthError {
  constructor(message: string) {
    super('tool', message);
  }
}

export class SandboxError extends VynthError {
  constructor(message: string) {
    super('sandbox', message);
  }
}

export class McpError extends VynthError {
  constructor(message: string) {
    super('mcp', message);
  }
}

export class PluginError extends VynthError {
  constructor(message: string) {
    super('plugin', message);
  }
}
