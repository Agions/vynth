import { afterEach, beforeEach, describe, expect, test } from 'bun:test';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { audit, initAudit, resetAudit } from './audit';

let dir: string;

beforeEach(() => {
  dir = mkdtempSync(join(tmpdir(), 'vynth-audit-'));
});
afterEach(() => {
  resetAudit();
  rmSync(dir, { recursive: true, force: true });
});

describe('AuditLog（F14 审计）', () => {
  test('enabled 时 record 写入 JSONL 且字段完整', () => {
    initAudit({ dataDir: dir, audit: true });
    audit().record('tool_exec', { name: 'read_file', ok: true }, true);
    audit().record('file_access', { op: 'write', ok: false }, false);

    const all = audit().readAllSync();
    expect(all).toHaveLength(2);
    expect(all[0]?.kind).toBe('tool_exec');
    expect(all[0]?.ok).toBe(true);
    expect(all[0]?.detail).toEqual({ name: 'read_file', ok: true });
    expect(typeof all[0]?.ts).toBe('string');
    expect(all[1]?.kind).toBe('file_access');
    expect(all[1]?.ok).toBe(false);
  });

  test('disabled 时 record 为 no-op（不创建文件）', () => {
    initAudit({ dataDir: dir, audit: false });
    audit().record('tool_exec', { name: 'x' });
    expect(audit().enabled).toBe(false);
    expect(audit().readAllSync()).toEqual([]);
  });

  test('未 initAudit 时单例为 no-op（record 不抛错）', () => {
    resetAudit();
    expect(() => audit().record('tool_exec', {})).not.toThrow();
  });
});
