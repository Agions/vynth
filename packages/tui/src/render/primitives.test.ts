
import { describe, expect, it } from 'bun:test';
import { createBuffer, setCell } from '../kernel/buffer';
import {
  clearBufferWithBg,
  renderBadgeToBuffer,
  renderDividerToBuffer,
  renderLineToBuffer,
  renderPanelToBuffer,
  renderTextToBuffer
} from './primitives';

describe('renderTextToBuffer', () => {
  it('renders plain text at position', () => {
    const buf = createBuffer(10, 5);
    renderTextToBuffer(buf, 2, 1, 'AB', 3, 4);
    expect(buf.cells[1 * 10 + 2]).toBe('A');
    expect(buf.cells[1 * 10 + 3]).toBe('B');
    expect(buf.styles[1 * 10 + 2]).toBeGreaterThan(0);
  });

  it('skips ANSI escape sequences', () => {
    const buf = createBuffer(10, 5);
    renderTextToBuffer(buf, 0, 0, '\x1b[31mRed\x1b[0m', 1, 2);
    expect(buf.cells[0]).toBe('R');
    expect(buf.cells[1]).toBe('e');
    expect(buf.cells[2]).toBe('d');
  });
});

describe('renderLineToBuffer', () => {
  it('renders and truncates to buffer width', () => {
    const buf = createBuffer(5, 5);
    renderLineToBuffer(buf, 0, 'HelloWorld', 1, 2);
    expect(buf.cells.slice(0, 5).join('')).toBe('Hello');
  });

  it('wraps CJK text correctly', () => {
    const buf = createBuffer(4, 10);
    renderLineToBuffer(buf, 0, '你好世界', 1, 2);
    expect(buf.cells[0]).toBe('你');
    expect(buf.cells[2]).toBe('好');
    expect(buf.cells[4]).toBe(' ');
  });
});

describe('clearBufferWithBg', () => {
  it('clears cells and sets background', () => {
    const buf = createBuffer(10, 5);
    setCell(buf, 0, 0, 'X', 1, 2);
    clearBufferWithBg(buf, 5);
    expect(buf.cells[0]).toBe(' ');
    expect(buf.styles[0]).toBe(5 << 8); // bg=5, fg=0
  });
});

describe('renderDividerToBuffer', () => {
  it('fills row with divider char', () => {
    const buf = createBuffer(10, 5);
    renderDividerToBuffer(buf, 2, '=', 3, 4);
    expect(buf.cells.slice(2 * 10, 2 * 10 + 10).join('')).toBe('==========');
  });
});

describe('renderPanelToBuffer', () => {
  it('renders panel with border and body', () => {
    const buf = createBuffer(10, 5);
    renderPanelToBuffer(buf, 0, 0, 10, 5, {
      title: 'Test',
      bodyLines: ['Line 1', 'Line 2']
    });
    expect(buf.cells[0]).toBe('╭');
    expect(buf.cells[9]).toBe('╮');
    expect(buf.cells[1 * 10]).toBe('│');
    expect(buf.cells[1 * 10 + 1]).toBe('L'); // body "Line 1"
    expect(buf.cells[1 * 10 + 9]).toBe('│');
  });
});

describe('renderBadgeToBuffer', () => {
  it('renders padded text', () => {
    const buf = createBuffer(10, 5);
    renderBadgeToBuffer(buf, 2, 1, 'OK', 1, 2);
    expect(buf.cells[1 * 10 + 2]).toBe(' ');
    expect(buf.cells[1 * 10 + 3]).toBe('O');
    expect(buf.cells[1 * 10 + 4]).toBe('K');
    expect(buf.cells[1 * 10 + 5]).toBe(' ');
  });
});
