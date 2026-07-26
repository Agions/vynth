import { PluginError } from '@vynth/core';
import type { ToolRegistry } from '@vynth/engine';
import { loadPlugin } from './loader';

/** 信任确认回调：返回 true 才真正 import + activate 插件 */
export type TrustConfirm = (info: { path: string }) => Promise<boolean>;

export interface PluginResolution {
  /** 已信任并成功激活的插件名 */
  loaded: string[];
  /** 用户拒绝（未加载）的插件路径 */
  declined: string[];
  /** 确认后加载失败（import/activate 抛错）的插件 */
  errors: { path: string; error: string }[];
}

/**
 * 信任门禁式的批量插件加载。
 *
 * 安全要点：插件 `import()` 即在本进程执行任意代码，因此确认回调 **必须在
 * import 之前** 返回 true 才继续。无头脚本场景可传入 `async () => true`
 * —— `-p` 显式授权即视为已信任；交互 TUI 则传入交互式确认回调（信任模型联动）。
 *
 * @param paths 插件入口路径（应已 resolve 为绝对路径）
 * @param reg   目标工具注册表（插件在此注册工具）
 * @param confirm 逐插件的信任确认回调
 */
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
      errors.push({ path: p, error: errMsg(err) });
    }
  }
  return { loaded, declined, errors };
}

function errMsg(err: unknown): string {
  if (err instanceof PluginError) return err.message;
  return err instanceof Error ? err.message : String(err);
}
