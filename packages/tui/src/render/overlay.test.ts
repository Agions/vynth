import { describe, expect, test } from 'bun:test';
import { composeOverlay, overlayPanelWidth, stripPanelBorders, withBackground } from './overlay';

const strip = (s: string): string => s.replace(/\x1b\[[0-9;]*m/g, '');

function makeFrame(rows: number, fill = 'x'): string[] {
  return Array.from({ length: rows }, (_, i) => fill.repeat(10) + String(i));
}

describe('withBackground', () => {
  const BG = '\x1b[48;2;30;30;46m';

  test('整行首尾包裹背景序列', () => {
    const out = withBackground('hello', BG);
    expect(out.startsWith(BG)).toBe(true);
    expect(out.endsWith('\x1b[0m')).toBe(true);
  });

  test('行内 reset 之后重新注入背景（背景连续不断）', () => {
    const line = `a\x1b[0mb`;
    const out = withBackground(line, BG);
    expect(out).toContain(`\x1b[0m${BG}b`);
  });
});

describe('composeOverlay', () => {
  test('浮窗行覆写帧行，帧总行数不变（布局零位移）', () => {
    const frame = makeFrame(20);
    const before = frame.length;
    composeOverlay(frame, ['AAA', 'BBB', 'CCC'], { left: 2, width: 8, anchorBottom: 15 });
    expect(frame.length).toBe(before);
  });

  test('底边锚定：最后一行落在 anchorBottom - 1', () => {
    const frame = makeFrame(20);
    composeOverlay(frame, ['AAA', 'BBB'], { left: 2, width: 8, anchorBottom: 15 });
    expect(strip(frame[14])).toContain('BBB');
    expect(strip(frame[13])).toContain('AAA');
    expect(strip(frame[15])).toBe('xxxxxxxxxx15');
  });

  test('保留浮窗左侧的底层内容', () => {
    const frame = makeFrame(20);
    composeOverlay(frame, ['AAA'], { left: 4, width: 8, anchorBottom: 10 });
    expect(strip(frame[9]).startsWith('xxxx')).toBe(true);
  });

  test('空间不足时从顶部裁剪（footer 保留）', () => {
    const frame = makeFrame(20);
    const overlay = ['t1', 't2', 't3', 't4', 't5', 'footer'];
    composeOverlay(frame, overlay, { left: 0, width: 8, anchorBottom: 7, minRow: 3 });
    expect(strip(frame[6])).toContain('footer');
    expect(strip(frame[3])).toContain('t3');
    expect(strip(frame[2])).toBe('xxxxxxxxxx2');
  });

  test('bgSeq 时每行携带实底背景', () => {
    const frame = makeFrame(20);
    const BG = '\x1b[48;2;1;2;3m';
    composeOverlay(frame, ['AAA'], { left: 0, width: 6, anchorBottom: 10, bgSeq: BG });
    expect(frame[9]).toContain(BG);
  });

  test('内容行截断/填充到定宽', () => {
    const frame = makeFrame(20);
    composeOverlay(frame, ['toolongcontenthere'], { left: 0, width: 6, anchorBottom: 10 });
    expect(strip(frame[9]).length).toBe(6);
  });

  test('空 overlay 是 no-op', () => {
    const frame = makeFrame(5);
    const snapshot = [...frame];
    composeOverlay(frame, [], { left: 0, width: 6, anchorBottom: 4 });
    expect(frame).toEqual(snapshot);
  });
});

describe('overlayPanelWidth', () => {
  test('取最长可见行宽 + padding', () => {
    expect(overlayPanelWidth(['ab', 'abcd'], 2, 80)).toBe(6);
  });

  test('钳制到 maxWidth', () => {
    expect(overlayPanelWidth(['a'.repeat(100)], 2, 40)).toBe(40);
  });

  test('忽略 ANSI 序列宽度', () => {
    expect(overlayPanelWidth(['\x1b[31mab\x1b[0m'], 0, 80)).toBe(2);
  });
});

describe('stripPanelBorders', () => {
  test('顶边框行：剥掉框角与横线，保留内嵌标题', () => {
    const top = '\x1b[38;2;1;2;3m╭─ 📊 Token 用量统计 ──────╮\x1b[0m';
    const [out] = stripPanelBorders([top]);
    const p = out.replace(/\x1b\[[0-9;]*m/g, '');
    expect(p).toContain('Token 用量统计');
    expect(p).not.toMatch(/[╭╮─]{2,}/);
    expect(p).not.toContain('╭');
    expect(p).not.toContain('╮');
  });

  test('底边框行：保留 footer 文字', () => {
    const bot = '╰─ esc 关闭 ─────╯';
    const [out] = stripPanelBorders([bot]);
    expect(out).toContain('esc 关闭');
    expect(out).not.toContain('╰');
  });

  test('纯边框行（无标题）变为空行，维持面板高度', () => {
    const [out] = stripPanelBorders(['╭──────────╮']);
    expect(out).toBe('');
  });

  test('中间行：剥掉首尾 │，内容保留', () => {
    const mid = '│  Model  gpt-x  │';
    const [out] = stripPanelBorders([mid]);
    expect(out).toContain('Model  gpt-x');
    expect(out).not.toContain('│');
  });

  test('内容中的 │ 不受影响（仅剥首尾）', () => {
    const mid = '│ a │ b │';
    const [out] = stripPanelBorders([mid]);
    expect(out.split('│').length - 1).toBe(1);
  });

  test('无边框行原样返回', () => {
    const plain = '  ❯ /theme  切换主题';
    const [out] = stripPanelBorders([plain]);
    expect(out).toBe(plain);
  });

  test('行内分组标题横线（── xx ──）也被剥掉，文字保留', () => {
    const mid = '│  ── 近期请求明细 ──  │';
    const [out] = stripPanelBorders([mid]);
    expect(out).toContain('近期请求明细');
    expect(out).not.toContain('─');
  });

  test('剥完不残留任何单个 ─（如标题行 "─ 标题 ─"）', () => {
    const top = '\x1b[38;2;1;2;3m╭─ ⚙ AI 配置中心 ──────╮\x1b[0m';
    const [out] = stripPanelBorders([top]);
    const p = out.replace(/\x1b\[[0-9;]*m/g, '');
    expect(p).toContain('AI 配置中心');
    expect(p).not.toContain('─');
  });

  test('无侧边框的分组标题行（SlashPalette 风格）横线剥净', () => {
    const line = '  \x1b[38;2;9;9;9m─ 配置管理 (Config)\x1b[0m';
    const [out] = stripPanelBorders([line]);
    const p = out.replace(/\x1b\[[0-9;]*m/g, '');
    expect(p).toContain('配置管理 (Config)');
    expect(p).not.toContain('─');
  });

  test('纯分隔线行变为空行', () => {
    const [out] = stripPanelBorders(['\x1b[38;2;1;2;3m────────────\x1b[0m']);
    expect(out.replace(/\x1b\[[0-9;]*m/g, '').trim()).toBe('');
  });
});
