import { resolve } from 'node:path';
import * as readline from 'node:readline';
import type { VynthConfig } from '@vynth/core';
import { builtinTools, createProvider, runAgent } from '@vynth/engine';
import { type TrustConfirm, loadPluginsWithTrust } from '@vynth/plugins';
import { StreamArea } from './stream-escape-hatch';
import { fg, palette, reset } from './theme';

/**
 * 轻量 ANSI TUI（无外部布局引擎），保证 `bun build --compile` 产出的单二进制
 * 不依赖 yoga.wasm 等外部资源文件。高频 token 更新走 StreamArea 直写，规避重渲染。
 *
 * pluginPaths：待加载的本地插件入口（相对 cwd 的路径或绝对路径）。每个插件在
 * import 之前会弹出信任确认（信任模型联动）——插件在进程内执行任意代码，拥有与
 * Vynth 同等的权限，故需用户显式确认后才加载。
 */
export async function startTui(config: VynthConfig, pluginPaths: string[] = []): Promise<void> {
  const c = palette(config.theme);
  const provider = createProvider(config);
  const tools = builtinTools(config.sandbox.cwd, { networkAllowed: config.sandbox.networkAllowed });
  const history: string[] = [];
  let input = '';
  let live = '';
  let prompting = false;

  const rl = readline.createInterface({ input: process.stdin, output: process.stdout });
  readline.emitKeypressEvents(process.stdin, rl);
  if (process.stdin.isTTY) process.stdin.setRawMode(true);

  const area = new StreamArea((s) => process.stdout.write(s));
  draw();

  // TTY 信任确认：插件在 import 前需用户显式授权（避免执行未受信的任意代码）。
  const confirmTty: TrustConfirm = async ({ path }) => {
    return askYesNo(
      `${fg(c.red)}⚠ 信任并加载插件？${reset}\n  该插件将在本进程执行任意代码，拥有与 Vynth 同等的文件系统/网络/命令权限。\n  路径: ${path}\n  仅加载你完全信任的插件。确认加载`
    );
  };

  if (pluginPaths.length > 0) {
    const abs = pluginPaths.map((p) => resolve(config.sandbox.cwd, p));
    const res = await loadPluginsWithTrust(abs, tools, confirmTty);
    for (const n of res.loaded) history.push(`${fg(c.green)}› 已信任并加载插件: ${n}${reset}`);
    for (const p of res.declined)
      history.push(`${fg(c.yellow)}⚠ 已拒绝插件 (未加载): ${p}${reset}`);
    for (const e of res.errors) history.push(`${fg(c.red)}✗ 插件加载失败: ${e.error}${reset}`);
    draw();
  }

  function askYesNo(question: string): Promise<boolean> {
    return new Promise((resolvePrompt) => {
      prompting = true;
      const prevRaw = process.stdin.isTTY ? process.stdin.isRawMode : false;
      if (process.stdin.isTTY) process.stdin.setRawMode(false);
      rl.question(`${question} [y/N] `, (ans) => {
        if (process.stdin.isTTY) process.stdin.setRawMode(prevRaw ?? false);
        prompting = false;
        draw();
        const a = ans.trim().toLowerCase();
        resolvePrompt(a === 'y' || a === 'yes');
      });
    });
  }

  function draw(): void {
    const lines: string[] = [];
    lines.push(`${fg(c.mauve)}Vynth${reset} · ${config.mode} mode`);
    for (const h of history) lines.push(h);
    if (live) lines.push(`${fg(c.teal)}${live}${reset}`);
    lines.push(`${fg(c.mauve)}>${reset}${input}`);
    process.stdout.write(`\x1b[2J\x1b[H${lines.join('\n')}`);
    area.clear();
  }

  process.stdin.on('keypress', (str: string | null, key: readline.Key) => {
    if (prompting) return; // 信任确认期间忽略常规按键
    if (key.ctrl && (key.name === 'c' || key.name === 'd')) {
      cleanup();
      process.exit(0);
    }
    if (key.name === 'return') {
      void submit();
      return;
    }
    if (key.name === 'backspace') {
      input = input.slice(0, -1);
      draw();
      return;
    }
    if (str && !key.ctrl && !key.meta) {
      input += str;
      draw();
    }
  });

  async function submit(): Promise<void> {
    const goal = input.trim();
    if (!goal) return;
    input = '';
    live = '';
    history.push(`> ${goal}`);
    draw();
    let acc = '';
    for await (const ev of runAgent(goal, { provider, tools })) {
      if (ev.type === 'token') {
        acc += ev.text;
        live = acc;
        draw();
      } else if (ev.type === 'tool') {
        history.push(`  [tool] ${ev.call.name}(${JSON.stringify(ev.call.args)})`);
        live = '';
        draw();
      }
    }
    history.push(acc);
    live = '';
    draw();
  }

  function cleanup(): void {
    if (process.stdin.isTTY) process.stdin.setRawMode(false);
    rl.close();
  }
}
