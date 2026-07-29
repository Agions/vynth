
import type { CellModifiers, ScreenBuffer } from '../kernel/buffer';
import { clearBuffer, setCell } from '../kernel/buffer';
import { charWidth, visibleWidth, wrapLine } from '../utils/unicode';

export function renderTextToBuffer(
  buf: ScreenBuffer,
  x: number,
  y: number,
  text: string,
  fg = 0,
  bg = 0,
  mods: CellModifiers = {}
): void {
  let posX = x;
  for (let i = 0; i < text.length; i++) {
    const ch = text[i];
    if (ch === '\x1b') {
      const match = /^\x1b\[[0-9;?]*[a-zA-Z]/.exec(text.slice(i));
      if (match) {
        i += match[0].length - 1;
        continue;
      }
    }
    setCell(buf, posX, y, ch, fg, bg, mods);
    posX += charWidth(ch);
  }
}

export function renderLineToBuffer(
  buf: ScreenBuffer,
  y: number,
  text: string,
  fg = 0,
  bg = 0,
  mods: CellModifiers = {}
): void {
  const lines = wrapLine(text, buf.width);
  const line = lines[0] ?? '';
  let x = 0;
  for (let i = 0; i < line.length && x < buf.width; i++) {
    const ch = line[i];
    if (ch === '\x1b') {
      const match = /^\x1b\[[0-9;?]*[a-zA-Z]/.exec(line.slice(i));
      if (match) {
        i += match[0].length - 1;
        continue;
      }
    }
    setCell(buf, x, y, ch, fg, bg, mods);
    x += charWidth(ch);
  }
}

export function clearBufferWithBg(buf: ScreenBuffer, bg = 0): void {
  clearBuffer(buf);
  for (let i = 0; i < buf.styles.length; i++) {
    const existing = buf.styles[i];
    const fgIdx = existing & 0xff;
    buf.styles[i] = (fgIdx & 0xff) | ((bg & 0xff) << 8);
  }
}

export function renderDividerToBuffer(
  buf: ScreenBuffer,
  y: number,
  char = '─',
  fg = 0,
  bg = 0,
  mods: CellModifiers = {}
): void {
  for (let x = 0; x < buf.width; x++) {
    setCell(buf, x, y, char, fg, bg, mods);
  }
}

export function renderPanelToBuffer(
  buf: ScreenBuffer,
  x: number,
  y: number,
  width: number,
  height: number,
  opts: {
    title?: string;
    titleFg?: number;
    borderFg?: number;
    bodyLines?: string[];
    bodyFg?: number;
  } = {}
): void {
  const { title, titleFg = 0, borderFg = 0, bodyLines = [], bodyFg = 0 } = opts;
  const innerW = width - 2;

  setCell(buf, x, y, '╭', borderFg, 0);
  for (let i = 1; i < width - 1; i++) {
    setCell(buf, x + i, y, '─', borderFg, 0);
  }
  setCell(buf, x + width - 1, y, '╮', borderFg, 0);

  if (title && innerW > 0) {
    const titleText = ` ${title} `;
    for (let i = 0; i < Math.min(titleText.length, innerW); i++) {
      setCell(buf, x + 1 + i, y, titleText[i], titleFg, 0);
    }
  }

  for (let row = 1; row < height - 1; row++) {
    setCell(buf, x, y + row, '│', borderFg, 0);
    const body = bodyLines[row - 1] ?? '';
    for (let col = 0; col < innerW; col++) {
      const ch = col < body.length ? body[col] : ' ';
      setCell(buf, x + 1 + col, y + row, ch, bodyFg, 0);
    }
    setCell(buf, x + width - 1, y + row, '│', borderFg, 0);
  }

  setCell(buf, x, y + height - 1, '╰', borderFg, 0);
  for (let i = 1; i < width - 1; i++) {
    setCell(buf, x + i, y + height - 1, '─', borderFg, 0);
  }
  setCell(buf, x + width - 1, y + height - 1, '╯', borderFg, 0);
}

export function renderBadgeToBuffer(
  buf: ScreenBuffer,
  x: number,
  y: number,
  text: string,
  fg = 0,
  bg = 0,
  mods: CellModifiers = {}
): void {
  const padded = ` ${text} `;
  for (let i = 0; i < padded.length; i++) {
    setCell(buf, x + i, y, padded[i], fg, bg, mods);
  }
}
