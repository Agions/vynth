import { afterAll, beforeAll, describe, expect, it } from 'bun:test';
import { mkdir, mkdtemp, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { createFile, grepSearch, listFiles } from './fsops';

let root: string;

beforeAll(async () => {
  root = await mkdtemp(join(tmpdir(), 'zeno-fsops-'));
  await mkdir(join(root, 'src', 'utils'), { recursive: true });
  await mkdir(join(root, 'node_modules', 'junk'), { recursive: true });
  await writeFile(join(root, 'README.md'), '# Hello Zeno\n', 'utf8');
  await writeFile(join(root, 'src', 'index.ts'), 'export const x = 42;\nconsole.log(x);\n', 'utf8');
  await writeFile(
    join(root, 'src', 'utils', 'calc.ts'),
    'export function add(a: number, b: number) {\n  return a + b;\n}\n',
    'utf8'
  );
  await writeFile(join(root, 'node_modules', 'junk', 'a.ts'), 'should not appear\n', 'utf8');
});

afterAll(async () => {
  await rm(root, { recursive: true, force: true });
});

describe('listFiles', () => {
  it('lists project tree and skips node_modules', async () => {
    const res = await listFiles('.', root);
    expect(res.ok).toBe(true);
    expect(res.output).toContain('src/index.ts');
    expect(res.output).toContain('src/utils/calc.ts');
    expect(res.output).toContain('README.md');
    expect(res.output).not.toContain('junk');
  });

  it('rejects path escaping sandbox', async () => {
    const res = await listFiles('../..', root);
    expect(res.ok).toBe(false);
    expect(res.error).toContain('VC-03000');
  });
});

describe('grepSearch', () => {
  it('finds literal substring with path:line', async () => {
    const res = await grepSearch('return a + b', root);
    expect(res.ok).toBe(true);
    expect(res.output).toContain('src/utils/calc.ts:2');
  });

  it('is case-insensitive by default', async () => {
    const res = await grepSearch('HELLO', root);
    expect(res.ok).toBe(true);
    expect(res.output).toContain('README.md:1');
  });

  it('supports regex', async () => {
    const res = await grepSearch('export (const|function)', root, { regex: true });
    expect(res.ok).toBe(true);
    expect(res.output).toContain('src/index.ts:1');
    expect(res.output).toContain('src/utils/calc.ts:1');
  });

  it('honors include filter', async () => {
    const res = await grepSearch('Hello', root, { include: '.ts' });
    expect(res.ok).toBe(true);
    expect(res.output).not.toContain('README.md');
  });

  it('reports no matches cleanly', async () => {
    const res = await grepSearch('zzz_not_present_zzz', root);
    expect(res.ok).toBe(true);
    expect(res.output).toContain('no matches');
  });

  it('rejects invalid regex', async () => {
    const res = await grepSearch('(', root, { regex: true });
    expect(res.ok).toBe(false);
    expect(res.error).toContain('非法正则');
  });
});

describe('createFile', () => {
  it('creates file with parent dirs', async () => {
    const res = await createFile('newdir/deep/file.txt', 'hi', root);
    expect(res.ok).toBe(true);
    const verify = await listFiles('newdir', root);
    expect(verify.output).toContain('deep/file.txt');
  });

  it('refuses to overwrite existing file', async () => {
    const res = await createFile('README.md', 'overwrite?', root);
    expect(res.ok).toBe(false);
    expect(res.error).toContain('已存在');
  });

  it('rejects path escaping sandbox', async () => {
    const res = await createFile('../escape.txt', 'x', root);
    expect(res.ok).toBe(false);
    expect(res.error).toContain('VC-03000');
  });
});
