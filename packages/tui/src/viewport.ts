export interface ViewportSize {
  cols: number;
  rows: number;
}

export interface ViewportLayout {
  topStart: number;
  topEnd: number;
  midStart: number;
  midEnd: number;
  botStart: number;
  botEnd: number;
}

const ESC = '\x1b';

export const enterAltScreen = `${ESC}[?1049h`;
export const leaveAltScreen = `${ESC}[?1049l`;

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

export const resetScrollRegion = `${ESC}[r`;

export const cursorHome = `${ESC}[H`;
export const cursorTo = (row: number, col: number): string => `${ESC}[${row};${col}H`;
export const eraseLine = `${ESC}[2K`;
export const eraseDown = `${ESC}[J`;
export const eraseUp = `${ESC}[1J`;

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

export function appendToScrollback(opts: {
  midStart: number;
  midEnd: number;
  currentLines: number;
  buffer: string[];
}): string {
  const { midStart, midEnd, currentLines, buffer } = opts;
  const height = midEnd - midStart + 1;
  const out: string[] = [];
  out.push(cursorTo(midEnd, 1));
  const overflow = Math.max(0, currentLines - height);
  if (overflow > 0) {
    out.push(`${ESC}[${overflow}S`);
  }
  const tail = buffer.slice(-height);
  for (const line of tail) {
    out.push(line);
    out.push('\n');
  }
  return out.join('');
}

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

export function paintBottom(row: number, lines: string[]): string {
  const out: string[] = [];
  for (let i = 0; i < lines.length; i++) {
    out.push(cursorTo(row + i, 1));
    out.push(eraseLine);
    out.push(lines[i] ?? '');
  }
  return out.join('');
}

export function paintTop(row: number, lines: string[]): string {
  return paintBottom(row, lines);
}

export function truncateAnsiLine(input: string, maxCols: number): string {
  if (maxCols <= 0) return '';
  const re = new RegExp(`${ESC}\\[[0-9;?]*[a-zA-Z]`, 'g');
  const plain = input.replace(re, '');
  if (plain.length <= maxCols) return input;
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
