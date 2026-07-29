import { highlightCode } from '../render/syntax';
import { fg, reset } from '../theme';
import type { Palette } from '../theme';
import { ansiBackground, hexToRgb } from './color';
import { errorHintFor, parseVcCode } from './error-hints';
import { padToWidth, visibleWidth, wrapLine } from './unicode';

// ─── Spinner ──────────────────────────────────────────────────────────────────

const SPINNER_FRAMES = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
const CURSOR_BLINK = ['▏', ' '];

// ─── Kbd / Badge ──────────────────────────────────────────────────────────────

export function renderKbd(text: string, palette: Palette): string {
  const c = palette;
  return `${ansiBackground(c.surface0 ?? c.mantle)}${fg(c.text)} ${text} ${reset}`;
}

export function renderBadge(text: string, color: string, bgHex?: string): string {
  if (!bgHex) return fg(color) + text + reset;
  return ansiBackground(bgHex) + fg(color) + text + reset;
}

// ─── Layout ───────────────────────────────────────────────────────────────────

export function renderDivider(opts: {
  width: number;
  label?: string;
  color?: string;
  char?: string;
}): string {
  const w = Math.max(10, opts.width);
  const color = opts.color ?? '#585b70';
  const ch = opts.char ?? '─';
  if (opts.label) {
    const t = ` ${opts.label} `;
    const tw = visibleWidth(t);
    const left = Math.max(1, Math.floor((w - tw) / 2));
    const right = Math.max(1, w - tw - left);
    return `${fg(color)}${ch.repeat(left)}${t}${ch.repeat(right)}${reset}`;
  }
  return `${fg(color)}${ch.repeat(w)}${reset}`;
}

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
  const border = fg(opts.borderColor);
  const out: string[] = [];

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

  for (const raw of opts.body) {
    for (const line of wrapLine(raw, innerW - 2)) {
      const padded = padToWidth(line, innerW - 2);
      out.push(`${border}│${reset} ${opts.accent ?? ''}${padded}${reset} ${border}│${reset}`);
    }
  }

  out.push(`${border}╰${'─'.repeat(innerW)}╯${reset}`);
  return out.join('\n');
}

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
  const styled = (s: string) => fg(opts.textColor ?? opts.color) + s + reset;
  return (
    ansiBackground(opts.bgHex) + styled(opts.left) + ' '.repeat(pad) + styled(opts.right) + reset
  );
}

export function renderSection(title: string, color: string, width = 60): string {
  const t = ` ${title} `;
  const tw = visibleWidth(t);
  const left = Math.max(1, Math.floor((width - tw) / 2));
  const right = Math.max(1, width - tw - left);
  return fg(color) + '─'.repeat(left) + t + '─'.repeat(right) + reset;
}

// ─── Inline Markdown ──────────────────────────────────────────────────────────

export function renderMarkdownInline(text: string, p: Palette): string {
  return text
    .replace(
      /`([^`]+)`/g,
      `${ansiBackground(p.surface0 ?? p.mantle)}${fg(p.teal)}\x1b[1m $1 ${reset}`
    )
    .replace(/\*\*([^*]+)\*\*/g, `\x1b[1m${fg(p.text)}$1${reset}`)
    .replace(/\*([^*]+)\*/g, `\x1b[3m${fg(p.subtext)}$1${reset}`)
    .replace(/\[([^\]]+)\]\(([^)]+)\)/g, `${fg(p.blue)}\x1b[4m$1${reset}`);
}

export function renderInline(text: string, palette: Palette): string {
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
        out.push(
          `${ansiBackground(palette.mantle)}${fg(palette.teal)} ${code} ${reset}`,
          fg(palette.subtext)
        );
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

// ─── Markdown Block ───────────────────────────────────────────────────────────

export function renderMarkdownContent(content: string, p: Palette, width = 80): string {
  const lines = content.split('\n');
  const result: string[] = [];
  let inCodeBlock = false;
  let codeBuffer: string[] = [];
  let codeLang = '';
  let codeFilename = '';

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];

    if (line.trim().startsWith('```')) {
      if (!inCodeBlock) {
        inCodeBlock = true;
        const meta = line.trim().slice(3).trim();
        const colonIdx = meta.indexOf(':');
        codeLang = colonIdx !== -1 ? meta.slice(0, colonIdx) : meta;
        codeFilename = colonIdx !== -1 ? meta.slice(colonIdx + 1) : '';
        codeBuffer = [];
      } else {
        inCodeBlock = false;
        result.push(...renderCodeBlock(codeBuffer, codeLang, codeFilename, p, width));
        codeBuffer = [];
        codeLang = '';
        codeFilename = '';
      }
      continue;
    }

    if (inCodeBlock) {
      codeBuffer.push(line);
      continue;
    }

    // Headers
    const headerMatch = line.match(/^(#{1,6})\s+(.*)$/);
    if (headerMatch) {
      const level = headerMatch[1].length;
      const text = headerMatch[2];
      const icon = level === 1 ? '◈ ' : level === 2 ? '◆ ' : level === 3 ? '◇ ' : '▸ ';
      const col = level <= 2 ? p.mauve : level === 3 ? p.lavender : p.subtext;
      result.push(`${fg(col)}\x1b[1m${icon}${renderMarkdownInline(text, p)}${reset}`);
      continue;
    }

    // Blockquote
    const quoteMatch = line.match(/^>\s+(.*)$/);
    if (quoteMatch) {
      result.push(
        `${fg(p.overlay0 ?? p.subtext)}▎${reset}${fg(p.subtext)} ${renderMarkdownInline(quoteMatch[1], p)}${reset}`
      );
      continue;
    }

    // Unordered list
    const unorderMatch = line.match(/^[\*\-]\s+(.*)$/);
    if (unorderMatch) {
      result.push(`  ${fg(p.mauve)}·${reset} ${renderMarkdownInline(unorderMatch[1], p)}`);
      continue;
    }

    // Ordered list
    const orderMatch = line.match(/^(\d+)\.\s+(.*)$/);
    if (orderMatch) {
      result.push(
        `  ${fg(p.mauve)}${orderMatch[1]}.${reset} ${renderMarkdownInline(orderMatch[2], p)}`
      );
      continue;
    }

    // HR
    if (/^[\-\*]{3,}$/.test(line.trim())) {
      result.push(`${fg(p.overlay0 ?? p.subtext)}${'─'.repeat(Math.min(width, 60))}${reset}`);
      continue;
    }

    result.push(renderMarkdownInline(line, p));
  }

  // Handle unclosed code blocks
  if (inCodeBlock && codeBuffer.length > 0) {
    result.push(...renderCodeBlock(codeBuffer, codeLang, codeFilename, p, width));
  }

  return result.join('\n');
}

function renderCodeBlock(
  lines: string[],
  lang: string,
  filename: string,
  p: Palette,
  width: number
): string[] {
  const result: string[] = [];
  const blockBg = p.surface0 ?? p.mantle;
  const blockBgAnsi = ansiBackground(blockBg);
  const borderCol = fg(p.overlay0 ?? p.subtext);
  const lineNumW = lines.length >= 100 ? 4 : 3;
  const innerContentW = Math.max(10, width - lineNumW - 4);

  const langBadge = lang
    ? `${ansiBackground(p.mauve)}${fg(p.crust ?? p.base)}\x1b[1m ${lang} ${reset}`
    : '';
  const fileBadge = filename ? `${fg(p.subtext)} ${filename}${reset}` : '';
  const badgeStr = [langBadge, fileBadge].filter(Boolean).join('');
  const badgeVW = visibleWidth(badgeStr);
  const topDashW = Math.max(2, width - badgeVW - 2);
  result.push(`${borderCol}╭${reset}${badgeStr}${borderCol}${'─'.repeat(topDashW)}╮${reset}`);

  const highlighted = highlightCode(lines.join('\n'), lang, p);
  const hlLines = highlighted.split('\n');

  for (let idx = 0; idx < hlLines.length; idx++) {
    const lineNum = String(idx + 1).padStart(lineNumW);
    const lineNumStr = `${fg(p.overlay0 ?? p.subtext)}${lineNum}${reset}`;
    const pipe = `${fg(p.overlay0 ?? p.subtext)}│${reset}`;
    const code = hlLines[idx];
    const row = `${blockBgAnsi}${lineNumStr} ${pipe} ${code}${reset}`;
    result.push(`${borderCol}│${reset}${row}${borderCol}│${reset}`);
  }

  const lineCountStr = `${fg(p.subtext)} ${lines.length} line${lines.length !== 1 ? 's' : ''} ${reset}`;
  const lcVW = visibleWidth(lineCountStr);
  const botDashW = Math.max(2, width - lcVW - 2);
  result.push(`${borderCol}╰${reset}${lineCountStr}${borderCol}${'─'.repeat(botDashW)}╯${reset}`);

  return result;
}

// ─── Message Card ─────────────────────────────────────────────────────────────

function relativeTime(ts: number): string {
  const diff = Math.floor((Date.now() - ts) / 1000);
  if (diff < 5) return 'just now';
  if (diff < 60) return `${diff}s ago`;
  if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
  return `${Math.floor(diff / 3600)}h ago`;
}

export function renderMessage(opts: {
  role: 'user' | 'assistant' | 'system' | 'tool';
  content: string;
  palette: Palette;
  width: number;
  background?: boolean;
  timestamp?: number;
}): string {
  if (opts.role === 'system') {
    return opts.content;
  }

  const p = opts.palette;
  const innerW = Math.max(10, opts.width - 4);
  const result: string[] = [];

  if (opts.role === 'user') {
    const contentLines = opts.content.split('\n');
    for (let idx = 0; idx < contentLines.length; idx++) {
      const line = contentLines[idx];
      const prefix = idx === 0 ? `  ${fg(p.mauve)}\x1b[1m❯${reset} ` : '    ';
      const wrapped = wrapLine(line, innerW);
      for (const wl of wrapped) {
        result.push(`${prefix}${fg(p.text)}\x1b[1m${wl}${reset}`);
      }
    }
    result.push('');
  } else if (opts.role === 'assistant') {
    const formatted = renderMarkdownContent(opts.content, p, innerW);
    for (const rawLine of formatted.split('\n')) {
      const wrapped = wrapLine(rawLine, innerW);
      for (const wl of wrapped) {
        result.push(`  ${wl}${reset}`);
      }
    }
    result.push('');
  } else {
    // tool role
    const formatted = renderInline(opts.content, p);
    result.push(`  ${fg(p.lavender)}⚙${reset} ${formatted}`);
  }

  return result.join('\n');
}

// ─── Tool Block ───────────────────────────────────────────────────────────────

export function renderToolBlock(opts: {
  name: string;
  args: string;
  status: 'queued' | 'running' | 'ok' | 'error' | 'blocked';
  output?: string;
  palette: Palette;
  width: number;
  collapsed?: boolean;
  selected?: boolean;
  spinnerFrame?: number;
  durationMs?: number;
  hint?: string;
}): string {
  const p = opts.palette;
  const innerW = Math.max(20, opts.width - 2);
  const borderCol = fg(
    opts.selected
      ? p.yellow
      : opts.status === 'running'
        ? p.yellow
        : opts.status === 'ok'
          ? p.green
          : opts.status === 'blocked'
            ? p.yellow
            : p.red
  );

  const frame = SPINNER_FRAMES[(opts.spinnerFrame || 0) % SPINNER_FRAMES.length];
  const statusIcon =
    opts.status === 'queued'
      ? `${fg(p.subtext)}⧖${reset}`
      : opts.status === 'running'
        ? `${fg(p.yellow)}${frame}${reset}`
        : opts.status === 'ok'
          ? `${fg(p.green)}✔${reset}`
          : opts.status === 'blocked'
            ? `${fg(p.yellow)}⚠${reset}`
            : `${fg(p.red)}✖${reset}`;

  const nameStr = `${fg(opts.selected ? p.yellow : p.lavender)}\x1b[1m${opts.name}${reset}`;
  const foldIcon = opts.collapsed ? `${fg(p.subtext)}▸${reset}` : `${fg(p.subtext)}▾${reset}`;
  const selectMark = opts.selected ? ` ${fg(p.yellow)}◀ selected${reset}` : '';
  const durationStr =
    opts.durationMs !== undefined && opts.status !== 'running'
      ? ` ${fg(p.subtext)}${opts.durationMs < 1000 ? `${opts.durationMs}ms` : `${(opts.durationMs / 1000).toFixed(1)}s`}${reset}`
      : '';

  const titleContent = ` ${statusIcon} ${nameStr} ${foldIcon}${durationStr}${selectMark} `;
  const titleVW = visibleWidth(titleContent);
  const topDashW = Math.max(2, innerW - titleVW - 2);
  const topBorder = `${borderCol}╭─${titleContent}${borderCol}${'─'.repeat(topDashW)}╮${reset}`;

  const lines: string[] = [topBorder];

  if (opts.collapsed) {
    let preview = '';
    if (opts.output) {
      preview = opts.output.replace(/\n/g, ' ').slice(0, innerW - 28);
    }
    const collapsedRow = `  ${fg(p.subtext)}${preview ? `…${preview}` : '(press ↵ to expand)'}${reset}`;
    lines.push(`${borderCol}│${reset}${padToWidth(collapsedRow, innerW)}${borderCol}│${reset}`);
  } else {
    if (opts.args && opts.args !== '{}') {
      const argsStr = `  ${fg(p.subtext)}args ${reset}${fg(p.text)}${opts.args}${reset}`;
      lines.push(`${borderCol}│${reset}${padToWidth(argsStr, innerW)}${borderCol}│${reset}`);
    }
    if (opts.output) {
      lines.push(`${borderCol}│${reset}${padToWidth('', innerW)}${borderCol}│${reset}`);
      const outputLines = opts.output.split('\n').slice(0, 20);
      for (const ol of outputLines) {
        const wrapped = wrapLine(ol, innerW - 4);
        for (const wl of wrapped) {
          const rowStr = `  ${fg(p.text)}${wl}${reset}`;
          lines.push(`${borderCol}│${reset}${padToWidth(rowStr, innerW)}${borderCol}│${reset}`);
        }
      }
      if (opts.output.split('\n').length > 20) {
        const moreStr = `  ${fg(p.subtext)}… ${opts.output.split('\n').length - 20} more lines (press ↵ to collapse)${reset}`;
        lines.push(`${borderCol}│${reset}${padToWidth(moreStr, innerW)}${borderCol}│${reset}`);
      }
    }
  }

  if (opts.status === 'error') {
    const code = parseVcCode(opts.output ?? '');
    const hint = opts.hint ?? (code ? errorHintFor(code) : undefined);
    if (code) {
      const codeLine = `  ${fg(p.red)}\x1b[1m${code}${reset} ${fg(p.subtext)}诊断码${reset}`;
      lines.push(`${borderCol}│${reset}${padToWidth(codeLine, innerW)}${borderCol}│${reset}`);
    }
    if (hint) {
      const hintLine = `  ${fg(p.yellow)}→ ${hint}${reset}`;
      lines.push(`${borderCol}│${reset}${padToWidth(hintLine, innerW)}${borderCol}│${reset}`);
    }
  }

  const footerStr = opts.selected
    ? ` ${fg(p.yellow)}↵${fg(p.subtext)} toggle   ${fg(p.yellow)}Tab${fg(p.subtext)} next   ${fg(p.yellow)}Esc${fg(p.subtext)} deselect `
    : ` ${fg(p.subtext)}↵ toggle · Tab next${reset} `;
  const footerVW = visibleWidth(footerStr);
  const botDashW = Math.max(2, innerW - footerVW - 2);
  const botBorder = `${borderCol}╰─${footerStr}${borderCol}${'─'.repeat(botDashW)}╯${reset}`;
  lines.push(botBorder);

  return lines.join('\n');
}

// ─── Input Panel ──────────────────────────────────────────────────────────────

export function renderInputPanel(opts: {
  width: number;
  input: string;
  model: string;
  mode: string;
  theme: string;
  status: string;
  statusColor: string;
  palette: Palette;
  multiline?: boolean;
  liveStatus?: string;
  spinnerFrame?: number;
  currentTool?: string | null;
}): string {
  const p = opts.palette;
  const w = Math.max(20, opts.width);

  const frame = SPINNER_FRAMES[(opts.spinnerFrame || 0) % SPINNER_FRAMES.length];
  const cursorChar = CURSOR_BLINK[Math.floor((opts.spinnerFrame || 0) / 5) % 2];

  let statusBadge = '';
  if (opts.liveStatus === 'thinking') {
    statusBadge = `${fg(p.lavender)}\x1b[1m${frame} thinking…${reset}`;
  } else if (opts.liveStatus === 'streaming') {
    statusBadge = `${fg(p.teal)}\x1b[1m${frame} streaming…${reset}`;
  } else if (opts.liveStatus === 'tool') {
    statusBadge = `${fg(p.yellow)}\x1b[1m${frame} ${opts.currentTool || 'tool'}…${reset}`;
  }

  const promptIcon = `${fg(p.mauve)}\x1b[1m❯${reset}`;
  const lines: string[] = [];

  if (opts.multiline && opts.input.includes('\n')) {
    const rawLines = opts.input.split('\n');
    for (let idx = 0; idx < rawLines.length; idx++) {
      const lineNum = `${fg(p.subtext)}${String(idx + 1).padStart(3)}${fg(p.overlay0 ?? p.subtext)}│${reset}`;
      const lineText = `${fg(p.text)}${rawLines[idx]}${reset}`;
      lines.push(`  ${lineNum} ${lineText}`);
    }
  } else {
    const charCount = opts.input.length;
    let content: string;
    if (charCount === 0) {
      const placeholder = `${fg(p.subtext)}Ask Vynth… type / for commands, @ for files${reset}`;
      content = `  ${promptIcon} ${placeholder}`;
    } else {
      const inputText = `${fg(p.text)}${opts.input}${reset}`;
      const cursor =
        opts.liveStatus === 'idle' || !opts.liveStatus ? `${fg(p.mauve)}${cursorChar}${reset}` : '';
      content = `  ${promptIcon} ${inputText}${cursor}`;
    }
    lines.push(content);
  }

  return lines.join('\n');
}

export function renderHintsBar(opts: {
  width: number;
  palette: Palette;
  liveStatus: string;
}): string {
  const c = opts.palette;
  const w = Math.max(20, opts.width);
  const bgHex = c.mantle ?? c.base;
  const hints: Array<[string, string]> = [];

  if (opts.liveStatus === 'streaming' || opts.liveStatus === 'thinking') {
    hints.push(['Esc×2', 'interrupt'], ['/', 'slash'], ['?', 'help']);
  } else if (opts.liveStatus === 'tool') {
    hints.push(['↵', 'toggle-fold'], ['Tab', 'next-tool'], ['Esc', 'deselect']);
  } else {
    hints.push(
      ['⏎', 'send'],
      ['⇧↵', 'newline'],
      ['/', 'slash'],
      ['@', 'file'],
      ['^B', '侧栏'],
      ['^T', '切换'],
      ['^O', 'split'],
      ['?', 'help'],
      ['↑↓', 'scroll'],
      ['^P^N', 'history']
    );
  }

  const parts = hints.map(
    ([key, label]) =>
      `${ansiBackground(c.surface0 ?? c.mantle)}${fg(c.text)} ${key} ${reset}${fg(c.subtext)} ${label}${reset}`
  );
  const hintStr = `  ${parts.join('   ')}  `;
  const hintVW = visibleWidth(hintStr);
  const pad = Math.max(0, w - hintVW);

  return `${ansiBackground(bgHex)}${fg(c.subtext)}${hintStr}${' '.repeat(pad)}${reset}`;
}
