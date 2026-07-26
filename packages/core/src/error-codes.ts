/**
 * Vynth 6 位错误码表（VC-XXXXXX）。
 *
 * 编号规则：
 *   VC-AABBCC
 *     AA = 错误族（01=Config, 02=LLM, 03=Sandbox, 04=Tool, 05=Plugin, 06=MCP）
 *     BB = 子类（同族内聚类）
 *     CC = 实例序号
 *
 * 设计动机：在 6 位码之前的字符串码（`config`/`llm`/`tool`/`sandbox`/`mcp`/`plugin`）
 * 对应的人类可读性与可 grep 性差，给运维与用户诊断带来负担。本表统一 6 位码，
 * 旧字符串仍可经 `fromLegacy()` 解码为新码（向后兼容）。
 *
 * 错误类设计（`packages/core/src/errors.ts`）保留字符串域 `code` 作向后兼容字段，
 * 同时暴露 `numericCode: VynthErrorCode` 作为本表权威值。
 */

export type VynthErrorCode =
  // ----- 01 Config / 参数族 -----
  | 'VC-010001' // CONFIG_MISSING_KEY
  | 'VC-010002' // CONFIG_INVALID_MODE
  | 'VC-010003' // CONFIG_UNKNOWN_FLAG
  | 'VC-010004' // CONFIG_VALUE_MISSING // -g/-m/-p 缺值
  | 'VC-010005' // CONFIG_FILE_SECRET // 配置文件中不得写入密钥
  | 'VC-010006' // CONFIG_FILE_INVALID // 配置文件 schema / JSON 非法
  // ----- 02 LLM 族 -----
  | 'VC-020001' // LLM_AUTH_FAILED
  | 'VC-020002' // LLM_RATE_LIMITED
  | 'VC-020003' // LLM_NETWORK
  | 'VC-020004' // LLM_INVALID_RESPONSE // SSE 解析失败
  | 'VC-020005' // LLM_PLAINTEXT_HTTP // 拒绝明文 http
  // ----- 03 Sandbox 族 -----
  | 'VC-030001' // SANDBOX_PATH_ESCAPE // ../ 越界
  | 'VC-030002' // SANDBOX_SYMLINK_ESCAPE
  | 'VC-030003' // SANDBOX_NETWORK_BLOCKED // run_shell 被 VYNTH_NET=0 阻断
  | 'VC-030004' // SANDBOX_READ_FAILED
  | 'VC-030005' // SANDBOX_WRITE_FAILED
  // ----- 04 Tool 族 -----
  | 'VC-040001' // TOOL_NOT_FOUND
  | 'VC-040002' // TOOL_EXECUTION_FAILED
  | 'VC-040003' // TOOL_INVALID_ARGS
  // ----- 05 Plugin 族 -----
  | 'VC-050001' // PLUGIN_LOAD_FAILED // 动态 import 失败
  | 'VC-050002' // PLUGIN_MISSING_ACTIVATE
  | 'VC-050003' // PLUGIN_MISSING_NAME
  // ----- 06 MCP 族 -----
  | 'VC-060001' // MCP_NOT_IMPLEMENTED // F12 尚未落地
  | 'VC-060002' // MCP_PROTOCOL_PARSE
  | 'VC-060003'; // MCP_REQUEST_TIMEOUT;

// 默认族代码（错误类未显式传 code 时回退）
export const DEFAULT_CODE_BY_FAMILY = {
  config: 'VC-010099',
  llm: 'VC-020099',
  sandbox: 'VC-030099',
  tool: 'VC-040099',
  plugin: 'VC-050099',
  mcp: 'VC-060099'
} as const satisfies Record<string, VynthErrorCode>;

const ALL_CODES: ReadonlySet<VynthErrorCode> = new Set<VynthErrorCode>([
  'VC-010001',
  'VC-010002',
  'VC-010003',
  'VC-010004',
  'VC-010005',
  'VC-010006',
  'VC-020001',
  'VC-020002',
  'VC-020003',
  'VC-020004',
  'VC-020005',
  'VC-030001',
  'VC-030002',
  'VC-030003',
  'VC-030004',
  'VC-030005',
  'VC-040001',
  'VC-040002',
  'VC-040003',
  'VC-050001',
  'VC-050002',
  'VC-050003',
  'VC-060001',
  'VC-060002',
  'VC-060003'
]);

const CODE_RE = /^VC-\d{6}$/;

/**
 * 校验一个字符串是否为合法的 6 位码（不强制在 ALL_CODES 中，便于未来加码）。
 */
export function isVynthErrorCode(s: string): s is VynthErrorCode {
  return CODE_RE.test(s);
}

/**
 * 把字符串或未知值解析为 6 位码：
 *  - 已是合法 6 位码 → 原样返回
 *  - 旧族字符串（`config`/`llm`/`tool`/`sandbox`/`plugin`/`mcp`）→ 该族默认码
 *  - 其余 → null（非法）
 */
const LEGACY_FAMILY_TO_DEFAULT: Record<string, VynthErrorCode> = {
  config: 'VC-010099',
  llm: 'VC-020099',
  sandbox: 'VC-030099',
  tool: 'VC-040099',
  plugin: 'VC-050099',
  mcp: 'VC-060099'
};

export function fromLegacy(value: string | null | undefined): VynthErrorCode | null {
  if (!value) return null;
  if (isVynthErrorCode(value)) return value;
  const def = LEGACY_FAMILY_TO_DEFAULT[value];
  return def ?? null;
}

/** 给 CLI / log 使用：`VC-030001 SANDBOX_PATH_ESCAPE` */
export function describe(code: VynthErrorCode): string {
  switch (code) {
    case 'VC-010001':
      return 'CONFIG_MISSING_KEY';
    case 'VC-010002':
      return 'CONFIG_INVALID_MODE';
    case 'VC-010003':
      return 'CONFIG_UNKNOWN_FLAG';
    case 'VC-010004':
      return 'CONFIG_VALUE_MISSING';
    case 'VC-010005':
      return 'CONFIG_FILE_SECRET';
    case 'VC-010006':
      return 'CONFIG_FILE_INVALID';
    case 'VC-020001':
      return 'LLM_AUTH_FAILED';
    case 'VC-020002':
      return 'LLM_RATE_LIMITED';
    case 'VC-020003':
      return 'LLM_NETWORK';
    case 'VC-020004':
      return 'LLM_INVALID_RESPONSE';
    case 'VC-020005':
      return 'LLM_PLAINTEXT_HTTP';
    case 'VC-030001':
      return 'SANDBOX_PATH_ESCAPE';
    case 'VC-030002':
      return 'SANDBOX_SYMLINK_ESCAPE';
    case 'VC-030003':
      return 'SANDBOX_NETWORK_BLOCKED';
    case 'VC-030004':
      return 'SANDBOX_READ_FAILED';
    case 'VC-030005':
      return 'SANDBOX_WRITE_FAILED';
    case 'VC-040001':
      return 'TOOL_NOT_FOUND';
    case 'VC-040002':
      return 'TOOL_EXECUTION_FAILED';
    case 'VC-040003':
      return 'TOOL_INVALID_ARGS';
    case 'VC-050001':
      return 'PLUGIN_LOAD_FAILED';
    case 'VC-050002':
      return 'PLUGIN_MISSING_ACTIVATE';
    case 'VC-050003':
      return 'PLUGIN_MISSING_NAME';
    case 'VC-060001':
      return 'MCP_NOT_IMPLEMENTED';
    case 'VC-060002':
      return 'MCP_PROTOCOL_PARSE';
    case 'VC-060003':
      return 'MCP_REQUEST_TIMEOUT';
  }
}

/** 测试用：列举所有已声明码（验证 ALL_CODES 与联合类型一致） */
export function allCodes(): readonly VynthErrorCode[] {
  return [...ALL_CODES];
}
