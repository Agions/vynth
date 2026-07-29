import {
  DEFAULT_CODE_BY_FAMILY,
  type ZenoErrorCode,
  fromLegacy,
  isZenoErrorCode
} from './error-codes';

export type { ZenoErrorCode } from './error-codes';

export class ZenoError extends Error {
  code: string;
  numericCode: ZenoErrorCode;

  constructor(code: string, message: string, numericCode?: ZenoErrorCode) {
    super(message);
    this.name = 'ZenoError';
    this.code = code;
    if (numericCode && isZenoErrorCode(numericCode)) {
      this.numericCode = numericCode;
    } else {
      const decoded = fromLegacy(code);
      const familyDefault = (DEFAULT_CODE_BY_FAMILY as Record<string, ZenoErrorCode | undefined>)[
        code
      ];
      this.numericCode = decoded ?? familyDefault ?? 'VC-010099';
    }
  }
}

export class ConfigError extends ZenoError {
  constructor(message: string, numericCode?: ZenoErrorCode) {
    super('config', message, numericCode);
    this.name = 'ConfigError';
  }
}

export class LlmError extends ZenoError {
  constructor(message: string, numericCode?: ZenoErrorCode) {
    super('llm', message, numericCode);
    this.name = 'LlmError';
  }
}

export class ToolError extends ZenoError {
  constructor(message: string, numericCode?: ZenoErrorCode) {
    super('tool', message, numericCode);
    this.name = 'ToolError';
  }
}

export class SandboxError extends ZenoError {
  constructor(message: string, numericCode?: ZenoErrorCode) {
    super('sandbox', message, numericCode);
    this.name = 'SandboxError';
  }
}

export class McpError extends ZenoError {
  constructor(message: string, numericCode?: ZenoErrorCode) {
    super('mcp', message, numericCode);
    this.name = 'McpError';
  }
}

export class PluginError extends ZenoError {
  constructor(message: string, numericCode?: ZenoErrorCode) {
    super('plugin', message, numericCode);
    this.name = 'PluginError';
  }
}

export function toErrorMessage(err: unknown): string {
  if (err instanceof ZenoError) return err.message;
  return err instanceof Error ? err.message : String(err);
}

export function formatZenoError(err: unknown): string {
  if (err && typeof err === 'object' && 'numericCode' in err && 'message' in err) {
    const e = err as { numericCode?: string; message?: string };
    if (e.numericCode && /^VC-\d{6}$/.test(e.numericCode)) {
      return `[${e.numericCode}] ${e.message ?? ''}`.trim();
    }
  }
  return toErrorMessage(err);
}
