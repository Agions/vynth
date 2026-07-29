import { describe, expect, it } from 'bun:test';
import { Scrollback } from './scrollback';
import {
  computeLayout,
  enableMouseTracking,
  enterAltScreen,
  parseMouse,
  setScrollRegion
} from './viewport';

describe('viewport layout', () => {
  it('computes three segments within rows', () => {
    const layout = computeLayout(32, 1, 3);
    expect(layout.topStart).toBe(1);
    expect(layout.topEnd).toBeGreaterThanOrEqual(1);
    expect(layout.midStart).toBe(layout.topEnd + 1);
    expect(layout.botEnd).toBe(32);
    expect(layout.botEnd).toBeGreaterThan(layout.midEnd);
    expect(layout.midEnd - layout.midStart + 1).toBeGreaterThanOrEqual(5);
  });

  it('emits DECSTBM sequences for top/mid/bot', () => {
    const r = setScrollRegion(1, 30, 32);
    expect(r.top).toContain('[1;1r');
    expect(r.mid).toContain('[2;29r');
    expect(r.bot).toContain('[30;32r');
  });

  it('exposes alternate screen + mouse tracking sequences', () => {
    expect(enterAltScreen).toContain('1049');
    expect(enableMouseTracking).toContain('1006');
  });
});

describe('parseMouse', () => {
  it('parses SGR wheel-up (button 64)', () => {
    const out = parseMouse('\x1b[<64;10;5m');
    expect(out?.event.kind).toBe('wheel');
    expect(out?.event.button).toBe(64);
    expect(out?.event.row).toBe(5);
    expect(out?.event.col).toBe(10);
  });

  it('parses SGR wheel-down (button 65)', () => {
    const out = parseMouse('\x1b[<65;1;3M');
    expect(out?.event.button).toBe(65);
    expect(out?.event.kind).toBe('wheel');
  });

  it('returns null for non-mouse data', () => {
    expect(parseMouse('hello')).toBeNull();
  });
});

describe('Scrollback', () => {
  it('appends and respects capacity', () => {
    const sb = new Scrollback(3);
    sb.push('one\ntwo');
    sb.push('three\nfour');
    expect(sb.size()).toBe(3);
    expect(sb.all()).toEqual(['two', 'three', 'four']);
  });

  it('visibleForViewport returns last N', () => {
    const sb = new Scrollback(100);
    for (let i = 0; i < 10; i++) sb.push(`line ${i}`);
    expect(sb.visibleForViewport(3)).toEqual(['line 7', 'line 8', 'line 9']);
  });

  it('reset replaces contents and trims to capacity', () => {
    const sb = new Scrollback(3);
    sb.push('a\nb\nc');
    sb.reset('d\ne\nf\ng');
    expect(sb.size()).toBe(3);
    expect(sb.all()).toEqual(['e', 'f', 'g']);
  });

  it('scrollUp slices off the bottom', () => {
    const sb = new Scrollback(100);
    for (let i = 0; i < 5; i++) sb.push(`line ${i}`);
    expect(sb.scrollUp(2)).toEqual(['line 0', 'line 1', 'line 2']);
  });
});
