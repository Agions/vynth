/**
 * TUI 渲染原语：行宽测量、Unicode 安全的可见宽度、边框、徽章、文本折行、面板。
 * 所有可见宽度都按 Unicode 宽度计算（CJK 字符按 2 个 col 算）。
 * 全部函数纯函数，不写 stdout（除 `renderFrame` 由调用方负责清屏与重绘）。
 */
import ansiEscapes from 'ansi-escapes';
import { bg, fg, reset } from './theme';
import type { Palette } from './theme';

const ESCAPES = ansiEscapes as unknown as {
  cursorTo: (x: number, y?: number) => string;
  eraseDown: string;
  [key: string]: unknown;
};

const ESC = String.fromCharCode(0x1b);
const CSI_RE = new RegExp(`${ESC}\\[[0-9;?]*[a-zA-Z]`, 'g');

/** 一个 box-drawing 字符宽度（CJK 与宽字符仍由 visibleWidth 算） */
export function visibleWidth(str: string): number {
  // 去掉 ANSI 转义后再算 codepoint 宽度
  const plain = stripAnsi(str);
  let w = 0;
  for (const ch of plain) {
    w += charWidth(ch);
  }
  return w;
}

function stripAnsi(str: string): string {
  return str.replace(CSI_RE, '');
}

function charWidth(ch: string): number {
  const cp = ch.codePointAt(0) ?? 0;
  // CJK 区间统一按 2
  if (
    (cp >= 0x1100 && cp <= 0x115f) ||
    (cp >= 0x2e80 && cp <= 0x303e) ||
    (cp >= 0x3041 && cp <= 0x33ff) ||
    (cp >= 0x3400 && cp <= 0x4dbf) ||
    (cp >= 0x4e00 && cp <= 0x9fff) ||
    (cp >= 0xa000 && cp <= 0xa4cf) ||
    (cp >= 0xac00 && cp <= 0xd7a3) ||
    (cp >= 0xf900 && cp <= 0xfaff) ||
    (cp >= 0xfe30 && cp <= 0xfe4f) ||
    (cp >= 0xff00 && cp <= 0xff60) ||
    (cp >= 0xffe0 && cp <= 0xffe6) ||
    (cp >= 0x20000 && cp <= 0x2fffd) ||
    (cp >= 0x30000 && cp <= 0x3fffd)
  ) {
    return 2;
  }
  if (cp === 0x1b) return 0;
  if (cp < 0x20) return 0; // 控制字符不可见
  return 1;
}

/** 按可见宽度把字符串折成多行（ANSI 转义保留，不切到转义中段） */
export function wrapLine(input: string, width: number): string[] {
  if (width <= 0) return [input];
  const lines: string[] = [];
  let buf = '';
  let bufWidth = 0;
  let openCodes: string[] = []; // 已打开但未关闭的 SGR

  for (let i = 0; i < input.length; ) {
    if (input[i] === ESC) {
      // 切走整段 CSI 转义：以 'm' 结尾视为 SGR；其它忽略
      const end = input.indexOf('m', i);
      if (end === -1) {
        buf += input[i];
        i++;
        continue;
      }
      const seq = input.slice(i, end + 1);
      buf += seq;
      if (seq === '\x1b[0m') {
        openCodes = [];
      } else if (seq.length > 3 && seq[seq.length - 1] === 'm') {
        openCodes.push(seq);
      }
      i = end + 1;
      continue;
    }
    const ch = input[i];
    const cw = charWidth(ch);
    if (bufWidth + cw > width) {
      lines.push(buf);
      // 重新打开之前的颜色
      buf = openCodes.join('');
      bufWidth = 0;
    }
    buf += ch;
    bufWidth += cw;
    i++;
  }
  if (buf.length > 0) lines.push(buf);
  return lines.length > 0 ? lines : [''];
}

function padToWidth(input: string, width: number): string {
  const w = visibleWidth(input);
  if (w >= width) return input;
  return input + ' '.repeat(width - w);
}

/** 渲染一个面板：上下左右各 1 格边框，圆角，可选标题 */
export function renderPanel(opts: {
  width: number;
  title?: string;
  titleAlign?: 'left' | 'center' | 'right';
  body: string[];
  borderColor: string;
  titleColor: string;
  accent?: string;
}): string {
  const w = Math.max(8, opts.width);
  const innerW = w - 2;
  const border = `${fg(opts.borderColor)}`;
  const out: string[] = [];

  // 顶
  let top = '╭';
  if (opts.title) {
    const titleStr = ` ${opts.title} `;
    const titleW = visibleWidth(titleStr);
    const left = Math.max(1, Math.floor((innerW - titleW) / 2));
    const right = Math.max(1, innerW - titleW - left);
    top = `${top}${'─'.repeat(left)}${fg(opts.titleColor)}${titleStr}${border}${'─'.repeat(right)}`;
  } else {
    top = `${top}${'─'.repeat(innerW)}`;
  }
  top += '╮';
  out.push(`${border}${top}${reset}`);

  // body
  for (const raw of opts.body) {
    for (const line of wrapLine(raw, innerW - 2)) {
      const padded = padToWidth(line, innerW - 2);
      out.push(`${border}│${reset} ${opts.accent ?? ''}${padded}${reset} ${border}│${reset}`);
    }
  }

  // 底
  out.push(`${border}╰${'─'.repeat(innerW)}╯${reset}`);
  return out.join('\n');
}

/** 渲染一个软标签（用于状态栏：mode / theme / model / 长度） */
export function renderBadge(text: string, color: string, bgHex?: string): string {
  const textStyled = fg(color) + text + reset;
  if (!bgHex) return textStyled;
  return bg(bgHex) + fg(color) + text + reset;
}

/** 渲染一条状态栏（左侧：mode/theme/model；右侧：长度/状态） */
export function renderStatusBar(opts: {
  width: number;
  left: string;
  right: string;
  color: string;
  bgHex: string;
  textColor?: string;
}): string {
  const w = Math.max(20, opts.width);
  const leftW = visibleWidth(opts.left);
  const rightW = visibleWidth(opts.right);
  const pad = Math.max(1, w - leftW - rightW);
  const styledText = (s: string) => fg(opts.textColor ?? opts.color) + s + reset;
  return bg(opts.bgHex) + styledText(opts.left) + ' '.repeat(pad) + styledText(opts.right) + reset;
}

/** 渲染分隔线（标题 + 字符） */
export function renderSection(title: string, color: string, width = 60): string {
  const t = ` ${title} `;
  const tw = visibleWidth(t);
  const left = Math.max(1, Math.floor((width - tw) / 2));
  const right = Math.max(1, width - tw - left);
  return fg(color) + '─'.repeat(left) + t + '─'.repeat(right) + reset;
}

/** 渲染一个 markdown-ish 的轻量段落（支持 `code` 灰底与 *bold*） */
export function renderInline(text: string, palette: Palette): string {
  // 用极简的 *bold* / `code` 处理
  const out: string[] = [];
  let i = 0;
  let bold = false;
  let buf = '';
  const flush = () => {
    if (!buf) return;
    if (bold) out.push(fg(palette.text), buf, reset, fg(palette.subtext));
    else out.push(buf);
    buf = '';
  };
  out.push(fg(palette.subtext));
  while (i < text.length) {
    const ch = text[i];
    if (ch === '*' && text[i + 1] !== ' ' && text[i + 1] !== undefined) {
      flush();
      bold = !bold;
      out.push(bold ? fg(palette.lavender) : fg(palette.subtext));
      i++;
      continue;
    }
    if (ch === '`') {
      const end = text.indexOf('`', i + 1);
      if (end !== -1) {
        flush();
        const code = text.slice(i + 1, end);
        out.push(`${bg(palette.mantle)}${fg(palette.teal)} ${code} ${reset}`, fg(palette.subtext));
        i = end + 1;
        continue;
      }
    }
    buf += ch;
    i++;
  }
  flush();
  out.push(reset);
  return out.join('');
}

/** 渲染一行 role-aware 消息：仅用左侧 1 格色块 + 缩进体，不再显示角色文字。 */
export function renderMessage(opts: {
  role: 'user' | 'assistant' | 'system' | 'tool';
  content: string;
  palette: Palette;
  width: number;
}): string {
  const barColor = (
    {
      user: opts.palette.blue,
      assistant: opts.palette.mauve,
      system: opts.palette.yellow,
      tool: opts.palette.teal
    } as const
  )[opts.role];

  const innerW = Math.max(10, opts.width - 4);
  const lines: string[] = [];

  const body = opts.role === 'tool' ? renderInline(opts.content, opts.palette) : opts.content;
  // 首行带左侧色块 + 缩进对齐；其余行沿用 2 空格缩进
  const wrapped = wrapLine(body, innerW);
  for (let i = 0; i < wrapped.length; i++) {
    if (i === 0) {
      lines.push(`${fg(barColor)}▎${reset}  ${wrapped[i]}${reset}`);
    } else {
      lines.push(`   ${wrapped[i]}${reset}`);
    }
  }
  // 空消息时仍渲染一个色块，保证视觉对齐
  if (wrapped.length === 0) lines.push(`${fg(barColor)}▎${reset}`);
  return lines.join('\n');
}

/**
 * 渲染专业底栏区域（5 行）：
 *   ─ deepseek-v4-pro · vibe · mocha ───────────────────────────── ready ─
 *
 *   ❯ {input}▏
 *
 *    ⏎ send   ⎋ esc   ^c quit   ⇧↑↓ scroll   ⇟ page
 *
 * 设计要点：
 *   - 分隔线带上下文信息（模型 / 模式 / 主题），右侧实时状态
 *   - 空行留白，与聊天区分离
 *   - 输入行用 ❯ 提示符 + 光标块，无重边框
 *   - 键位提示用 kbd 风格小标签
 */
export function renderInputArea(opts: {
  width: number;
  input: string;
  model: string;
  mode: string;
  theme: string;
  status: string;
  statusColor: string;
  palette: Palette;
}): string[] {
  const c = opts.palette;
  const w = opts.width;

  // Line 1: 上下文分隔线
  const leftText = `${opts.model} · ${opts.mode} · ${opts.theme}`;
  const rightText = opts.status;
  const leftVis = visibleWidth(leftText);
  const rightVis = visibleWidth(rightText);
  const dashTotal = Math.max(4, w - leftVis - rightVis - 4);
  const dashLeft = Math.floor(dashTotal / 2);
  const dashRight = dashTotal - dashLeft;
  const sep =
    `${fg(c.overlay0 ?? c.subtext)}${'─'.repeat(2)} ` +
    `${fg(c.subtext)}${leftText}${reset}` +
    ` ${fg(c.overlay0 ?? c.subtext)}${'─'.repeat(dashLeft)}` +
    `${fg(opts.statusColor)}${rightText}` +
    `${fg(c.overlay0 ?? c.subtext)}${'─'.repeat(dashRight)}${reset}`;

  // Line 2: 空白留白
  const blank = '';

  // Line 3: 输入行
  const promptIcon = `${fg(c.mauve)}❯${reset}`;
  const inputText = `${fg(c.text)}${opts.input}${reset}`;
  const cursor = `${fg(c.subtext)}▏${reset}`;
  const inputLine = `  ${promptIcon} ${inputText}${cursor}`;

  // Line 4: 空白留白
  // Line 5: 键位提示
  const kbd = (key: string, label: string): string =>
    `${fg(c.overlay0 ?? c.subtext)} ${key} ${reset}` + `${fg(c.subtext)}${label}${reset}`;

  const hints = [
    kbd('⏎', ' send'),
    kbd('⎋', ' esc'),
    kbd('^c', ' quit'),
    kbd('⇧↑↓', ' scroll'),
    kbd('⇟', ' page')
  ].join('   ');
  const hintLine = `  ${hints}`;

  return [sep, blank, inputLine, blank, hintLine];
}

/** 渲染一个带样式的工具调用块（name + 摘要 + 状态） */
export function renderToolBlock(opts: {
  name: string;
  args: string;
  status: 'running' | 'ok' | 'error';
  output?: string;
  palette: Palette;
  width: number;
}): string {
  const statusColor =
    opts.status === 'running'
      ? opts.palette.yellow
      : opts.status === 'ok'
        ? opts.palette.green
        : opts.palette.red;
  const statusIcon = opts.status === 'running' ? '◐' : opts.status === 'ok' ? '✓' : '✗';
  const innerW = Math.max(10, opts.width - 4);
  const lines: string[] = [];
  lines.push(
    `${fg(statusColor)}${statusIcon} ${fg(opts.palette.lavender)}${opts.name}${reset} ${fg(opts.palette.subtext)}${opts.args}${reset}`
  );
  if (opts.output) {
    const head = renderInline(opts.output, opts.palette);
    for (const ln of wrapLine(head, innerW - 2)) {
      lines.push(`  ${fg(opts.palette.subtext)}│${reset} ${ln}`);
    }
  }
  return lines.join('\n');
}

/** 把光标移到 (0,0) 并清屏下方 */
export function renderFrameStart(width: number): string {
  return `${ESCAPES.cursorTo(0, 0)}${ESCAPES.eraseDown}`;
}

/** 截断消息历史：保留首条欢迎 + 末尾 N 条，使总可见行数不超过 maxLines */
export function clipHistory(messages: string[], maxLines: number): string[] {
  const total = messages.reduce((acc, m) => acc + m.split('\n').length, 0);
  if (total <= maxLines) return messages;
  // 简化：直接保留前 1 + 后 N
  const head = messages.slice(0, 1);
  const tail = messages.slice(-3);
  return [...head, `${'─'.repeat(40)}  earlier turns hidden  ${'─'.repeat(40)}`, ...tail];
}
