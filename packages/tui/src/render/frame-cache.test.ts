
import { describe, expect, it } from 'bun:test';
import { FrameCache } from './frame-cache';

const CLEAR = '\x1b[2J\x1b[H';

describe('FrameCache', () => {
  it('first flush does a full redraw with clear screen', () => {
    const fc = new FrameCache();
    const out = fc.flush(['line1', 'line2'], 80, 24);
    expect(out).toContain(CLEAR);
    expect(out).toContain('line1');
    expect(out).toContain('line2');
  });

  it('identical frame produces zero output', () => {
    const fc = new FrameCache();
    fc.flush(['a', 'b', 'c'], 80, 24);
    const out = fc.flush(['a', 'b', 'c'], 80, 24);
    expect(out).toBe('');
  });

  it('only rewrites changed lines', () => {
    const fc = new FrameCache();
    fc.flush(['a', 'b', 'c'], 80, 24);
    const out = fc.flush(['a', 'CHANGED', 'c'], 80, 24);
    expect(out).not.toContain(CLEAR);
    expect(out).toContain('\x1b[2;1H');
    expect(out).toContain('CHANGED');
    expect(out).not.toContain('\x1b[1;1H');
    expect(out).not.toContain('\x1b[3;1H');
  });

  it('clears leftover lines when new frame is shorter', () => {
    const fc = new FrameCache();
    fc.flush(['a', 'b', 'c', 'd'], 80, 24);
    const out = fc.flush(['a', 'b'], 80, 24);
    expect(out).toContain('\x1b[3;1H\x1b[2K');
    expect(out).toContain('\x1b[4;1H\x1b[2K');
  });

  it('terminal resize triggers full redraw', () => {
    const fc = new FrameCache();
    fc.flush(['a', 'b'], 80, 24);
    const out = fc.flush(['a', 'b'], 100, 24);
    expect(out).toContain(CLEAR);
  });

  it('invalidate forces full redraw on next flush', () => {
    const fc = new FrameCache();
    fc.flush(['a'], 80, 24);
    fc.invalidate();
    const out = fc.flush(['a'], 80, 24);
    expect(out).toContain(CLEAR);
  });

  it('wraps output in autowrap off/on guards', () => {
    const fc = new FrameCache();
    const out = fc.flush(['x'], 80, 24);
    expect(out.startsWith('\x1b[?7l')).toBe(true);
    expect(out.endsWith('\x1b[?7h')).toBe(true);
  });

  it('appends erase-to-EOL after each rewritten line', () => {
    const fc = new FrameCache();
    fc.flush(['long line here'], 80, 24);
    const out = fc.flush(['short'], 80, 24);
    expect(out).toContain('short\x1b[K');
  });
});
