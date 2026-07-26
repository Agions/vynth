import { PluginError, audit } from '@vynth/core';
import type { ToolRegistry } from '@vynth/engine';

export interface Plugin {
  name: string;
  activate: (reg: ToolRegistry) => void;
}

interface PluginModule {
  pluginName: string;
  activate: (reg: ToolRegistry) => void;
}

export async function loadPlugin(entryPath: string): Promise<Plugin> {
  let mod: PluginModule;
  try {
    mod = (await import(entryPath)) as PluginModule;
  } catch (err) {
    audit().record('plugin_load', { path: entryPath, ok: false }, false);
    throw new PluginError(`failed to load plugin ${entryPath}: ${errMsg(err)}`);
  }
  if (!mod.pluginName || typeof mod.activate !== 'function') {
    audit().record('plugin_load', { path: entryPath, ok: false }, false);
    throw new PluginError(`plugin ${entryPath} must export pluginName and activate()`);
  }
  audit().record('plugin_load', { path: entryPath, name: mod.pluginName, ok: true }, true);
  return { name: mod.pluginName, activate: mod.activate };
}

export async function loadAll(entries: string[], reg: ToolRegistry): Promise<string[]> {
  const loaded: string[] = [];
  for (const entry of entries) {
    const plugin = await loadPlugin(entry);
    plugin.activate(reg);
    loaded.push(plugin.name);
  }
  return loaded;
}

function errMsg(err: unknown): string {
  return err instanceof Error ? err.message : String(err);
}
