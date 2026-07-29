import { afterEach, describe, expect, it } from 'bun:test';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { getTaskManager } from './tasks';

const dirs: string[] = [];
async function tmp(): Promise<string> {
  const d = await mkdtemp(join(tmpdir(), 'zeno-task-'));
  dirs.push(d);
  return d;
}
afterEach(async () => {
  while (dirs.length) {
    const d = dirs.pop();
    if (d) await rm(d, { recursive: true, force: true });
  }
});

function waitUntil(fn: () => boolean, timeoutMs = 5000): Promise<void> {
  return new Promise((resolve, reject) => {
    const start = Date.now();
    const tick = () => {
      if (fn()) return resolve();
      if (Date.now() - start > timeoutMs) return reject(new Error('timeout'));
      setTimeout(tick, 10);
    };
    tick();
  });
}

describe('background task manager', () => {
  it('runs a foreground-success command and captures output', async () => {
    const dir = await tmp();
    const mgr = getTaskManager();
    const t = mgr.spawn('echo hello-world', dir, true);
    await waitUntil(() => t.finishedAt !== null);
    expect(t.status).toBe('done');
    expect(t.exitCode).toBe(0);
    expect(t.output).toContain('hello-world');
  });

  it('marks non-zero exit as failed with the right code', async () => {
    const dir = await tmp();
    const mgr = getTaskManager();
    const t = mgr.spawn('exit 3', dir, true);
    await waitUntil(() => t.finishedAt !== null);
    expect(t.status).toBe('failed');
    expect(t.exitCode).toBe(3);
  });

  it('blocks when network is disallowed', async () => {
    const dir = await tmp();
    const mgr = getTaskManager();
    const t = mgr.spawn('echo nope', dir, false);
    expect(t.status).toBe('failed');
    expect(t.output).toContain('network blocked');
  });

  it('tracks running count and resets on completion', async () => {
    const dir = await tmp();
    const mgr = getTaskManager();
    const t = mgr.spawn('sleep 0.2; echo done', dir, true);
    expect(mgr.runningCount()).toBeGreaterThanOrEqual(1);
    await waitUntil(() => t.finishedAt !== null);
    expect(mgr.runningCount()).toBe(0);
  });
});
