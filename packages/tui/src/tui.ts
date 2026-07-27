import { resolve } from 'node:path';
import * as readline from 'node:readline';
import type { VynthConfig } from '@vynth/core';
import { builtinTools, createProvider, runAgent } from '@vynth/engine';
import { type TrustConfirm, loadPluginsWithTrust } from '@vynth/plugins';
import {
  renderBadge,
  renderDivider,
  renderInline,
  renderInputPanel,
  renderMessage,
  renderStatusBar,
  renderToolBlock,
  visibleWidth,
  wrapLine
} from './render';
import { Scrollback } from './scrollback';
import { fg, palette, reset } from './theme';

const ESC = '\x1b';
const CLEAR_SCREEN = `${ESC}[2J`;
const CURSOR_HOME = `${ESC}[H`;
const HIDE_CURSOR = `${ESC}[?25l`;
const SHOW_CURSOR = `${ESC}[?25h`;
const enterAltScreen = `${ESC}[?1049h`;
const leaveAltScreen = `${ESC}[?1049l`;

/**
 * 3 段式 IDE 风 TUI（对标 opencode / codex / Claude Code）：
 *   ┌─ 顶栏（mode / theme / model / 状态 / 回合）──────────────┐
 *   │  [VIBE] [mocha] [model] [streaming]           turn 3    │
 *   ├─ 中间可滚动聊天区（鼠标滚轮 / Shift+↑↓ / PgUp/PgDn）──┤
 *   │  ▎ 欢迎语                                              │
 *   │  ▎ 上一轮目标                                          │
 *   │  ▎ 上一轮回答                                          │
 *   │  ✓ read_file {...}                                     │
 *   │  ▎ (流式 token)                                        │
 *   ├─ 底栏（矩形输入框）──────────────────────────────────────┤
 *   │  ╭──────────────────────────────────────────────────╮   │
 *   │  │ ❯ {input}▏                                      │   │
 *   │  ╰──────────────────────────────────────────────────╯   │
 *   │  ⏎ submit · ⇧↑/⇧↓ scroll history · esc 取消           │
 *   └──────────────────────────────────────────────────────────┘
 *
 * 渲染策略：每次 draw() 输出一个完整帧（清屏 + 逐行写满），不做增量。
 * 这避免了 DECSTBM scroll region 与 eraseDown 的交互造成的乱码/残影。
 * 每行写完后以 \x1b[0m 收尾，杜绝 SGR 泄漏。
 */
export async function startTui(config: VynthConfig, pluginPaths: string[] = []): Promise<void> {
  const c = palette(config.theme);
  const provider = createProvider(config);
  const tools = builtinTools(config.sandbox.cwd, {
    networkAllowed: config.sandbox.networkAllowed
  });

  const scrollback = new Scrollback(2_000);
  const transcript: Array<{
    role: 'user' | 'assistant' | 'system' | 'tool';
    content: string;
    toolId?: string;
    collapsed?: boolean;
  }> = [];
  let input = '';
  let liveStatus: 'idle' | 'streaming' | 'tool' = 'idle';
  let prompting = false;
  let lastToolName: string | null = null;
  let userScrollOffset = 0;
  const connectionStatus: 'connected' | 'disconnected' | 'error' = 'connected';
  let selectedToolId: string | null = null;
  let toolIdCounter = 0;

  const rl = readline.createInterface({ input: process.stdin, output: process.stdout });
  readline.emitKeypressEvents(process.stdin, rl);
  const stdinTty =
    process.stdout.isTTY === true && (process.stdin as { isTTY?: boolean }).isTTY === true;
  if (stdinTty) process.stdin.setRawMode(true);

  if (stdinTty) {
    process.stdout.write(enterAltScreen);
    process.stdout.write(HIDE_CURSOR);
  }

  // ─── 信任确认 ───
  const confirmTty: TrustConfirm = async ({ path }) => {
    return askYesNo(
      `${fg(c.red)}⚠ 信任并加载插件？${reset}\n  插件将在本进程执行任意代码，拥有与 Vynth 同等的文件/网络/命令权限。\n  路径: ${path}\n  仅加载你完全信任的插件。确认加载`
    );
  };

  // ─── 启动时欢迎语 ───
  const welcome = `Vynth ${config.mode === 'plan' ? '计划模式' : '随手模式'} · 模型 ${config.model} · 沙箱 cwd=${config.sandbox.cwd}`;
  appendSystemMessage(welcome);

  if (pluginPaths.length > 0) {
    const abs = pluginPaths.map((p) => resolve(config.sandbox.cwd, p));
    const res = await loadPluginsWithTrust(abs, tools, confirmTty);
    for (const n of res.loaded) appendSystemMessage(`✓ plugin loaded: ${n}`);
    for (const p of res.declined) appendSystemMessage(`✗ plugin declined: ${p}`);
    for (const e of res.errors) appendSystemMessage(`✗ plugin error: ${e.error}`);
  }

  draw();

  // ─── 终端大小变化 ───
  process.stdout.on('resize', () => draw());

  function askYesNo(question: string): Promise<boolean> {
    return new Promise((resolvePrompt) => {
      prompting = true;
      const stdinTty2 =
        process.stdout.isTTY === true && (process.stdin as { isTTY?: boolean }).isTTY === true;
      const prevRaw =
        stdinTty2 && 'isRaw' in process.stdin
          ? (process.stdin as { isRaw?: boolean }).isRaw === true
          : false;
      if (stdinTty2) process.stdin.setRawMode(false);
      rl.question(`${question} [y/N] `, (ans) => {
        if (stdinTty2) process.stdin.setRawMode(prevRaw);
        prompting = false;
        draw();
        const a = ans.trim().toLowerCase();
        resolvePrompt(a === 'y' || a === 'yes');
      });
    });
  }

  // ─── 核心渲染：整帧重绘 ───
  function draw(): void {
    const cols = Math.max(60, Math.min(140, (process.stdout.columns ?? 100) - 1));
    const rows = Math.max(20, process.stdout.rows ?? 32);
    const innerW = cols - 4;
    const TOP_H = 2;
    const BOT_H = selectedToolId ? 10 : 8;
    const midH = Math.max(5, rows - TOP_H - BOT_H);

    // 1. 清屏 + 光标归位
    process.stdout.write(`${CLEAR_SCREEN}${CURSOR_HOME}`);

    // 2. 顶栏（2 行：品牌 + 状态）
    const topLines = renderTopBar(cols);
    for (const line of topLines) {
      process.stdout.write(`${line}${reset}\n`);
    }

    // 3. 分隔线
    process.stdout.write(
      `${renderDivider({ width: cols, color: c.overlay0 ?? c.subtext })}${reset}\n`
    );

    // 4. 中间聊天区（midH 行）
    const allLines = renderTranscriptToLines(innerW);
    const totalLines = allLines.length;
    const maxScroll = Math.max(0, totalLines - midH);
    const effectiveScroll = Math.min(userScrollOffset, maxScroll);

    let visibleLines: string[];
    if (effectiveScroll === 0) {
      visibleLines = allLines.slice(-midH);
    } else {
      visibleLines = allLines.slice(effectiveScroll, effectiveScroll + midH);
    }

    for (let i = 0; i < midH; i++) {
      const line = visibleLines[i] ?? '';
      const truncated = truncateVisible(line, cols);
      process.stdout.write(`${truncated}${reset}\n`);
    }

    // 6. 底栏分隔线
    process.stdout.write(
      `${renderDivider({ width: cols, color: c.overlay0 ?? c.subtext })}${reset}\n`
    );

    // 6b. Tool 选中状态指示
    if (selectedToolId) {
      const toolEntry = transcript.find((t) => t.toolId === selectedToolId);
      if (toolEntry) {
        const toolName = toolEntry.content.split('(')[0] || 'tool';
        const isCollapsed = collapsedTools.has(selectedToolId);
        const toolStatus = `${fg(c.yellow)}▸ ${toolName} ${isCollapsed ? 'collapsed' : 'expanded'}${reset}`;
        process.stdout.write(`${toolStatus}${reset}\n`);
        process.stdout.write(
          `${renderDivider({ width: cols, color: c.overlay0 ?? c.subtext })}${reset}\n`
        );
      }
    }

    // 7. 底栏（面板式输入框 + 键位提示）
    const statusText =
      liveStatus === 'streaming'
        ? 'streaming'
        : liveStatus === 'tool'
          ? `tool: ${lastToolName ?? '…'}`
          : 'ready';
    const statusColor =
      liveStatus === 'streaming' ? c.yellow : liveStatus === 'tool' ? c.lavender : c.green;

    const inputPanel = renderInputPanel({
      width: cols,
      input,
      model: config.model,
      mode: config.mode,
      theme: config.theme,
      status: statusText,
      statusColor,
      palette: c
    });
    for (const line of inputPanel.split('\n')) {
      const truncated = truncateVisible(line, cols);
      process.stdout.write(`${truncated}${reset}\n`);
    }

    // 7. 光标定位到输入行（输入框第 2 行 = rows - BOT_H + 3）
    const cursorRow = rows - BOT_H + 3;
    const cursorCol = visibleWidth(`  ❯ ${input}`) + 3;
    process.stdout.write(`${ESC}[${cursorRow};${cursorCol}H`);
  }

  function renderTopBar(cols: number): string[] {
    // Line 1: 品牌 + 模式徽章 + 模型 + 主题 + 连接状态
    const brand = `${fg(c.mauve)}VYNT${reset}`;
    const modeBadge = renderBadge(
      ` ${config.mode === 'plan' ? 'PLAN' : 'VIBE'} `,
      c.crust ?? c.base,
      c.mauve
    );
    const modelBadge = renderBadge(` ${config.model} `, c.crust ?? c.base, c.teal);
    const themeBadge = renderBadge(` ${config.theme} `, c.crust ?? c.base, c.blue);

    const connColor =
      connectionStatus === 'connected' ? c.green : connectionStatus === 'error' ? c.yellow : c.red;
    const connIcon =
      connectionStatus === 'connected' ? '●' : connectionStatus === 'error' ? '◐' : '○';
    const connBadge = renderBadge(
      ` ${connIcon} ${connectionStatus} `,
      c.crust ?? c.base,
      connColor
    );

    const left = [brand, modeBadge, modelBadge, themeBadge, connBadge].join('  ');

    // Line 2: 状态 + 回合数
    const statusBadge =
      liveStatus === 'streaming'
        ? renderBadge(' streaming ', c.crust ?? c.base, c.yellow)
        : liveStatus === 'tool'
          ? renderBadge(` tool: ${lastToolName ?? '…'} `, c.crust ?? c.base, c.lavender)
          : renderBadge(' ready ', c.crust ?? c.base, c.green);
    const right = renderBadge(
      ` turn ${transcript.filter((t) => t.role === 'user').length} `,
      c.crust ?? c.base,
      c.mantle
    );
    const statusLine = renderStatusBar({
      width: cols,
      left: statusBadge,
      right,
      color: c.subtext,
      bgHex: c.mantle,
      textColor: c.subtext
    });

    return [
      renderStatusBar({
        width: cols,
        left,
        right: '',
        color: c.subtext,
        bgHex: c.mantle,
        textColor: c.text
      }),
      statusLine
    ];
  }

  function appendToMid(multiLine: string): void {
    scrollback.push(multiLine);
  }

  function renderTranscriptToLines(width: number): string[] {
    const lines: string[] = [];
    for (const entry of transcript) {
      if (entry.role === 'tool' && entry.toolId) {
        const isCollapsed = collapsedTools.has(entry.toolId);
        const isSelected = selectedToolId === entry.toolId;
        const block = renderToolBlock({
          name: entry.content.split('(')[0] || 'tool',
          args: entry.content.includes('(')
            ? entry.content.slice(entry.content.indexOf('('))
            : '{}',
          status: 'ok',
          palette: c,
          width,
          collapsed: isCollapsed,
          selected: isSelected
        });
        for (const line of block.split('\n')) {
          lines.push(line);
        }
      } else {
        const block = renderMessage({
          role: entry.role,
          content: entry.content,
          palette: c,
          width,
          background: entry.role !== 'system'
        });
        for (const line of block.split('\n')) {
          lines.push(line);
        }
      }
      lines.push(''); // empty line between messages
    }
    return lines;
  }

  function appendMessage(
    role: 'user' | 'assistant' | 'system' | 'tool',
    content: string,
    options?: { toolId?: string; background?: boolean }
  ): void {
    transcript.push({
      role,
      content,
      toolId: options?.toolId,
      collapsed: role === 'tool' ? collapsedTools.has(options?.toolId ?? '') : undefined
    });
    const cols = Math.max(60, Math.min(140, (process.stdout.columns ?? 100) - 1));
    const innerW = cols - 4;
    const block = renderMessage({
      role,
      content,
      palette: c,
      width: innerW,
      background: options?.background ?? role !== 'system'
    });
    appendToMid(`${block}\n`);
    userScrollOffset = 0;
  }

  function appendSystemMessage(content: string): void {
    appendMessage('system', content, { background: false });
  }

  // ─── 输入事件 ───
  process.stdin.on('keypress', (str: string | null, key: readline.Key) => {
    if (prompting || !key) return;

    if (key.ctrl && key.name === 'c') {
      cleanup();
      process.exit(0);
    }

    if (key.shift && key.name === 'up') {
      const allLines = renderTranscriptToLines(innerW);
      const maxScroll = Math.max(0, allLines.length - midH);
      userScrollOffset = Math.min(userScrollOffset + 3, maxScroll);
      draw();
      return;
    }
    if (key.shift && key.name === 'down') {
      userScrollOffset = Math.max(userScrollOffset - 3, 0);
      draw();
      return;
    }
    if (key.name === 'pageup') {
      const allLines = renderTranscriptToLines(innerW);
      const maxScroll = Math.max(0, allLines.length - midH);
      userScrollOffset = Math.min(userScrollOffset + midH, maxScroll);
      draw();
      return;
    }
    if (key.name === 'pagedown') {
      userScrollOffset = Math.max(userScrollOffset - midH, 0);
      draw();
      return;
    }

    if (key.name === 'tab') {
      const currentIdx = selectedToolId
        ? transcript.findIndex((t) => t.toolId === selectedToolId)
        : -1;
      const nextIdx = currentIdx === -1 ? 0 : currentIdx + 1;
      const nextTool = findNextToolId(nextIdx, 1);
      selectedToolId = nextTool;
      draw();
      return;
    }
    if (key.name === 'escape') {
      selectedToolId = null;
      draw();
      return;
    }
    if (key.name === 'return') {
      if (selectedToolId) {
        toggleToolCollapse(selectedToolId);
        draw();
        return;
      }
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

  // 滚轮（SGR 1006）
  process.stdin.on('data', (chunk) => {
    const text = chunk.toString('utf8');
    let rest = text;
    const cols = Math.max(60, Math.min(140, (process.stdout.columns ?? 100) - 1));
    const rows = Math.max(20, process.stdout.rows ?? 32);
    const innerW = cols - 4;
    const TOP_H = 2;
    const BOT_H = 8;
    const midH = Math.max(5, rows - TOP_H - BOT_H);
    while (true) {
      const parsed = parseSGRMouse(rest);
      if (!parsed) break;
      rest = parsed.rest;
      const ev = parsed.event;
      if (ev.button === 64) {
        const allLines = renderTranscriptToLines(innerW);
        const maxScroll = Math.max(0, allLines.length - midH);
        userScrollOffset = Math.min(userScrollOffset + 3, maxScroll);
        draw();
      } else if (ev.button === 65) {
        userScrollOffset = Math.max(userScrollOffset - 3, 0);
        draw();
      }
    }
  });

  async function submit(): Promise<void> {
    const goal = input.trim();
    if (!goal) return;
    input = '';
    liveStatus = 'idle';
    appendMessage('user', goal);
    draw();
    let acc = '';
    for await (const ev of runAgent(goal, { provider, tools })) {
      if (ev.type === 'reasoning') {
        void acc;
      } else if (ev.type === 'token') {
        acc += ev.text;
        liveStatus = 'streaming';
        draw();
      } else if (ev.type === 'tool') {
        const cols = Math.max(60, Math.min(140, (process.stdout.columns ?? 100) - 1));
        const innerW = cols - 4;
        const toolId = `tool-${++toolIdCounter}`;
        const isCollapsed = collapsedTools.has(toolId);
        const block = renderToolBlock({
          name: ev.call.name,
          args: JSON.stringify(ev.call.args),
          status: 'ok',
          output: '',
          palette: c,
          width: innerW,
          collapsed: isCollapsed
        });
        appendToMid(`${block}\n`);
        transcript.push({
          role: 'tool',
          content: `${ev.call.name}(${JSON.stringify(ev.call.args)})`,
          toolId
        });
        lastToolName = ev.call.name;
        liveStatus = 'tool';
        draw();
      } else if (ev.type === 'done') {
        if (acc) {
          appendMessage('assistant', acc);
          acc = '';
        }
        liveStatus = 'idle';
        draw();
      }
    }
    if (acc) appendMessage('assistant', acc);
    liveStatus = 'idle';
    draw();
  }

  function cleanup(): void {
    if (stdinTty) {
      process.stdout.write(SHOW_CURSOR);
      process.stdout.write(leaveAltScreen);
    }
    if (process.stdin.isTTY) process.stdin.setRawMode(false);
    rl.close();
  }

  process.on('exit', cleanup);
  process.on('SIGINT', () => {
    cleanup();
    process.exit(0);
  });
}

// ─── 工具函数 ───

/** 按可见宽度截断一行（保留 ANSI 转义），不切到转义中段 */
function truncateVisible(input: string, maxCols: number): string {
  if (maxCols <= 0) return '';
  let vis = 0;
  let out = '';
  let i = 0;
  while (i < input.length && vis < maxCols) {
    if (input[i] === ESC) {
      const end = input.indexOf('m', i);
      if (end === -1) {
        i++;
        continue;
      }
      out += input.slice(i, end + 1);
      i = end + 1;
      continue;
    }
    const cp = input.codePointAt(i) ?? 0;
    const w = isWideChar(cp) ? 2 : 1;
    if (vis + w > maxCols) break;
    out += input[i];
    vis += w;
    i++;
  }
  return out;
}

function isWideChar(cp: number): boolean {
  return (
    (cp >= 0x1100 && cp <= 0x115f) ||
    (cp >= 0x2e80 && cp <= 0x303e) ||
    (cp >= 0x3041 && cp <= 0x33ff) ||
    (cp >= 0x3400 && cp <= 0x4dbf) ||
    (cp >= 0x4e00 && cp <= 0x9fff) ||
    (cp >= 0xac00 && cp <= 0xd7a3) ||
    (cp >= 0xf900 && cp <= 0xfaff) ||
    (cp >= 0xff00 && cp <= 0xff60) ||
    (cp >= 0xffe0 && cp <= 0xffe6)
  );
}

/** 解析 SGR 鼠标事件（1006 协议） */
function parseSGRMouse(
  buffer: string
): { event: { button: number; row: number; col: number }; rest: string } | null {
  const re = new RegExp(`${ESC}\\[<(\\d+);(\\d+);(\\d+)([mM])`);
  const m = re.exec(buffer);
  if (!m) return null;
  const button = Number(m[1]);
  const col = Number(m[2]);
  const row = Number(m[3]);
  const rest = buffer.slice(m.index + m[0].length);
  return { event: { button, row, col }, rest };
}
