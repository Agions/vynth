import { describe, expect, it } from 'bun:test';
import { createBuffer, setCell } from '../kernel/buffer';
import { ColorRegistry, hexToAnsi256, parseHex, supportsTruecolor } from './colors';
import { renderDiff } from './diff';

describe('supportsTruecolor', () => {
  it('detects COLORTERM=truecolor', () => {
    expect(supportsTruecolor({ COLORTERM: 'truecolor' })).toBe(true);
    expect(supportsTruecolor({ COLORTERM: '24bit' })).toBe(true);
  });

  it('detects TERM=*-direct', () => {
    expect(supportsTruecolor({ TERM: 'xterm-direct' })).toBe(true);
  });

  it('detects known terminal programs', () => {
    expect(supportsTruecolor({ TERM_PROGRAM: 'iTerm.app' })).toBe(true);
    expect(supportsTruecolor({ TERM_PROGRAM: 'WezTerm' })).toBe(true);
    expect(supportsTruecolor({ TERM_PROGRAM: 'vscode' })).toBe(true);
  });

  it('falls back to false for plain xterm-256color', () => {
    expect(supportsTruecolor({ TERM: 'xterm-256color' })).toBe(false);
    expect(supportsTruecolor({})).toBe(false);
  });
});

describe('parseHex', () => {
  it('parses #rrggbb', () => {
    expect(parseHex('#cba6f7')).toEqual([203, 166, 247]);
    expect(parseHex('1e1e2e')).toEqual([30, 30, 46]);
  });

  it('rejects invalid input', () => {
    expect(parseHex('#fff')).toBeNull();
    expect(parseHex('not-a-color')).toBeNull();
  });
});

describe('hexToAnsi256', () => {
  it('maps pure colors to cube', () => {
    expect(hexToAnsi256('#000000')).toBe(16);
    expect(hexToAnsi256('#ff0000')).toBe(196);
    expect(hexToAnsi256('#00ff00')).toBe(46);
  });

  it('maps near-grays to grayscale ramp', () => {
    const idx = hexToAnsi256('#808080');
    expect(idx).toBeGreaterThanOrEqual(232);
    expect(idx).toBeLessThanOrEqual(255);
  });
});

describe('ColorRegistry', () => {
  it('returns stable indices for repeated hex', () => {
    const reg = new ColorRegistry({ truecolor: true });
    const a = reg.index('#cba6f7');
    const b = reg.index('#CBA6F7');
    const c = reg.index('#94e2d5');
    expect(a).toBe(b);
    expect(c).not.toBe(a);
    expect(a).toBeGreaterThan(0);
    expect(reg.size).toBe(2);
  });

  it('emits truecolor sequences when supported', () => {
    const reg = new ColorRegistry({ truecolor: true });
    const idx = reg.index('#cba6f7');
    expect(reg.ansiFg(idx)).toBe('\x1b[38;2;203;166;247m');
    expect(reg.ansiBg(idx)).toBe('\x1b[48;2;203;166;247m');
  });

  it('degrades to 256-color when truecolor unsupported', () => {
    const reg = new ColorRegistry({ truecolor: false });
    const idx = reg.index('#ff0000');
    expect(reg.ansiFg(idx)).toBe('\x1b[38;5;196m');
  });

  it('returns empty string for index 0 (terminal default)', () => {
    const reg = new ColorRegistry({ truecolor: true });
    expect(reg.ansiFg(0)).toBe('');
    expect(reg.ansiBg(0)).toBe('');
  });
});

describe('renderDiff with ColorRegistry', () => {
  it('outputs truecolor sequences instead of raw 256 indices', () => {
    const reg = new ColorRegistry({ truecolor: true });
    const mauve = reg.index('#cba6f7');

    const front = createBuffer(10, 3);
    const back = createBuffer(10, 3);
    setCell(front, 0, 0, 'Z', mauve, 0);

    const chunks = renderDiff(front, back, { colors: reg });
    expect(chunks.length).toBe(1);
    expect(chunks[0].output).toContain('\x1b[38;2;203;166;247m');
    expect(chunks[0].output).not.toContain(`\x1b[38;5;${mauve}m`);
  });

  it('keeps legacy 256-index behavior without registry', () => {
    const front = createBuffer(10, 3);
    const back = createBuffer(10, 3);
    setCell(front, 0, 0, 'Z', 5, 0);

    const chunks = renderDiff(front, back);
    expect(chunks[0].output).toContain('\x1b[38;5;5m');
  });
});
