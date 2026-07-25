import { describe, expect, it } from 'bun:test';
import { bg, fg, palette, reset } from './theme';

describe('theme (catppuccin mocha | latte)', () => {
  it('palette("mocha") returns dark base and mauve accent', () => {
    const p = palette('mocha');
    expect(p.base).toBe('#1e1e2e');
    expect(p.text).toBe('#cdd6f4');
    expect(p.mauve).toBe('#cba6f7');
  });

  it('palette("latte") returns light base and mauve accent (distinct from mocha)', () => {
    const p = palette('latte');
    expect(p.base).toBe('#eff1f5');
    expect(p.text).toBe('#4c4f69');
    expect(p.mauve).toBe('#8839ef');
    // mocha and latte must NOT share the same base — otherwise dark vs light themes collapse
    expect(palette('mocha').base).not.toBe(palette('latte').base);
  });

  it('fg(hex) emits 24-bit ANSI fg escape', () => {
    expect(fg('#ff0000')).toBe('\x1b[38;2;255;0;0m');
    expect(fg('#00ff00')).toBe('\x1b[38;2;0;255;0m');
    expect(fg('#0000ff')).toBe('\x1b[38;2;0;0;255m');
  });

  it('bg(hex) emits 24-bit ANSI bg escape', () => {
    expect(bg('#1e1e2e')).toBe('\x1b[48;2;30;30;46m');
  });

  it('reset is the standard ANSI reset sequence', () => {
    expect(reset).toBe('\x1b[0m');
  });
});
