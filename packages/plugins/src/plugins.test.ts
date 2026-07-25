import { describe, expect, it } from 'bun:test';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { PluginError } from '@vynth/core';
import { ToolRegistry } from '@vynth/engine';
import { loadAll, loadPlugin } from './loader';

const here = dirname(fileURLToPath(import.meta.url));
const fx = (f: string) => join(here, 'fixtures', f);

describe('plugin headless loading (F9)', () => {
  it('loadPlugin returns {name, activate} for a valid plugin module', async () => {
    const p = await loadPlugin(fx('good-plugin.ts'));
    expect(p.name).toBe('good-plugin');
    expect(typeof p.activate).toBe('function');
  });

  it('activate() registers tools into a ToolRegistry without any TUI (headless)', async () => {
    const p = await loadPlugin(fx('good-plugin.ts'));
    const reg = new ToolRegistry();
    p.activate(reg);
    expect(reg.get('good_tool')).toMatchObject({ name: 'good_tool' });
    const res = await reg.run('good_tool', { x: 'hi' });
    expect(res.ok).toBe(true);
    expect(res.output).toContain('good:hi');
  });

  it('loadAll loads every entry, activates, and returns plugin names', async () => {
    const reg = new ToolRegistry();
    const names = await loadAll([fx('good-plugin.ts'), fx('good-plugin-2.ts')], reg);
    expect(names).toEqual(['good-plugin', 'good-plugin-2']);
    expect(reg.get('good_tool')).toBeDefined();
    expect(reg.get('good_tool_2')).toBeDefined();
  });

  it('loadPlugin throws PluginError when activate is missing', async () => {
    await expect(loadPlugin(fx('bad-no-activate.ts'))).rejects.toBeInstanceOf(PluginError);
  });

  it('loadPlugin throws PluginError when pluginName is missing', async () => {
    await expect(loadPlugin(fx('bad-no-name.ts'))).rejects.toBeInstanceOf(PluginError);
  });

  it('loadPlugin throws PluginError when the module path cannot be imported', async () => {
    await expect(loadPlugin(fx('does-not-exist.ts'))).rejects.toBeInstanceOf(PluginError);
  });
});
