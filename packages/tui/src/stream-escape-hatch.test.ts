import { describe, expect, it } from 'bun:test';
import { StreamArea } from './stream-escape-hatch';

const ESC = String.fromCharCode(0x1b);
const clearLineSeq = `${ESC}[G${ESC}[2K`;

describe('StreamArea (F3 headless 流式退路 / F2 TUI 流式退路)', () => {
  it('first update writes text directly without a leading clearLine', () => {
    const writes: string[] = [];
    const area = new StreamArea((s) => writes.push(s));
    area.update('hello');
    expect(writes).toEqual(['hello']);
  });

  it('second update clears previous line then writes new text', () => {
    const writes: string[] = [];
    const area = new StreamArea((s) => writes.push(s));
    area.update('first');
    area.update('second');
    expect(writes).toEqual(['first', clearLineSeq, 'second']);
  });

  it('clear() after update emits cursorLeft + clearLine and resets lastLen', () => {
    const writes: string[] = [];
    const area = new StreamArea((s) => writes.push(s));
    area.update('x');
    writes.length = 0;
    area.clear();
    expect(writes).toEqual([clearLineSeq]);
    // 再 update 应该不再擦线（lastLen=0）
    writes.length = 0;
    area.update('y');
    expect(writes).toEqual(['y']);
  });

  it('clear() when nothing was written is a no-op', () => {
    const writes: string[] = [];
    const area = new StreamArea((s) => writes.push(s));
    area.clear();
    expect(writes).toEqual([]);
  });

  it('empty-string update is safe and does not crash', () => {
    const writes: string[] = [];
    const area = new StreamArea((s) => writes.push(s));
    expect(() => area.update('')).not.toThrow();
    // lastLen 是文本"长度"，空串应让下一次 update 也不发出擦线（边界 case 当前实现允许 lastLen=0）
    area.update('a');
    expect(writes).toEqual(['', 'a']);
  });
});
