import { afterEach, beforeEach, describe, expect, it } from 'bun:test';
import { loadConfig } from './config';
import { ConfigError, LlmError, SandboxError, ToolError, ZenoError } from './errors';
import { Emitter } from './events';
import { log, setLogLevel } from './logger';

const SAVED = { ...process.env };

beforeEach(() => {
  for (const k of Object.keys(process.env)) {
    if (k.startsWith('ZENO_')) delete process.env[k];
  }
});

afterEach(() => {
  process.env = { ...SAVED };
});

describe('loadConfig 默认值（冻结 X1/X2）', () => {
  it('默认模型 deepseek-v4-pro、端点 api.deepseek.com/v1', () => {
    const c = loadConfig();
    expect(c.model).toBe('deepseek-v4-pro');
    expect(c.llmBaseUrl).toBe('https://api.deepseek.com/v1');
  });

  it('ZENO_API_KEY 缺省为空串（createProvider 时会抛出 LlmError）', () => {
    expect(loadConfig().apiKey).toBe('');
  });

  it('ZENO_MODE 缺省回退 vibe', () => {
    expect(loadConfig().mode).toBe('vibe');
  });

  it('ZENO_MODE=plan 生效', () => {
    process.env.ZENO_MODE = 'plan';
    expect(loadConfig().mode).toBe('plan');
  });

  it('ZENO_MODE 非法值回退 vibe', () => {
    process.env.ZENO_MODE = 'bogus';
    expect(loadConfig().mode).toBe('vibe');
  });

  it('ZENO_THEME=latte 生效，否则默认 mocha', () => {
    expect(loadConfig().theme).toBe('mocha');
    process.env.ZENO_THEME = 'latte';
    expect(loadConfig().theme).toBe('latte');
  });
});

describe('ZENO_NET 出站开关解析', () => {
  it('未设 → 放行（true）', () => {
    expect(loadConfig().sandbox.networkAllowed).toBe(true);
  });

  it('空串 → 视为未设（true）', () => {
    process.env.ZENO_NET = '';
    expect(loadConfig().sandbox.networkAllowed).toBe(true);
  });

  it('off / 0 / false / no → 拒绝（false）', () => {
    for (const v of ['off', '0', 'false', 'no']) {
      process.env.ZENO_NET = v;
      expect(loadConfig().sandbox.networkAllowed).toBe(false);
    }
  });
});

describe('ZenoError 错误码体系', () => {
  it('基类携带 code 字符串', () => {
    const e = new ZenoError('config', 'boom');
    expect(e).toBeInstanceOf(Error);
    expect(e.code).toBe('config');
    expect(e.message).toBe('boom');
  });

  it('子类落地域码（current: 字符串域，6 位 VC-XXXXXX 待迁移）', () => {
    expect(new ConfigError('x').code).toBe('config');
    expect(new LlmError('x').code).toBe('llm');
    expect(new ToolError('x').code).toBe('tool');
    expect(new SandboxError('x').code).toBe('sandbox');
  });

  it('子类可经 instanceof 向上识别', () => {
    expect(new ConfigError('x')).toBeInstanceOf(ZenoError);
  });
});

describe('Emitter 事件总线', () => {
  it('on/emit 投递，off 取消订阅', () => {
    const em = new Emitter<{ tick: number }>();
    const seen: number[] = [];
    const off = em.on('tick', (n) => seen.push(n));
    em.emit('tick', 1);
    off();
    em.emit('tick', 2);
    expect(seen).toEqual([1]);
  });

  it('未订阅的 key 不抛错', () => {
    const em = new Emitter<{ a: number }>();
    expect(() => em.emit('a', 1)).not.toThrow();
  });
});

describe('log 日志分级', () => {
  it('低于当前级别被抑制', () => {
    setLogLevel('warn');
    const lines: string[] = [];
    const orig = console.error;
    console.error = (...a: unknown[]) => lines.push(a.join(' '));
    try {
      log('debug', 'd');
      log('info', 'i');
      log('warn', 'w');
      log('error', 'e');
    } finally {
      console.error = orig;
    }
    expect(lines.some((l) => l.includes('[WARN]'))).toBe(true);
    expect(lines.some((l) => l.includes('[ERROR]'))).toBe(true);
    expect(lines.some((l) => l.includes('[INFO]'))).toBe(false);
    expect(lines.some((l) => l.includes('[DEBUG]'))).toBe(false);
  });
});
