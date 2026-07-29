
import { describe, expect, it } from 'bun:test';
import {
  applyDiff,
  clearBuffer,
  cloneBuffer,
  computeDiff,
  createBuffer,
  getCell,
  packStyle,
  setCell,
  swapBuffers,
  unpackStyle
} from './buffer';

describe('packStyle / unpackStyle', () => {
  it('packs and unpacks basic style', () => {
    const packed = packStyle(1, 2, { bold: true, underline: true });
    const unpacked = unpackStyle(packed);
    expect(unpacked.fg).toBe(1);
    expect(unpacked.bg).toBe(2);
    expect(unpacked.modifiers.bold).toBe(true);
    expect(unpacked.modifiers.underline).toBe(true);
    expect(unpacked.modifiers.italic).toBe(false);
  });

  it('round-trips all modifier flags', () => {
    const packed = packStyle(3, 4, {
      bold: true,
      italic: true,
      underline: true,
      dim: true,
      inverse: true
    });
    const unpacked = unpackStyle(packed);
    expect(unpacked.fg).toBe(3);
    expect(unpacked.bg).toBe(4);
    expect(unpacked.modifiers.bold).toBe(true);
    expect(unpacked.modifiers.italic).toBe(true);
    expect(unpacked.modifiers.underline).toBe(true);
    expect(unpacked.modifiers.dim).toBe(true);
    expect(unpacked.modifiers.inverse).toBe(true);
  });
});

describe('createBuffer', () => {
  it('creates empty buffer with correct dimensions', () => {
    const buf = createBuffer(80, 24);
    expect(buf.width).toBe(80);
    expect(buf.height).toBe(24);
    expect(buf.cells.length).toBe(1920);
    expect(buf.styles.length).toBe(1920);
    expect(buf.cells.every((c) => c === ' ')).toBe(true);
    expect(buf.styles.every((s) => s === 0)).toBe(true);
  });
});

describe('setCell / getCell', () => {
  it('sets and retrieves a cell', () => {
    const buf = createBuffer(10, 10);
    setCell(buf, 3, 2, 'A', 5, 6, { bold: true });
    const cell = getCell(buf, 3, 2)!;
    expect(cell.char).toBe('A');
    expect(cell.fg).toBe(5);
    expect(cell.bg).toBe(6);
    expect(cell.modifiers.bold).toBe(true);
  });

  it('ignores out-of-bounds writes', () => {
    const buf = createBuffer(10, 10);
    setCell(buf, -1, 0, 'X', 0, 0);
    setCell(buf, 10, 0, 'X', 0, 0);
    setCell(buf, 0, -1, 'X', 0, 0);
    setCell(buf, 0, 10, 'X', 0, 0);
    expect(buf.cells.every((c) => c === ' ')).toBe(true);
  });
});

describe('clearBuffer', () => {
  it('resets all cells to space and style to 0', () => {
    const buf = createBuffer(10, 10);
    setCell(buf, 0, 0, 'X', 1, 2, { bold: true });
    setCell(buf, 5, 5, 'Y', 3, 4, { italic: true });
    clearBuffer(buf);
    expect(buf.cells.every((c) => c === ' ')).toBe(true);
    expect(buf.styles.every((s) => s === 0)).toBe(true);
  });
});

describe('cloneBuffer', () => {
  it('creates a deep copy', () => {
    const src = createBuffer(10, 10);
    setCell(src, 0, 0, 'A', 1, 2);
    setCell(src, 1, 1, 'B', 3, 4, { bold: true });
    const dst = cloneBuffer(src);
    expect(dst.cells).not.toBe(src.cells);
    expect(dst.styles).not.toBe(src.styles);
    expect(dst.cells[0 * 10 + 0]).toBe('A');
    expect(dst.cells[1 * 10 + 1]).toBe('B');
    // mutate source should not affect clone
    setCell(src, 0, 0, 'Z', 9, 9);
    expect(dst.cells[0]).toBe('A');
  });
});

describe('swapBuffers', () => {
  it('swaps cell and style arrays between front and back', () => {
    const front = createBuffer(10, 10);
    const back = createBuffer(10, 10);
    setCell(front, 0, 0, 'A', 1, 2);
    setCell(back, 1, 1, 'B', 3, 4);
    swapBuffers(front, back);
    expect(front.cells[1 * 10 + 1]).toBe('B');
    expect(back.cells[0]).toBe('A');
    // styles should also swap
    expect(front.styles[1 * 10 + 1]).not.toBe(0);
    expect(back.styles[0]).not.toBe(0);
  });
});

describe('computeDiff / applyDiff', () => {
  it('detects changed cells', () => {
    const front = createBuffer(10, 10);
    const back = createBuffer(10, 10);
    setCell(front, 0, 0, 'A', 1, 2);
    setCell(front, 2, 2, 'B', 3, 4, { bold: true });
    setCell(back, 0, 0, 'A', 1, 2); // same
    setCell(back, 1, 1, 'C', 5, 6); // back has char where front is empty
    const diff = computeDiff(front, back);
    expect(diff.ops.length).toBe(2);
    const opAt22 = diff.ops.find((o) => o.x === 2 && o.y === 2)!;
    expect(opAt22.char).toBe('B');
    const opAt11 = diff.ops.find((o) => o.x === 1 && o.y === 1)!;
    expect(opAt11.char).toBe(' ');
  });

  it('applyDiff patches a clone of back to match front', () => {
    const front = createBuffer(10, 10);
    const back = createBuffer(10, 10);
    setCell(front, 0, 0, 'A', 1, 2);
    setCell(front, 1, 0, 'B', 3, 4, { bold: true });
    setCell(back, 0, 0, 'A', 1, 2); // same
    setCell(back, 1, 0, 'X', 0, 0); // different
    const diff = computeDiff(front, back);
    applyDiff(back, diff);
    expect(back.cells[0]).toBe('A');
    expect(back.cells[1]).toBe('B');
    expect(back.styles[1]).toBe(packStyle(3, 4, { bold: true }));
  });

  it('empty diff when buffers identical', () => {
    const a = createBuffer(10, 10);
    const b = cloneBuffer(a);
    const diff = computeDiff(a, b);
    expect(diff.ops.length).toBe(0);
  });
});
