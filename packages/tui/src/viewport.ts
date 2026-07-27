/**
 * 终端滚动区（viewport）：用 DECSTBM 切分顶栏 / 中间 / 底栏三个区域；
 * 通过 SU/SD/RESET/CUP/EL 等转义做增量重绘，规避每次全屏清屏带来的闪烁。
 *
 * 设计目标（对标 opencode / codex / Claude Code TUI）：
 *  - 顶栏固定：mode / theme / model / 实时状态 / 回合计数
 *  - 中间可滚动：完整历史（scrollback），用户可用滚轮 / Shift+↑↓ 浏览
 *  - 底栏固定：状态指示 + 输入框 + 提示符
 *
 * 兼容性：
 *  - DECSTBM `\x1b[{top};{bottom}r` 在 kitty / iTerm2 / WezTerm / Windows Terminal / VS Code 终端均支持
 *  - 鼠标滚轮 SGR 1006 + 任意按钮追踪 1003 + 焦点事件 1004
 *  - 终端不支持时（CI / 无 TTY）会自动降级为「整屏重绘 + 接受不可滚动」
 */
export interface ViewportSize {
  cols: number;
  rows: number;
}

export interface ViewportLayout {
  /** 第 1 行开始（包含） */
  topStart: number;
  /** 第 1 行结束（包含） */
  topEnd: number;
  /** 中间区起始（包含） */
  midStart: number;
  /** 中间区结束（包含） */
  midEnd: number;
  /** 底栏起始（包含） */
  botStart: number;
  /** 底栏结束（包含） */
  botEnd: number;
}

const ESC = '\x1b';

/** 进入/退出 alternate screen buffer */
export const enterAltScreen = `${ESC}[?1049h`;
export const leaveAltScreen = `${ESC}[?1049l`;

/** 把屏幕分成 [topStart..topEnd] / [midStart..midEnd] / [botStart..botEnd] 三段。 */
export function setScrollRegion(
  topEnd: number,
  botStart: number,
  rows: number
): { top: string; mid: string; bot: string } {
  const top = `${ESC}[1;${topEnd}r`;
  const mid = `${ESC}[${topEnd + 1};${botStart - 1}r`;
  const bot = `${ESC}[${botStart};${rows}r`;
  return { top, mid, bot };
}

/** 重置滚动区为整屏 */
export const resetScrollRegion = `${ESC}[r`;

export const cursorHome = `${ESC}[H`;
export const cursorTo = (row: number, col: number): string => `${ESC}[${row};${col}H`;
export const eraseLine = `${ESC}[2K`;
export const eraseDown = `${ESC}[J`;
export const eraseUp = `${ESC}[1J`;

/** 启用 SGR 鼠标协议 + 任意按钮追踪 + 滚轮追踪 */
export const enableMouseTracking = [
  `${ESC}[?1006h`,
  `${ESC}[?1003h`,
  `${ESC}[?1005h`,
  `${ESC}[?1004h`
].join('');
export const disableMouseTracking = [
  `${ESC}[?1006l`,
  `${ESC}[?1003l`,
  `${ESC}[?1005l`,
  `${ESC}[?1004l`
].join('');

/** 解析 SGR 鼠标事件。返回 { kind: 'press'|'release'|'drag', row, col, button } 或 null */
export interface MouseEvent {
  kind: 'press' | 'release' | 'drag' | 'wheel';
  row: number;
  col: number;
  /** 0=left 1=middle 2=right 64=wheel-up 65=wheel-down */
  button: number;
}

const SGR_MOUSE_RE = new RegExp(`${ESC}\\[<(\\d+);(\\d+);(\\d+)([mM])`, 'g');

export function parseMouse(buffer: string): { event: MouseEvent; rest: string } | null {
  SGR_MOUSE_RE.lastIndex = 0;
  const m = SGR_MOUSE_RE.exec(buffer);
  if (!m) return null;
  const button = Number(m[1]);
  const col = Number(m[2]);
  const row = Number(m[3]);
  const end = m[4];
  let kind: MouseEvent['kind'] = 'press';
  // SGR 滚轮：button=64 up / 65 down，且以 'M' 结尾（press 语义）
  if (button === 64 || button === 65) {
    kind = 'wheel';
  } else if (end === 'm') {
    kind = 'release';
  } else if ((button & 32) !== 0) {
    kind = 'drag';
  }
  const rest = buffer.slice(SGR_MOUSE_RE.lastIndex);
  SGR_MOUSE_RE.lastIndex = 0;
  return { event: { kind, row, col, button }, rest };
}

/** 计算三段布局（顶/中间/底）。中间行数会确保 ≥ 5。 */
export function computeLayout(rows: number, topLines: number, botLines: number): ViewportLayout {
  const safeRows = Math.max(10, rows);
  const topEnd = Math.min(safeRows - 6, Math.max(1, topLines));
  const botStart = Math.max(topEnd + 6, safeRows - Math.max(3, botLines) + 1);
  const midStart = topEnd + 1;
  const midEnd = botStart - 1;
  return {
    topStart: 1,
    topEnd,
    midStart,
    midEnd,
    botStart,
    botEnd: safeRows
  };
}

/**
 * 滚动区增量重绘：当历史总行数超过视口行数时，把光标移到视口底端，
 * 滚动区域设到中间段，使用 SU 把内容向上滚，再用 EL 清除底部一行，写入新内容。
 * 这是 Claude Code / opencode 风格的核心：仅重绘差异行，不闪烁。
 */
export function appendToScrollback(opts: {
  midStart: number;
  midEnd: number;
  currentLines: number;
  /** 已渲染的整屏缓冲（不含 top/bot 区） */
  buffer: string[];
}): string {
  const { midStart, midEnd, currentLines, buffer } = opts;
  const height = midEnd - midStart + 1;
  const out: string[] = [];
  // 1. 把光标放到滚动区底端
  out.push(cursorTo(midEnd, 1));
  // 2. 如果超出视口，先 SU 上滚
  const overflow = Math.max(0, currentLines - height);
  if (overflow > 0) {
    out.push(`${ESC}[${overflow}S`);
  }
  // 3. 写新内容（buffer 末尾若干行）
  const tail = buffer.slice(-height);
  for (const line of tail) {
    out.push(line);
    out.push('\n');
  }
  return out.join('');
}

/** 在中间视口里把内容完全重绘（用于初次绘制或视口大小变化）。 */
export function repaintScrollback(opts: {
  midStart: number;
  midEnd: number;
  buffer: string[];
}): string {
  const out: string[] = [];
  out.push(cursorTo(opts.midStart, 1));
  out.push(eraseDown);
  for (const line of opts.buffer) {
    out.push(line);
    out.push('\n');
  }
  return out.join('');
}

/** 在底栏绘制整段（EL 清理后再写）。 */
export function paintBottom(row: number, lines: string[]): string {
  const out: string[] = [];
  for (let i = 0; i < lines.length; i++) {
    out.push(cursorTo(row + i, 1));
    out.push(eraseLine);
    out.push(lines[i] ?? '');
  }
  return out.join('');
}

/** 在顶栏绘制整段（EL 清理后再写）。 */
export function paintTop(row: number, lines: string[]): string {
  return paintBottom(row, lines);
}

/** 简易 ANSI 行截断：限制长度，不切到转义中段。 */
export function truncateAnsiLine(input: string, maxCols: number): string {
  if (maxCols <= 0) return '';
  const re = new RegExp(`${ESC}\\[[0-9;?]*[a-zA-Z]`, 'g');
  const plain = input.replace(re, '');
  if (plain.length <= maxCols) return input;
  // 简化：保留可见宽度截断
  let out = '';
  let vis = 0;
  let i = 0;
  while (i < input.length && vis < maxCols) {
    if (input[i] === ESC) {
      const end = input.indexOf('m', i);
      if (end === -1) {
        out += input[i];
        i++;
        continue;
      }
      out += input.slice(i, end + 1);
      i = end + 1;
      continue;
    }
    out += input[i];
    vis++;
    i++;
  }
  return `${out}${ESC}[0m`;
}
