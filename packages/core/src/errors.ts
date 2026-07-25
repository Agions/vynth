import {
  DEFAULT_CODE_BY_FAMILY,
  type VynthErrorCode,
  fromLegacy,
  isVynthErrorCode
} from './error-codes';

export type { VynthErrorCode } from './error-codes';

/**
 * 所有 Vynth 错误的根类。v0.1.0 起，每个错误实例同时携带：
 *   - `code: string`  — 向后兼容字段（旧族名 `config`/`llm`/...）
 *   - `numericCode: VynthErrorCode` — 权威 6 位码 `VC-XXXXXX`
 *
 * 老调用点：`new ConfigError('xxx')` —— 自动取族默认码
 * 新调用点：`new ConfigError('xxx', 'VC-010002')` —— 显式 6 位码
 */
export class VynthError extends Error {
  code: string;
  numericCode: VynthErrorCode;

  constructor(code: string, message: string, numericCode?: VynthErrorCode) {
    super(message);
    this.name = 'VynthError';
    // 兼容老字段：保留家族字符串
    this.code = code;
    // 6 位码：优先用显式 → 否则从老 code 解码 → 否则降级为族默认
    if (numericCode && isVynthErrorCode(numericCode)) {
      this.numericCode = numericCode;
    } else {
      const decoded = fromLegacy(code);
      this.numericCode =
        decoded ?? (DEFAULT_CODE_BY_FAMILY as Record<string, VynthErrorCode>)[code] ?? 'VC-010099';
    }
  }
}

export class ConfigError extends VynthError {
  constructor(message: string, numericCode?: VynthErrorCode) {
    super('config', message, numericCode);
    this.name = 'ConfigError';
  }
}

export class LlmError extends VynthError {
  constructor(message: string, numericCode?: VynthErrorCode) {
    super('llm', message, numericCode);
    this.name = 'LlmError';
  }
}

export class ToolError extends VynthError {
  constructor(message: string, numericCode?: VynthErrorCode) {
    super('tool', message, numericCode);
    this.name = 'ToolError';
  }
}

export class SandboxError extends VynthError {
  constructor(message: string, numericCode?: VynthErrorCode) {
    super('sandbox', message, numericCode);
    this.name = 'SandboxError';
  }
}

export class McpError extends VynthError {
  constructor(message: string, numericCode?: VynthErrorCode) {
    super('mcp', message, numericCode);
    this.name = 'McpError';
  }
}

export class PluginError extends VynthError {
  constructor(message: string, numericCode?: VynthErrorCode) {
    super('plugin', message, numericCode);
    this.name = 'PluginError';
  }
}
