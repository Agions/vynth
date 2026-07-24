import { PluginError } from '@vynth/core';
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
    throw new PluginError(`failed to load plugin ${entryPath}: ${errMsg(err)}`);
  }
  if (!mod.pluginName || typeof mod.activate !== 'function') {
    throw new PluginError(`plugin ${entryPath} must export pluginName and activate()`);
  }
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
