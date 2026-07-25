import { homedir } from 'node:os';
import { join } from 'node:path';
import type { Mode, VynthConfig } from './types';

const MODES: Mode[] = ['plan', 'vibe'];

export function loadConfig(overrides: Partial<VynthConfig> = {}): VynthConfig {
  const modeRaw = overrides.mode ?? (process.env.VYNTH_MODE as Mode | undefined);
  const mode: Mode = modeRaw && MODES.includes(modeRaw) ? modeRaw : 'vibe';

  const netRaw = process.env.VYNTH_NET;
  const networkAllowed =
    overrides.sandbox?.networkAllowed ??
    (netRaw === undefined || netRaw === ''
      ? true
      : !['0', 'off', 'false', 'no'].includes(netRaw.toLowerCase()));

  const config: VynthConfig = {
    mode,
    llmBaseUrl:
      overrides.llmBaseUrl ?? process.env.VYNTH_LLM_BASE_URL ?? 'https://api.deepseek.com/v1',
    apiKey: overrides.apiKey ?? process.env.VYNTH_API_KEY ?? '',
    model: overrides.model ?? process.env.VYNTH_MODEL ?? 'deepseek-v4-pro',
    theme: overrides.theme ?? (process.env.VYNTH_THEME === 'latte' ? 'latte' : 'mocha'),
    sandbox: {
      networkAllowed,
      cwd: overrides.sandbox?.cwd ?? process.cwd()
    },
    dataDir: overrides.dataDir ?? process.env.VYNTH_DATA_DIR ?? join(homedir(), '.vynth')
  };

  return config;
}
