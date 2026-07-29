import { afterAll, beforeAll, describe, expect, test } from 'bun:test';
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { buildRepoMap } from './repomap';

let root: string;

beforeAll(() => {
  root = mkdtempSync(join(tmpdir(), 'zeno-repomap-'));
  writeFileSync(
    join(root, 'sample.ts'),
    [
      'export class Service {',
      '  private x = 1;',
      '  async fetchData(id: number) {',
      '    return Service.helper(id);',
      '  }',
      '  static helper(id: number) { return id; }',
      '}',
      'export function bootstrap() {',
      '  const s = new Service();',
      '  return s.fetchData(1);',
      '}',
      'export interface Repo { id: number; }',
      'export type Id = number;',
      'export const VERSION = "1.0";',
      'export enum Color { Red, Green }'
    ].join('\n')
  );
  writeFileSync(
    join(root, 'main.go'),
    [
      'package main',
      'type Server struct {',
      '  addr string',
      '}',
      'func (s *Server) Start() {',
      '  s.addr = "127.0.0.1"',
      '}',
      'func NewServer() *Server {',
      '  return &Server{}',
      '}'
    ].join('\n')
  );
  writeFileSync(
    join(root, 'mod.py'),
    [
      'class Engine:',
      '    def run(self):',
      '        return self.run()',
      'def helper():',
      '    return 42'
    ].join('\n')
  );
  writeFileSync(
    join(root, 'lib.rs'),
    [
      'pub struct Cache {',
      '  size: usize,',
      '}',
      'impl Cache {',
      '  pub fn get(&self, k: u64) -> u64 { k }',
      '}',
      'pub fn build() -> Cache { Cache { size: 0 } }'
    ].join('\n')
  );
});

afterAll(() => {
  rmSync(root, { recursive: true, force: true });
});

describe('repo-map 提取器', () => {
  test('TS/JS 提取 class/function/method/interface/type/enum/const', async () => {
    const res = await buildRepoMap({ root, maxSymbols: 100 });
    const names = res.symbols.map((s) => s.name);
    expect(names).toContain('Service');
    expect(names).toContain('bootstrap');
    expect(names).toContain('fetchData'); // method
    expect(names).toContain('Repo'); // interface
    expect(names).toContain('Id'); // type
    expect(names).toContain('VERSION'); // const
    expect(names).toContain('Color'); // enum
    const fetch = res.symbols.find((s) => s.name === 'fetchData');
    expect(fetch?.kind).toBe('method');
    expect(fetch?.parent).toBe('Service');
    const helper = res.symbols.find((s) => s.name === 'helper');
    expect(helper?.kind).toBe('method');
  });

  test('Go 提取 func/method/struct', async () => {
    const res = await buildRepoMap({ root, maxSymbols: 100 });
    const start = res.symbols.find((s) => s.name === 'Start');
    expect(start?.kind).toBe('method');
    expect(start?.parent).toBe('Server');
    expect(res.symbols.find((s) => s.name === 'NewServer')?.kind).toBe('function');
    expect(res.symbols.find((s) => s.name === 'Server')?.kind).toBe('struct');
  });

  test('Python 提取 class/method/function', async () => {
    const res = await buildRepoMap({ root, maxSymbols: 100 });
    const run = res.symbols.find((s) => s.name === 'run');
    expect(run?.kind).toBe('method');
    expect(run?.parent).toBe('Engine');
    expect(res.symbols.find((s) => s.file === 'mod.py' && s.name === 'helper')?.kind).toBe('function');
    expect(res.symbols.find((s) => s.name === 'Engine')?.kind).toBe('class');
  });

  test('Rust 提取 fn/method/struct/impl', async () => {
    const res = await buildRepoMap({ root, maxSymbols: 100 });
    const get = res.symbols.find((s) => s.name === 'get');
    expect(get?.kind).toBe('method');
    expect(get?.parent).toBe('Cache');
    expect(res.symbols.find((s) => s.name === 'build')?.kind).toBe('function');
    expect(res.symbols.find((s) => s.name === 'Cache')?.kind).toBe('struct');
  });

  test('跨文件引用计数排名（高频符号优先）', async () => {
    const res = await buildRepoMap({ root, maxSymbols: 3 });
    const top = res.ranked[0];
    expect(top).toBeDefined();
    expect(res.symbolCount).toBeGreaterThan(0);
    expect(res.mapText).toContain('repo-map');
    expect(res.mapText).toContain('sample.ts');
  });

  test('测试文件默认被排除', async () => {
    writeFileSync(
      join(root, 'sample.test.ts'),
      ['export function ignored() { return 1; }'].join('\n')
    );
    const res = await buildRepoMap({ root, maxSymbols: 100 });
    expect(res.symbols.find((s) => s.name === 'ignored')).toBeUndefined();
    const res2 = await buildRepoMap({ root, maxSymbols: 100, includeTests: true });
    expect(res2.symbols.find((s) => s.name === 'ignored')).toBeDefined();
  });
});
