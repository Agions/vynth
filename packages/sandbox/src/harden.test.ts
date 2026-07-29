import { afterEach, describe, expect, test } from 'bun:test';
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { SandboxError } from '@zeno/core';
import { buildBwrapArgs, buildSbplProfile, detectHardenBackend, spawnHardened } from './harden';
import { runCommand } from './sandbox';

let dir: string | null = null;
let backendKnown: boolean | null = null;

afterEach(() => {
  if (dir) {
    rmSync(dir, { recursive: true, force: true });
    dir = null;
  }
  process.env.ZENO_HARDEN = '';
});

const EMPTY_CMD = { command: '' } as const;

async function backendAvailable(): Promise<boolean> {
  if (backendKnown !== null) return backendKnown;
  backendKnown = await detectHardenBackend();
  return backendKnown;
}

function fresh(): string {
  dir = mkdtempSync(join(tmpdir(), 'zeno-harden-'));
  return dir;
}

describe('buildSbplProfile', () => {
  test('deny default + 关键路径允许', () => {
    const p = buildSbplProfile({ cwd: '/tmp/x', networkAllowed: true, command: '' });
    expect(p).toContain('(deny default)');
    expect(p).toContain('(allow process-exec)');
    expect(p).toContain('(subpath "/tmp/x")');
    expect(p).toContain('(allow network-outbound)');
    expect(p).not.toContain('(deny network');
  });

  test('networkAllowed=false 时 deny network', () => {
    const p = buildSbplProfile({ cwd: '/x', networkAllowed: false, command: '' });
    expect(p).toContain('(deny network-outbound)');
    expect(p).toContain('(deny network-inbound)');
    expect(p).not.toContain('(allow network');
  });
});

describe('buildBwrapArgs', () => {
  test('默认参数含 cap-drop ALL + cwd 读写', () => {
    const a = buildBwrapArgs({ cwd: '/w', networkAllowed: true, command: '' });
    expect(a).toContain('--cap-drop');
    expect(a).toContain('ALL');
    expect(a).toContain('--bind');
    expect(a).toContain('/w');
    expect(a).not.toContain('--unshare-net');
  });
  test('networkAllowed=false 时添加 --unshare-net', () => {
    const a = buildBwrapArgs({ cwd: '/w', networkAllowed: false, command: '' });
    expect(a).toContain('--unshare-net');
  });
});

describe('detectHardenBackend', () => {
  test('返回 boolean', async () => {
    const ok = await detectHardenBackend();
    expect(typeof ok).toBe('boolean');
  });
});

describe('spawnHardened 平台分流', () => {
  test('不支持平台抛 VC-030006', () => {
    if (process.platform === 'darwin' || process.platform === 'linux') return;
    expect(() =>
      spawnHardened({
        command: 'echo x',
        cwd: '/tmp',
        networkAllowed: true,
        timeoutMs: 1000,
        onStdout: () => {},
        onStderr: () => {}
      })
    ).toThrow(SandboxError);
  });
});

describe('runCommand + ZENO_HARDEN=1 端到端', () => {
  test('硬化开启 + 后端可用 + 简单命令成功', async () => {
    if ((await backendAvailable()) === false) {
      console.log('skip: 后端不可用，平台=', process.platform);
      return;
    }
    process.env.ZENO_HARDEN = '1';
    const d = fresh();
    const r = await runCommand('echo hardened-ok', {
      cwd: d,
      networkAllowed: true,
      timeoutMs: 5000
    });
    expect(r.ok).toBe(true);
    expect(r.output.trim()).toBe('hardened-ok');
    expect(r.error).toBeUndefined();
  });

  test('硬化开启 + 退出非 0 → VC-030005 (hardened)', async () => {
    if ((await backendAvailable()) === false) {
      console.log('skip: 后端不可用');
      return;
    }
    process.env.ZENO_HARDEN = '1';
    const d = fresh();
    const r = await runCommand('exit 7', { cwd: d, networkAllowed: true, timeoutMs: 5000 });
    expect(r.ok).toBe(false);
    expect(r.error).toContain('VC-030005');
    expect(r.error).toContain('(hardened)');
    expect(r.error).toContain('7');
  });

  test('硬化开启 + 写文件被允许（cwd 读写），禁止读 cwd 外的文件', async () => {
    if ((await backendAvailable()) === false) {
      console.log('skip: 后端不可用');
      return;
    }
    process.env.ZENO_HARDEN = '1';
    const d = fresh();
    writeFileSync(join(d, 'inner.txt'), 'inner');
    const r1 = await runCommand('cat inner.txt', { cwd: d, networkAllowed: true, timeoutMs: 5000 });
    expect(r1.ok).toBe(true);
    expect(r1.output.trim()).toBe('inner');
    const r2 = await runCommand('cat /etc/passwd', {
      cwd: d,
      networkAllowed: true,
      timeoutMs: 5000
    });
    expect(r2.ok).toBe(false);
    expect(r2.error).toBeDefined();
  });

  test('硬化关闭 + ZENO_HARDEN 未设 → 走原路径，不带 (hardened) 后缀', async () => {
    process.env.ZENO_HARDEN = '';
    const d = fresh();
    const r = await runCommand('exit 0', { cwd: d, networkAllowed: true, timeoutMs: 5000 });
    expect(r.ok).toBe(true);
  });
});

describe('ZENO_HARDEN=1 + 后端不可用（App Sandbox 阻断场景）', () => {
  test('sandbox-exec 拒绝 apply 时 → 启动时即报 VC-030006（不静默降级）', async () => {
    if (process.platform !== 'darwin') {
      console.log('skip: macOS 专属场景');
      return;
    }
    if ((await backendAvailable()) === true) {
      console.log('skip: 后端可 apply，本机可以跑通硬化');
      return;
    }
    process.env.ZENO_HARDEN = '1';
    const d = fresh();
    const r = await runCommand('echo x', { cwd: d, networkAllowed: true, timeoutMs: 5000 });
    expect(r.ok).toBe(false);
    expect(r.error).toContain('VC-030006');
    expect(r.error).toContain('platform=darwin');
    expect(r.error).toContain('macOS 15');
    expect(r.error).toContain('root');
  });
});

describe('runCommand 硬化失败兜底', () => {
  test('spawnHardened 抛错 → VC-030006（不静默降级）', async () => {
    if (process.platform === 'darwin' || process.platform === 'linux') {
      console.log('skip: 当前平台会进入真实 spawn 路径');
      return;
    }
    process.env.ZENO_HARDEN = '1';
    const d = fresh();
    const r = await runCommand('echo x', { cwd: d, networkAllowed: true, timeoutMs: 5000 });
    expect(r.ok).toBe(false);
    expect(r.error).toContain('VC-030006');
  });
});
