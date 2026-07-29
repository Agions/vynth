import { afterEach, describe, expect, test } from 'bun:test';
import { mkdtempSync, rmSync, symlinkSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { readText, runCommand, writeText } from './sandbox';

let dir: string | null = null;
let outside: string | null = null;

afterEach(() => {
  if (dir) {
    rmSync(dir, { recursive: true, force: true });
    dir = null;
  }
  if (outside) {
    rmSync(outside, { recursive: true, force: true });
    outside = null;
  }
});

function fresh(): string {
  dir = mkdtempSync(join(tmpdir(), 'zeno-sb-'));
  return dir;
}

describe('safeResolve / 沙箱边界（F10 对抗 X3）', () => {
  test('cwd 内文件可读（F5）', async () => {
    const d = fresh();
    writeFileSync(join(d, 'note.txt'), 'hello-sandbox');
    const r = await readText('note.txt', d);
    expect(r.ok).toBe(true);
    expect(r.output).toContain('hello-sandbox');
  });

  test('拒绝 `../` 路径穿越', async () => {
    const d = fresh();
    const r = await readText('../escaped.txt', d);
    expect(r.ok).toBe(false);
    expect(r.error ?? '').toContain('escapes sandbox');
  });

  test('拒绝 cwd 外的绝对路径', async () => {
    const d = fresh();
    const r = await readText('/etc/passwd', d);
    expect(r.ok).toBe(false);
    expect(r.error ?? '').toContain('escapes sandbox');
  });

  test('拒绝经 symlink 逃逸到 cwd 外（F10）', async () => {
    const d = fresh();
    outside = mkdtempSync(join(tmpdir(), 'zeno-out-'));
    writeFileSync(join(outside, 'secret.txt'), 'TOPSECRET');
    symlinkSync(outside, join(d, 'escape'));
    const r = await readText('escape/secret.txt', d);
    expect(r.ok).toBe(false);
    expect(r.error ?? '').toContain('symlink');
  });

  test('writeText 同样受 safeResolve 约束（F5）', async () => {
    const d = fresh();
    const r = await writeText('../evil.txt', 'x', d);
    expect(r.ok).toBe(false);
    expect(r.error ?? '').toContain('escapes sandbox');
  });
});

describe('runCommand 联网闸门（F5 / ZENO_NET）', () => {
  test('ZENO_NET=off 时任何 shell 被拦截', async () => {
    const d = fresh();
    const r = await runCommand('echo hi', { cwd: d, networkAllowed: false });
    expect(r.ok).toBe(false);
    expect(r.error ?? '').toContain('network blocked');
  });

  test('允许联网时命令正常执行', async () => {
    const d = fresh();
    const r = await runCommand('echo ok-from-sandbox', { cwd: d, networkAllowed: true });
    expect(r.ok).toBe(true);
    expect(r.output).toContain('ok-from-sandbox');
  });
});
