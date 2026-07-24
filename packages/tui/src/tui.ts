import * as readline from 'node:readline';
import type { VynthConfig } from '@vynth/core';
import { builtinTools, createProvider, runAgent } from '@vynth/engine';
import { StreamArea } from './stream-escape-hatch';
import { fg, palette, reset } from './theme';

/**
 * 轻量 ANSI TUI（无外部布局引擎），保证 `bun build --compile` 产出的单二进制
 * 不依赖 yoga.wasm 等外部资源文件。高频 token 更新走 StreamArea 直写，规避重渲染。
 */
export function startTui(config: VynthConfig): void {
  const c = palette(config.theme);
  const provider = createProvider(config);
  const tools = builtinTools(config.sandbox.cwd);
  const history: string[] = [];
  let input = '';
  let live = '';

  const rl = readline.createInterface({ input: process.stdin, output: process.stdout });
  readline.emitKeypressEvents(process.stdin, rl);
  if (process.stdin.isTTY) process.stdin.setRawMode(true);

  const area = new StreamArea((s) => process.stdout.write(s));
  draw();

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
