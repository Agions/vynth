import { afterEach, beforeEach, describe, expect, test } from 'bun:test';
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { loadConfig } from './config';
import { ConfigError } from './errors';

let dir: string;
const savedEnv: Record<string, string | undefined> = {};

function setEnv(key: string, value: string | undefined): void {
  savedEnv[key] = process.env[key];
  if (value === undefined) delete process.env[key];
  else process.env[key] = value;
}

beforeEach(() => {
  dir = mkdtempSync(join(tmpdir(), 'zeno-cfg-'));
  setEnv('VYNTH_API_KEY', undefined);
  setEnv('VYNTH_MODEL', undefined);
  setEnv('VYNTH_THEME', undefined);
  setEnv('VYNTH_NET', undefined);
  setEnv('VYNTH_AUDIT', undefined);
  setEnv('VYNTH_HARDEN', undefined);
  setEnv('VYNTH_CONFIG_FILE', undefined);
  setEnv('VYNTH_DATA_DIR', dir);
});
afterEach(() => {
  rmSync(dir, { recursive: true, force: true });
  for (const [k, v] of Object.entries(savedEnv)) setEnv(k, v);
});

describe('loadConfig 可选配置文件（F14）', () => {
  test('配置文件不存在时回退默认（向后兼容）', () => {
    const c = loadConfig();
    expect(c.model).toBe('deepseek-v4-pro');
    expect(c.audit).toBe(false);
  });

  test('配置文件与 env 合并：env 始终优先于文件', () => {
    writeFileSync(join(dir, 'config.json'), JSON.stringify({ model: 'from-file', theme: 'latte' }));
    setEnv('VYNTH_MODEL', 'from-env');
    const c = loadConfig();
    expect(c.model).toBe('from-env');
    expect(c.theme).toBe('latte');
  });

  test('文件审计标志生效', () => {
    writeFileSync(join(dir, 'config.json'), JSON.stringify({ audit: true }));
    expect(loadConfig().audit).toBe(true);
  });

  test('VYNTH_AUDIT=1 优先于文件 audit:false', () => {
    writeFileSync(join(dir, 'config.json'), JSON.stringify({ audit: false }));
    setEnv('VYNTH_AUDIT', '1');
    expect(loadConfig().audit).toBe(true);
  });

  test('文件中含 apiKey 抛 VC-010005', () => {
    writeFileSync(join(dir, 'config.json'), JSON.stringify({ apiKey: 'sk-xxx' }));
    expect(() => loadConfig()).toThrow(ConfigError);
    try {
      loadConfig();
    } catch (e) {
      expect((e as ConfigError).numericCode).toBe('VC-010005');
    }
  });

  test('非法 JSON 抛 VC-010006', () => {
    writeFileSync(join(dir, 'config.json'), '{ not json');
    expect(() => loadConfig()).toThrow(ConfigError);
    try {
      loadConfig();
    } catch (e) {
      expect((e as ConfigError).numericCode).toBe('VC-010006');
    }
  });

  test('未知键抛 VC-010006', () => {
    writeFileSync(join(dir, 'config.json'), JSON.stringify({ bogus: 1 }));
    expect(() => loadConfig()).toThrow(ConfigError);
    try {
      loadConfig();
    } catch (e) {
      expect((e as ConfigError).numericCode).toBe('VC-010006');
    }
  });

  test('VYNTH_CONFIG_FILE 指向显式路径', () => {
    const p = join(dir, 'explicit.json');
    writeFileSync(p, JSON.stringify({ model: 'explicit-model' }));
    setEnv('VYNTH_CONFIG_FILE', p);
    expect(loadConfig().model).toBe('explicit-model');
  });

  describe('sandbox.harden（OS 硬隔离开关）', () => {
    test('默认 false（未配置）', () => {
      expect(loadConfig().sandbox.harden).toBe(false);
    });

    test('配置文件 sandbox.harden:true 生效', () => {
      writeFileSync(join(dir, 'config.json'), JSON.stringify({ sandbox: { harden: true } }));
      expect(loadConfig().sandbox.harden).toBe(true);
    });

    test('VYNTH_HARDEN=1 优先于文件 harden:false（安全闸门）', () => {
      writeFileSync(join(dir, 'config.json'), JSON.stringify({ sandbox: { harden: false } }));
      setEnv('VYNTH_HARDEN', '1');
      expect(loadConfig().sandbox.harden).toBe(true);
    });

    test('VYNTH_HARDEN=0 优先于文件 harden:true（安全闸门，可强制关闭）', () => {
      writeFileSync(join(dir, 'config.json'), JSON.stringify({ sandbox: { harden: true } }));
      setEnv('VYNTH_HARDEN', '0');
      expect(loadConfig().sandbox.harden).toBe(false);
    });

    test('env 未设时回落文件 harden:true', () => {
      writeFileSync(join(dir, 'config.json'), JSON.stringify({ sandbox: { harden: true } }));
      expect(loadConfig().sandbox.harden).toBe(true);
    });
  });
});
