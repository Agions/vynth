/**
 * 渲染原语单元测试：visibleWidth / wrapLine / renderMessage / renderPanel 等。
 * 不写 stdout，纯函数验证。
 */
import { describe, expect, it } from 'bun:test';
import {
  renderBadge,
  renderInputArea,
  renderMessage,
  renderPanel,
  renderSection,
  renderStatusBar,
  renderToolBlock,
  visibleWidth,
  wrapLine
} from './render';
import { palette } from './theme';

describe('render primitives', () => {
  it('visibleWidth treats CJK as 2 columns and ignores ANSI', () => {
    const s = '\x1b[38;2;1;2;3m中\x1b[0mA';
    expect(visibleWidth(s)).toBe(3);
  });

  it('visibleWidth handles ASCII correctly', () => {
    expect(visibleWidth('hello world')).toBe(11);
  });

  it('wrapLine keeps ANSI sequences intact and breaks on visible width', () => {
    const input = '\x1b[38;2;200;100;50mxxxxxx xxxx xxxx xxxx xxxx\x1b[0m';
    const out = wrapLine(input, 8);
    // 至少两行；每行去掉 ANSI 后宽度不超过 8
    expect(out.length).toBeGreaterThanOrEqual(2);
    for (const ln of out) {
      expect(visibleWidth(ln)).toBeLessThanOrEqual(8);
    }
  });

  it('renderBadge returns styled text', () => {
    const text = renderBadge('VIBE', '#cdd6f4');
    expect(text).toContain('VIBE');
  });

  it('renderSection produces a separator with centered title', () => {
    const out = renderSection('Title', '#cdd6f4', 30);
    expect(out).toContain('Title');
    expect(visibleWidth(out)).toBe(30);
  });

  it('renderStatusBar fills width with left/right', () => {
    const out = renderStatusBar({
      width: 40,
      left: 'LEFT',
      right: 'RIGHT',
      color: '#cdd6f4',
      bgHex: '#1e1e2e',
      textColor: '#cdd6f4'
    });
    expect(visibleWidth(out)).toBe(40);
    expect(out).toContain('LEFT');
    expect(out).toContain('RIGHT');
  });

  it('renderMessage emits a left color bar (no role label)', () => {
    const out = renderMessage({
      role: 'user',
      content: 'hello',
      palette: palette('mocha'),
      width: 40
    });
    expect(out).not.toContain('You');
    expect(out).not.toContain('System');
    expect(out).not.toContain('Vynth');
    expect(out).toContain('hello');
    expect(out).toContain('▎');
  });

  it('renderToolBlock marks running/ok/error distinctly', () => {
    const c = palette('mocha');
    expect(
      renderToolBlock({ name: 'read_file', args: '{}', status: 'ok', palette: c, width: 30 })
    ).toContain('read_file');
    expect(
      renderToolBlock({ name: 'read_file', args: '{}', status: 'error', palette: c, width: 30 })
    ).toContain('✗');
    expect(
      renderToolBlock({ name: 'read_file', args: '{}', status: 'running', palette: c, width: 30 })
    ).toContain('◐');
  });

  it('renderPanel wraps body lines and renders top/divider/bottom', () => {
    const out = renderPanel({
      width: 30,
      title: 'Hello',
      titleAlign: 'center',
      body: ['line one', 'line two'],
      borderColor: '#cdd6f4',
      titleColor: '#cba6f7'
    });
    expect(out).toContain('╭');
    expect(out).toContain('╯');
    expect(out).toContain('line one');
    expect(out).toContain('line two');
  });

  it('renderInputArea emits 5 lines: separator / blank / input / blank / hints', () => {
    const lines = renderInputArea({
      width: 60,
      input: 'hello world',
      model: 'deepseek-v4-pro',
      mode: 'vibe',
      theme: 'mocha',
      status: 'ready',
      statusColor: '#a6e3a1',
      palette: palette('mocha')
    });
    expect(lines.length).toBe(5);
    // Line 1: context separator with model name
    expect(lines[0]).toContain('deepseek-v4-pro');
    expect(lines[0]).toContain('vibe');
    expect(lines[0]).toContain('ready');
    expect(lines[0]).toContain('─');
    // Line 2: blank
    expect(lines[1]).toBe('');
    // Line 3: input with prompt
    expect(lines[2]).toContain('❯');
    expect(lines[2]).toContain('hello world');
    // Line 4: blank
    expect(lines[3]).toBe('');
    // Line 5: hints with key badges
    expect(lines[4]).toContain('send');
    expect(lines[4]).toContain('esc');
    expect(lines[4]).toContain('quit');
    expect(lines[4]).toContain('scroll');
  });
});
