import { describe, expect, it } from 'bun:test';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { ToolRegistry } from '@zeno/engine';
import { loadPluginsWithTrust } from './trust';

const here = dirname(fileURLToPath(import.meta.url));
const fx = (f: string) => join(here, 'fixtures', f);

describe('loadPluginsWithTrust (F13 信任门禁)', () => {
  it('confirm=true 时真正加载并激活插件（信任模型：授权后导入执行）', async () => {
    const reg = new ToolRegistry();
    const res = await loadPluginsWithTrust([fx('good-plugin.ts')], reg, async () => true);
    expect(res.loaded).toEqual(['good-plugin']);
    expect(res.declined).toEqual([]);
    expect(res.errors).toEqual([]);
    expect(reg.list().some((t) => t.name === 'good_tool')).toBe(true);
  });

  it('confirm=false 时拒绝加载（不 import、不执行插件代码）', async () => {
    const reg = new ToolRegistry();
    const res = await loadPluginsWithTrust([fx('good-plugin.ts')], reg, async () => false);
    expect(res.declined).toEqual([fx('good-plugin.ts')]);
    expect(res.loaded).toEqual([]);
    expect(res.errors).toEqual([]);
    expect(reg.list().some((t) => t.name === 'good_tool')).toBe(false);
  });

  it('确认后加载失败（模块无效）归入 errors，不污染 loaded', async () => {
    const reg = new ToolRegistry();
    const res = await loadPluginsWithTrust([fx('bad-no-activate.ts')], reg, async () => true);
    expect(res.loaded).toEqual([]);
    expect(res.declined).toEqual([]);
    expect(res.errors.length).toBe(1);
    expect(res.errors[0].path).toBe(fx('bad-no-activate.ts'));
  });

  it('逐插件独立确认：一个拒绝、一个信任', async () => {
    const reg = new ToolRegistry();
    const order: boolean[] = [false, true];
    const res = await loadPluginsWithTrust(
      [fx('good-plugin.ts'), fx('good-plugin-2.ts')],
      reg,
      async () => order.shift() ?? false
    );
    expect(res.declined).toEqual([fx('good-plugin.ts')]);
    expect(res.loaded).toEqual(['good-plugin-2']);
  });
});
