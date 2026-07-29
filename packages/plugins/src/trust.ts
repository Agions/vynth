import { toErrorMessage } from '@zeno/core';
import type { ToolRegistry } from '@zeno/engine';
import { loadPlugin } from './loader';

export type TrustConfirm = (info: { path: string }) => Promise<boolean>;

export interface PluginResolution {
  loaded: string[];
  declined: string[];
  errors: { path: string; error: string }[];
}

export async function loadPluginsWithTrust(
  paths: string[],
  reg: ToolRegistry,
  confirm: TrustConfirm
): Promise<PluginResolution> {
  const loaded: string[] = [];
  const declined: string[] = [];
  const errors: { path: string; error: string }[] = [];
  for (const p of paths) {
    const trusted = await confirm({ path: p });
    if (!trusted) {
      declined.push(p);
      continue;
    }
    try {
      const plugin = await loadPlugin(p);
      plugin.activate(reg);
      loaded.push(plugin.name);
    } catch (err) {
      errors.push({ path: p, error: toErrorMessage(err) });
    }
  }
  return { loaded, declined, errors };
}
