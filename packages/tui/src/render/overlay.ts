
import { padToWidth, stripAnsi, truncateVisible, visibleWidth } from '../utils/unicode';

const RESET = '\x1b[0m';

export function withBackground(line: string, bgSeq: string): string {
  return `${bgSeq}${line.split(RESET).join(`${RESET}${bgSeq}`)}${RESET}`;
}

export interface OverlayOptions {
  left: number;
  width: number;
  anchorBottom: number;
  minRow?: number;
  bgSeq?: string;
}

export function composeOverlay(frame: string[], overlay: string[], opts: OverlayOptions): string[] {
  const { left, width, anchorBottom, minRow = 3, bgSeq } = opts;
  if (overlay.length === 0) return frame;

  const available = Math.max(0, anchorBottom - minRow);
  const visible = overlay.length > available ? overlay.slice(overlay.length - available) : overlay;
  const startRow = anchorBottom - visible.length;

  for (let i = 0; i < visible.length; i++) {
    const row = startRow + i;
    if (row < 0 || row >= frame.length) continue;

    const under = frame[row] ?? '';
    const leftPart = padToWidth(truncateVisible(under, left), left);

    let content = truncateVisible(visible[i] ?? '', width);
    content = padToWidth(content, width);
    if (bgSeq) content = withBackground(content, bgSeq);

    frame[row] = `${leftPart}${content}${RESET}`;
  }
  return frame;
}

export function stripPanelBorders(lines: string[]): string[] {
  return lines.map((line) => {
    const plain = stripAnsi(line);
    if (/[╭╰┌└]/.test(plain)) {
      let out = line.replace(/[╭╮╰╯┌┐└┘]/g, '');
      out = stripRules(out);
      return stripAnsi(out).trim() === '' ? '' : ` ${out.trimEnd()}`;
    }
    if (/^\s*│/.test(plain) && /│\s*$/.test(plain.trimEnd())) {
      const first = line.indexOf('│');
      const last = line.lastIndexOf('│');
      if (first !== -1 && last > first) {
        const inner = `${line.slice(0, first)} ${line.slice(first + 1, last)}${line.slice(last + 1)}`;
        return stripRules(inner);
      }
    }
    if (plain.includes('─')) {
      const out = stripRules(line);
      return stripAnsi(out).trim() === '' ? '' : out;
    }
    return line;
  });
}

function stripRules(line: string): string {
  return line.replace(/ *─+ */g, (m) => (m.startsWith(' ') || m.endsWith(' ') ? ' ' : ''));
}

export function overlayPanelWidth(overlay: string[], padding: number, maxWidth: number): number {
  let max = 0;
  for (const line of overlay) {
    const wd = visibleWidth(line);
    if (wd > max) max = wd;
  }
  return Math.min(max + padding, maxWidth);
}
