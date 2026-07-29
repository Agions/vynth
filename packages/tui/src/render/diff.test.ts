
import { describe, expect, it } from 'bun:test';
import { clearBuffer, createBuffer, setCell } from '../kernel/buffer';
import { diffCellCount, renderDiff } from './diff';

describe('renderDiff', () => {
  it('returns empty chunks for identical buffers', () => {
    const front = createBuffer(10, 5);
    const back = createBuffer(10, 5);
    setCell(front, 0, 0, 'A', 1, 2);
    setCell(back, 0, 0, 'A', 1, 2);
    const chunks = renderDiff(front, back);
    expect(chunks.length).toBe(0);
  });

  it('renders changed cells with cursor positioning', () => {
    const front = createBuffer(10, 5);
    const back = createBuffer(10, 5);
    setCell(front, 2, 1, 'X', 3, 4);
    setCell(front, 5, 2, 'Y', 5, 6, { bold: true });
    setCell(back, 2, 1, 'X', 3, 4); // same as front
    setCell(back, 5, 2, ' ', 0, 0); // different from front
    const chunks = renderDiff(front, back);
    expect(chunks.length).toBe(1);
    const out = chunks[0].output;
    expect(out).toContain('\x1b[3;6H'); // y=2, x=5
    expect(out).toContain('Y');
  });

  it('merges consecutive cells with same style into run', () => {
    const front = createBuffer(10, 5);
    const back = createBuffer(10, 5);
    setCell(front, 0, 0, 'A', 1, 2);
    setCell(front, 1, 0, 'A', 1, 2);
    setCell(front, 2, 0, 'A', 1, 2);
    setCell(back, 0, 0, ' ', 0, 0);
    setCell(back, 1, 0, ' ', 0, 0);
    setCell(back, 2, 0, ' ', 0, 0);
    const chunks = renderDiff(front, back);
    expect(chunks[0].cells).toBe(3);
    expect(chunks[0].output).toContain('AAA');
  });

  it('counts diff cells correctly', () => {
    const front = createBuffer(10, 5);
    const back = createBuffer(10, 5);
    setCell(front, 0, 0, 'A', 1, 2);
    setCell(front, 1, 1, 'B', 3, 4);
    setCell(back, 0, 0, 'A', 1, 2); // same
    const diff = { ops: [] as any[] };
    expect(diffCellCount(diff)).toBe(0);
  });
});
