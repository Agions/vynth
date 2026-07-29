
import type { DiffOp, DiffResult, ScreenBuffer } from '../kernel/buffer';
import { computeDiff, unpackStyle } from '../kernel/buffer';
import { reset } from '../theme';
import type { ColorRegistry } from './colors';

export interface DiffRenderOptions {
  resetBetweenCells?: boolean;
  runLengthMerge?: boolean;
  colors?: ColorRegistry;
}

export interface DiffChunk {
  output: string;
  cells: number;
}

export function renderDiff(
  front: ScreenBuffer,
  back: ScreenBuffer,
  opts: DiffRenderOptions = {}
): DiffChunk[] {
  const { resetBetweenCells = false, runLengthMerge = true, colors } = opts;
  const diff = computeDiff(front, back);
  const chunks: DiffChunk[] = [];
  const parts: string[] = [];
  let cells = 0;

  const rows = new Map<number, DiffOp[]>();
  for (const op of diff.ops) {
    const rowOps = rows.get(op.y) || [];
    rowOps.push(op);
    rows.set(op.y, rowOps);
  }

  for (const [y, ops] of rows) {
    if (runLengthMerge) {
      ops.sort((a, b) => a.x - b.x);
      const runs: { x: number; char: string; style: number; len: number }[] = [];
      for (const op of ops) {
        if (op.type !== 'set' || op.char === undefined) continue;
        const prev = runs[runs.length - 1];
        if (prev && prev.x + prev.len === op.x && prev.style === op.style) {
          prev.len++;
        } else {
          runs.push({ x: op.x, char: op.char, style: op.style, len: 1 });
        }
      }

      for (const run of runs) {
        const { fg: fgIdx, bg: bgIdx, modifiers } = unpackStyle(run.style);
        const styleStr = buildStyleString(fgIdx, bgIdx, modifiers, colors);
        // cursorTo + style + chars
        parts.push(`\x1b[${y + 1};${run.x + 1}H`, styleStr, run.char.repeat(run.len));
        cells += run.len;
        if (resetBetweenCells) parts.push(reset);
      }
    } else {
      for (const op of ops) {
        if (op.type !== 'set' || op.char === undefined) continue;
        const { fg: fgIdx, bg: bgIdx, modifiers } = unpackStyle(op.style);
        const styleStr = buildStyleString(fgIdx, bgIdx, modifiers, colors);
        parts.push(`\x1b[${op.y + 1};${op.x + 1}H`, styleStr, op.char);
        cells++;
        if (resetBetweenCells) parts.push(reset);
      }
    }
  }

  if (parts.length > 0) {
    chunks.push({ output: parts.join(''), cells });
  }

  return chunks;
}

function buildStyleString(
  fgIdx: number,
  bgIdx: number,
  modifiers: {
    bold?: boolean;
    italic?: boolean;
    underline?: boolean;
    dim?: boolean;
    inverse?: boolean;
  },
  colors?: ColorRegistry
): string {
  const parts: string[] = [];

  if (colors) {
    const fgSeq = colors.ansiFg(fgIdx);
    const bgSeq = colors.ansiBg(bgIdx);
    if (fgSeq) parts.push(fgSeq);
    if (bgSeq) parts.push(bgSeq);
  } else {
    if (fgIdx > 0) parts.push(`\x1b[38;5;${fgIdx}m`);
    if (bgIdx > 0) parts.push(`\x1b[48;5;${bgIdx}m`);
  }

  if (modifiers.bold) parts.push('\x1b[1m');
  if (modifiers.italic) parts.push('\x1b[3m');
  if (modifiers.underline) parts.push('\x1b[4m');
  if (modifiers.dim) parts.push('\x1b[2m');
  if (modifiers.inverse) parts.push('\x1b[7m');

  return parts.join('');
}

export function diffCellCount(diff: DiffResult): number {
  return diff.ops.length;
}
