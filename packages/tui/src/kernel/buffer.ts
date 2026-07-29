
export type CellModifiers = {
  bold?: boolean;
  italic?: boolean;
  underline?: boolean;
  dim?: boolean;
  inverse?: boolean;
};

export interface ScreenCell {
  char: string;
  fg: number; // palette index
  bg: number; // palette index
  modifiers: number;
}

export interface ScreenBuffer {
  cells: string[];
  styles: Int32Array;
  width: number;
  height: number;
}

export interface DiffResult {
  ops: DiffOp[];
}

export interface DiffOp {
  type: 'set' | 'clear' | 'move';
  x: number;
  y: number;
  char?: string;
  style?: number;
}

const MOD_BOLD = 1 << 0;
const MOD_ITALIC = 1 << 1;
const MOD_UNDERLINE = 1 << 2;
const MOD_DIM = 1 << 3;
const MOD_INVERSE = 1 << 4;

export function packStyle(fg: number, bg: number, mods: CellModifiers = {}): number {
  let m = 0;
  if (mods.bold) m |= MOD_BOLD;
  if (mods.italic) m |= MOD_ITALIC;
  if (mods.underline) m |= MOD_UNDERLINE;
  if (mods.dim) m |= MOD_DIM;
  if (mods.inverse) m |= MOD_INVERSE;
  return (fg & 0xff) | ((bg & 0xff) << 8) | ((m & 0xff) << 16);
}

export function unpackStyle(style: number): { fg: number; bg: number; modifiers: CellModifiers } {
  const fg = style & 0xff;
  const bg = (style >> 8) & 0xff;
  const m = (style >> 16) & 0xff;
  return {
    fg,
    bg,
    modifiers: {
      bold: (m & MOD_BOLD) !== 0,
      italic: (m & MOD_ITALIC) !== 0,
      underline: (m & MOD_UNDERLINE) !== 0,
      dim: (m & MOD_DIM) !== 0,
      inverse: (m & MOD_INVERSE) !== 0
    }
  };
}

export function createBuffer(width: number, height: number): ScreenBuffer {
  const size = width * height;
  return {
    cells: new Array(size).fill(' '),
    styles: new Int32Array(size),
    width,
    height
  };
}

export function clearBuffer(buf: ScreenBuffer): void {
  buf.cells.fill(' ');
  buf.styles.fill(0);
}

export function cloneBuffer(src: ScreenBuffer): ScreenBuffer {
  const dst = createBuffer(src.width, src.height);
  dst.cells = [...src.cells];
  dst.styles.set(src.styles);
  return dst;
}

export function setCell(
  buf: ScreenBuffer,
  x: number,
  y: number,
  char: string,
  fg: number,
  bg: number,
  mods: CellModifiers = {}
): void {
  if (x < 0 || x >= buf.width || y < 0 || y >= buf.height) return;
  const idx = y * buf.width + x;
  buf.cells[idx] = char;
  buf.styles[idx] = packStyle(fg, bg, mods);
}

export function getCell(buf: ScreenBuffer, x: number, y: number): ScreenCell | null {
  if (x < 0 || x >= buf.width || y < 0 || y >= buf.height) return null;
  const idx = y * buf.width + x;
  return {
    char: buf.cells[idx],
    ...unpackStyle(buf.styles[idx])
  };
}

export function swapBuffers(front: ScreenBuffer, back: ScreenBuffer): void {
  const tmpCells = front.cells;
  const tmpStyles = front.styles;
  front.cells = back.cells;
  front.styles = back.styles;
  back.cells = tmpCells;
  back.styles = tmpStyles;
}

export function computeDiff(front: ScreenBuffer, back: ScreenBuffer): DiffResult {
  const ops: DiffOp[] = [];
  const len = front.cells.length;

  for (let i = 0; i < len; i++) {
    if (front.cells[i] !== back.cells[i] || front.styles[i] !== back.styles[i]) {
      const x = i % front.width;
      const y = Math.floor(i / front.width);
      ops.push({
        type: 'set',
        x,
        y,
        char: front.cells[i],
        style: front.styles[i]
      });
    }
  }

  return { ops };
}

export function applyDiff(buf: ScreenBuffer, diff: DiffResult): void {
  for (const op of diff.ops) {
    if (op.type === 'set' && op.char !== undefined) {
      const idx = op.y * buf.width + op.x;
      buf.cells[idx] = op.char;
      if (op.style !== undefined) {
        buf.styles[idx] = op.style;
      }
    }
  }
}
