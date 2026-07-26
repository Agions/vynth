import { existsSync, readFileSync } from 'node:fs';
import { homedir } from 'node:os';
import { join } from 'node:path';
import { ConfigError } from './errors';
import type { Mode, VynthConfig } from './types';

const MODES: Mode[] = ['plan', 'vibe'];

/** 配置文件中允许出现的键（不含 apiKey：密钥只允许走环境变量）。 */
interface ConfigFile {
  mode?: Mode;
  model?: string;
  llmBaseUrl?: string;
  theme?: 'mocha' | 'latte';
  sandbox?: { networkAllowed?: boolean; cwd?: string };
  dataDir?: string;
  audit?: boolean;
}

const ALLOWED_KEYS = new Set([
  'mode',
  'model',
  'llmBaseUrl',
  'theme',
  'sandbox',
  'dataDir',
  'audit'
]);

/**
 * 读取可选配置文件（F14，ADR-0003 受控扩展）。
 *
 * 查找顺序：
 *   1. `VYNTH_CONFIG_FILE` 显式路径
 *   2. `<dataDir>/config.json`
 *
 * 文件层优先级低于环境变量与代码 overrides：env 始终最高优先级，配置文件仅作便利补充。
 * 文件不存在时返回 `null`（不影响既有行为）。文件中含 `apiKey` 或非法 schema 时抛 ConfigError。
 */
function loadConfigFile(dataDir: string): ConfigFile | null {
  const explicit = process.env.VYNTH_CONFIG_FILE;
  const path = explicit && explicit.trim().length > 0 ? explicit : join(dataDir, 'config.json');
  if (!existsSync(path)) return null;

  let raw: unknown;
  try {
    raw = JSON.parse(readFileSync(path, 'utf8'));
  } catch (err) {
    throw new ConfigError(
      `配置文件解析失败: ${err instanceof Error ? err.message : String(err)}`,
      'VC-010006'
    );
  }
  if (raw === null || typeof raw !== 'object' || Array.isArray(raw)) {
    throw new ConfigError('配置文件必须是一个对象', 'VC-010006');
  }
  const obj = raw as Record<string, unknown>;

  if ('apiKey' in obj) {
    throw new ConfigError('配置文件中不得包含 apiKey，请改用 VYNTH_API_KEY 环境变量', 'VC-010005');
  }
  for (const key of Object.keys(obj)) {
    if (!ALLOWED_KEYS.has(key)) {
      throw new ConfigError(`配置文件含未知键: ${key}`, 'VC-010006');
    }
  }
  if ('sandbox' in obj && obj.sandbox !== null && typeof obj.sandbox !== 'object') {
    throw new ConfigError('配置文件中 sandbox 必须是对象', 'VC-010006');
  }
  return obj as ConfigFile;
}

export function loadConfig(overrides: Partial<VynthConfig> = {}): VynthConfig {
  const modeRaw = overrides.mode ?? (process.env.VYNTH_MODE as Mode | undefined);
  const mode: Mode = modeRaw && MODES.includes(modeRaw) ? modeRaw : 'vibe';

  // 网络开关：env 未设置时才回落文件 / 默认（env 作为安全闸门始终优先）
  const netRaw = process.env.VYNTH_NET;
  const envNetAllowed =
    netRaw === undefined
      ? undefined
      : netRaw === '' || !['0', 'off', 'false', 'no'].includes(netRaw.toLowerCase());

  // dataDir 先确定（决定配置文件查找路径），再读文件层
  const dataDir = overrides.dataDir ?? process.env.VYNTH_DATA_DIR ?? join(homedir(), '.vynth');
  const file = loadConfigFile(dataDir);

  const envTheme =
    process.env.VYNTH_THEME === undefined
      ? undefined
      : process.env.VYNTH_THEME === 'latte'
        ? 'latte'
        : 'mocha';
  const envAudit =
    process.env.VYNTH_AUDIT === undefined ? undefined : process.env.VYNTH_AUDIT === '1';

  const config: VynthConfig = {
    mode,
    llmBaseUrl:
      overrides.llmBaseUrl ??
      process.env.VYNTH_LLM_BASE_URL ??
      file?.llmBaseUrl ??
      'https://api.deepseek.com/v1',
    apiKey: overrides.apiKey ?? process.env.VYNTH_API_KEY ?? '',
    model: overrides.model ?? process.env.VYNTH_MODEL ?? file?.model ?? 'deepseek-v4-pro',
    theme: overrides.theme ?? envTheme ?? file?.theme ?? 'mocha',
    sandbox: {
      networkAllowed:
        overrides.sandbox?.networkAllowed ?? envNetAllowed ?? file?.sandbox?.networkAllowed ?? true,
      cwd: overrides.sandbox?.cwd ?? file?.sandbox?.cwd ?? process.cwd()
    },
    dataDir,
    audit: overrides.audit ?? envAudit ?? file?.audit ?? false
  };

  return config;
}
