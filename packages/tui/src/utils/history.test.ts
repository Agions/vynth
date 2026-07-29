import { afterEach, describe, expect, it } from 'bun:test';
import { mkdtemp, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { historyPath, loadHistory, saveHistory } from './history';

const dirs: string[] = [];
async function tmp(): Promise<string> {
  const d = await mkdtemp(join(tmpdir(), 'zeno-hist-'));
  dirs.push(d);
  return d;
}
afterEach(async () => {
  while (dirs.length) {
    const d = dirs.pop();
    if (d) await rm(d, { recursive: true, force: true });
  }
});

describe('history persistence', () => {
  it('round-trips entries to disk and dedups adjacent repeats', async () => {
    const dir = await tmp();
    saveHistory(dir, ['a', 'a', 'b', 'c']);
    const loaded = loadHistory(dir);
    expect(loaded).toEqual(['a', 'b', 'c']);
  });

  it('returns [] when no file exists', async () => {
    const dir = await tmp();
    expect(loadHistory(dir)).toEqual([]);
  });

  it('caps to the most recent max entries', async () => {
    const dir = await tmp();
    const many = Array.from({ length: 10 }, (_, i) => `cmd${i}`);
    saveHistory(dir, many, 3);
    expect(loadHistory(dir)).toEqual(['cmd7', 'cmd8', 'cmd9']);
  });

  it('tolerates corrupt JSON by returning []', async () => {
    const dir = await tmp();
    await writeFile(historyPath(dir), '{not valid json', 'utf8');
    expect(loadHistory(dir)).toEqual([]);
  });

  it('creates the dataDir if missing on save', () => {
    const dir = join(tmpdir(), `zeno-hist-${Date.now()}-nested/deep`);
    dirs.push(dir);
    saveHistory(dir, ['x']);
    expect(loadHistory(dir)).toEqual(['x']);
  });
});
