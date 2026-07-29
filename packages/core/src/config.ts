import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { homedir } from 'node:os';
import { join } from 'node:path';
import { ConfigError, toErrorMessage } from './errors';
import type { Mode, ZenoConfig } from './types';

const MODES: Mode[] = ['plan', 'vibe', 'auto'];

interface ConfigFile {
  mode?: Mode;
  model?: string;
  llmBaseUrl?: string;
  theme?: 'mocha' | 'latte' | 'neon';
  sandbox?: { networkAllowed?: boolean; cwd?: string; harden?: boolean };
  dataDir?: string;
  audit?: boolean;
  repomap?: { enabled?: boolean; maxSymbols?: number; includeTests?: boolean };
}

const ALLOWED_KEYS = new Set([
  'mode',
  'model',
  'llmBaseUrl',
  'theme',
  'sandbox',
  'dataDir',
  'audit',
  'repomap'
]);

function loadConfigFile(dataDir: string): ConfigFile | null {
  const explicit = process.env.ZENO_CONFIG_FILE;
  if (process.env.NODE_ENV === 'test' && !explicit && dataDir === join(homedir(), '.zeno')) return null;
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
    throw new ConfigError('配置文件中不得包含 apiKey，请改用 ZENO_API_KEY 环境变量', 'VC-010005');
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

function loadProjectConfigFile(cwd: string): ConfigFile | null {
  const candidates = [join(cwd, 'zeno.json'), join(cwd, '.zenorc')];
  for (const path of candidates) {
    if (existsSync(path)) {
      try {
        const raw = JSON.parse(readFileSync(path, 'utf8'));
        if (raw && typeof raw === 'object' && !Array.isArray(raw)) {
          return raw as ConfigFile;
        }
      } catch {}
    }
  }
  return null;
}

export function loadConfig(overrides: Partial<ZenoConfig> = {}): ZenoConfig {
  const modeRaw = overrides.mode ?? (process.env.ZENO_MODE as Mode | undefined);
  const mode: Mode = modeRaw && MODES.includes(modeRaw) ? modeRaw : 'vibe';

  const netRaw = process.env.ZENO_NET;
  const envNetAllowed =
    netRaw === undefined
      ? undefined
      : netRaw === '' || !['0', 'off', 'false', 'no'].includes(netRaw.toLowerCase());

  const envHarden =
    process.env.ZENO_HARDEN === undefined ? undefined : process.env.ZENO_HARDEN === '1';

  const dataDir = overrides.dataDir ?? process.env.ZENO_DATA_DIR ?? join(homedir(), '.zeno');
  const file = loadConfigFile(dataDir);
  const cwd = overrides.sandbox?.cwd ?? file?.sandbox?.cwd ?? process.cwd();
  const projectFile = loadProjectConfigFile(cwd);

  const envThemeRaw = process.env.ZENO_THEME;
  const envTheme =
    envThemeRaw === undefined
      ? undefined
      : envThemeRaw === 'latte'
        ? 'latte'
        : envThemeRaw === 'neon'
          ? 'neon'
          : 'mocha';
  const envAudit =
    process.env.ZENO_AUDIT === undefined ? undefined : process.env.ZENO_AUDIT === '1';

  const envRepoMap =
    process.env.ZENO_REPOMAP === undefined
      ? undefined
      : !['0', 'off', 'false', 'no'].includes(process.env.ZENO_REPOMAP.toLowerCase());
  const envRepoMapMaxRaw = process.env.ZENO_REPOMAP_MAX;
  const envRepoMapMax =
    envRepoMapMaxRaw && !Number.isNaN(Number(envRepoMapMaxRaw))
      ? Number(envRepoMapMaxRaw)
      : undefined;

  const config: ZenoConfig = {
    mode,
    llmBaseUrl:
      overrides.llmBaseUrl ??
      process.env.ZENO_LLM_BASE_URL ??
      projectFile?.llmBaseUrl ??
      file?.llmBaseUrl ??
      'https://api.deepseek.com/v1',
    apiKey: overrides.apiKey ?? process.env.ZENO_API_KEY ?? '',
    model: overrides.model ?? process.env.ZENO_MODEL ?? projectFile?.model ?? file?.model ?? 'deepseek-v4-pro',
    theme: overrides.theme ?? envTheme ?? projectFile?.theme ?? file?.theme ?? 'mocha',
    sandbox: {
      networkAllowed:
        overrides.sandbox?.networkAllowed ?? envNetAllowed ?? projectFile?.sandbox?.networkAllowed ?? file?.sandbox?.networkAllowed ?? true,
      harden:
        overrides.sandbox?.harden ?? envHarden ?? projectFile?.sandbox?.harden ?? file?.sandbox?.harden ?? false,
      cwd
    },
    dataDir,
    audit: overrides.audit ?? envAudit ?? file?.audit ?? false,
    repomap: {
      enabled: overrides.repomap?.enabled ?? envRepoMap ?? file?.repomap?.enabled ?? true,
      maxSymbols:
        overrides.repomap?.maxSymbols ??
        envRepoMapMax ??
        file?.repomap?.maxSymbols ??
        400,
      includeTests: overrides.repomap?.includeTests ?? file?.repomap?.includeTests ?? false
    }
  };

  return config;
}

export function saveConfigFile(dataDir: string, updates: Partial<ConfigFile>): void {
  const path = process.env.ZENO_CONFIG_FILE || join(dataDir, 'config.json');
  try {
    mkdirSync(dataDir, { recursive: true });
    let existing: Record<string, unknown> = {};
    if (existsSync(path)) {
      try {
        existing = JSON.parse(readFileSync(path, 'utf8'));
      } catch {
        existing = {};
      }
    }
    const cleanUpdates = { ...updates };
    delete (cleanUpdates as Record<string, unknown>).apiKey; // Safety redline
    const merged = { ...existing, ...cleanUpdates };
    writeFileSync(path, JSON.stringify(merged, null, 2), 'utf8');
  } catch (err) {
    throw new ConfigError(`保存配置文件失败: ${toErrorMessage(err)}`, 'VC-010006');
  }
}

