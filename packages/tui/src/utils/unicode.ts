export function stripAnsi(str: string): string {
  const ESC = '\x1b';
  const CSI_RE = new RegExp(`${ESC}\\[[0-9;?]*[a-zA-Z]`, 'g');
  return str.replace(CSI_RE, '');
}

export function charWidth(ch: string): number {
  const cp = ch.codePointAt(0) ?? 0;
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
  if (cp < 0x20) return 0;
  return 1;
}

export function visibleWidth(str: string): number {
  const plain = stripAnsi(str);
  let w = 0;
  for (const ch of plain) {
    w += charWidth(ch);
  }
  return w;
}

export function wrapLine(input: string, width: number): string[] {
  if (width <= 0) return [input];
  const lines: string[] = [];
  let buf = '';
  let bufWidth = 0;
  let openCodes: string[] = [];

  for (let i = 0; i < input.length; ) {
    if (input[i] === '\x1b') {
      const match = /^\x1b\[[0-9;?]*[a-zA-Z]/.exec(input.slice(i));
      if (match) {
        const seq = match[0];
        buf += seq;
        if (seq === '\x1b[0m') {
          openCodes = [];
        } else if (seq.endsWith('m')) {
          openCodes.push(seq);
        }
        i += seq.length;
        continue;
      }
      i++;
      continue;
    }
    const ch = input[i];
    const cw = charWidth(ch);
    if (bufWidth + cw > width) {
      lines.push(buf);
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

export function truncateVisible(input: string, maxCols: number): string {
  if (maxCols <= 0) return '';
  let vis = 0;
  let out = '';
  let i = 0;
  while (i < input.length && vis < maxCols) {
    if (input[i] === '\x1b') {
      const match = /^\x1b\[[0-9;?]*[a-zA-Z]/.exec(input.slice(i));
      if (match) {
        const seq = match[0];
        out += seq;
        i += seq.length;
        continue;
      }
      i++;
      continue;
    }
    const cp = input.codePointAt(i) ?? 0;
    const w = isWideChar(cp) ? 2 : 1;
    if (vis + w > maxCols) break;
    out += input[i];
    vis += w;
    i++;
  }
  return out;
}

function isWideChar(cp: number): boolean {
  return (
    (cp >= 0x1100 && cp <= 0x115f) ||
    (cp >= 0x2e80 && cp <= 0x303e) ||
    (cp >= 0x3041 && cp <= 0x33ff) ||
    (cp >= 0x3400 && cp <= 0x4dbf) ||
    (cp >= 0x4e00 && cp <= 0x9fff) ||
    (cp >= 0xac00 && cp <= 0xd7a3) ||
    (cp >= 0xf900 && cp <= 0xfaff) ||
    (cp >= 0xff00 && cp <= 0xff60) ||
    (cp >= 0xffe0 && cp <= 0xffe6)
  );
}

export function padToWidth(input: string, width: number): string {
  const w = visibleWidth(input);
  if (w >= width) return input;
  return input + ' '.repeat(width - w);
}
