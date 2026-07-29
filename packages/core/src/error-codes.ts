export type ZenoErrorCode =
  | 'VC-010001' // CONFIG_MISSING_KEY
  | 'VC-010002' // CONFIG_INVALID_MODE
  | 'VC-010003' // CONFIG_UNKNOWN_FLAG
  | 'VC-010004'
  | 'VC-010005'
  | 'VC-010006'
  | 'VC-010099'
  | 'VC-020001' // LLM_AUTH_FAILED
  | 'VC-020002' // LLM_RATE_LIMITED
  | 'VC-020003' // LLM_NETWORK
  | 'VC-020004'
  | 'VC-020005'
  | 'VC-020099'
  | 'VC-030001'
  | 'VC-030002' // SANDBOX_SYMLINK_ESCAPE
  | 'VC-030003'
  | 'VC-030004' // SANDBOX_READ_FAILED
  | 'VC-030005' // SANDBOX_WRITE_FAILED
  | 'VC-030006'
  | 'VC-030007'
  | 'VC-030099'
  | 'VC-040001' // TOOL_NOT_FOUND
  | 'VC-040002' // TOOL_EXECUTION_FAILED
  | 'VC-040003' // TOOL_INVALID_ARGS
  | 'VC-040099'
  | 'VC-050001'
  | 'VC-050002' // PLUGIN_MISSING_ACTIVATE
  | 'VC-050003' // PLUGIN_MISSING_NAME
  | 'VC-050099'
  | 'VC-060001'
  | 'VC-060002' // MCP_PROTOCOL_PARSE
  | 'VC-060003' // MCP_REQUEST_TIMEOUT;
  | 'VC-060099';

export const DEFAULT_CODE_BY_FAMILY = {
  config: 'VC-010099',
  llm: 'VC-020099',
  sandbox: 'VC-030099',
  tool: 'VC-040099',
  plugin: 'VC-050099',
  mcp: 'VC-060099'
} as const satisfies Record<string, ZenoErrorCode>;

const ALL_CODES: ReadonlySet<ZenoErrorCode> = new Set<ZenoErrorCode>([
  'VC-010001',
  'VC-010002',
  'VC-010003',
  'VC-010004',
  'VC-010005',
  'VC-010006',
  'VC-010099',
  'VC-020001',
  'VC-020002',
  'VC-020003',
  'VC-020004',
  'VC-020005',
  'VC-020099',
  'VC-030001',
  'VC-030002',
  'VC-030003',
  'VC-030004',
  'VC-030005',
  'VC-030006',
  'VC-030007',
  'VC-030099',
  'VC-040001',
  'VC-040002',
  'VC-040003',
  'VC-040099',
  'VC-050001',
  'VC-050002',
  'VC-050003',
  'VC-050099',
  'VC-060001',
  'VC-060002',
  'VC-060003',
  'VC-060099'
]);

const CODE_RE = /^VC-\d{6}$/;

export function isZenoErrorCode(s: string): s is ZenoErrorCode {
  return CODE_RE.test(s);
}

const LEGACY_FAMILY_TO_DEFAULT: Record<string, ZenoErrorCode> = {
  config: 'VC-010099',
  llm: 'VC-020099',
  sandbox: 'VC-030099',
  tool: 'VC-040099',
  plugin: 'VC-050099',
  mcp: 'VC-060099'
};

export function fromLegacy(value: string | null | undefined): ZenoErrorCode | null {
  if (!value) return null;
  if (isZenoErrorCode(value)) return value;
  const def = LEGACY_FAMILY_TO_DEFAULT[value];
  return def ?? null;
}

export function describe(code: ZenoErrorCode): string {
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
    case 'VC-010099':
      return 'CONFIG_DEFAULT';
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
    case 'VC-020099':
      return 'LLM_DEFAULT';
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
    case 'VC-030006':
      return 'SANDBOX_HARDEN_UNAVAILABLE';
    case 'VC-030007':
      return 'SANDBOX_HARDEN_LAUNCH_FAILED';
    case 'VC-030099':
      return 'SANDBOX_DEFAULT';
    case 'VC-040001':
      return 'TOOL_NOT_FOUND';
    case 'VC-040002':
      return 'TOOL_EXECUTION_FAILED';
    case 'VC-040003':
      return 'TOOL_INVALID_ARGS';
    case 'VC-040099':
      return 'TOOL_DEFAULT';
    case 'VC-050001':
      return 'PLUGIN_LOAD_FAILED';
    case 'VC-050002':
      return 'PLUGIN_MISSING_ACTIVATE';
    case 'VC-050003':
      return 'PLUGIN_MISSING_NAME';
    case 'VC-050099':
      return 'PLUGIN_DEFAULT';
    case 'VC-060001':
      return 'MCP_NOT_IMPLEMENTED';
    case 'VC-060002':
      return 'MCP_PROTOCOL_PARSE';
    case 'VC-060003':
      return 'MCP_REQUEST_TIMEOUT';
    case 'VC-060099':
      return 'MCP_DEFAULT';
  }
}

export function allCodes(): readonly ZenoErrorCode[] {
  return [...ALL_CODES];
}
