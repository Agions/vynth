const ESC = '\x1b';
const CLEAR_SCREEN = `${ESC}[2J${ESC}[H`;
const ERASE_LINE_REST = `${ESC}[K`;
const WRAP_OFF = `${ESC}[?7l`;
const WRAP_ON = `${ESC}[?7h`;

export class FrameCache {
  private prev: string[] = [];
  private prevCols = -1;
  private prevRows = -1;

  flush(lines: string[], cols: number, rows: number): string {
    const full = this.prevCols !== cols || this.prevRows !== rows || this.prev.length === 0;
    const parts: string[] = [WRAP_OFF];

    if (full) {
      parts.push(CLEAR_SCREEN);
      for (let i = 0; i < lines.length; i++) {
        parts.push(`${ESC}[${i + 1};1H${lines[i]}${ERASE_LINE_REST}`);
      }
    } else {
      const maxLen = Math.max(lines.length, this.prev.length);
      for (let i = 0; i < maxLen; i++) {
        const next = lines[i];
        const old = this.prev[i];
        if (next === old) continue;
        if (next === undefined) {
          parts.push(`${ESC}[${i + 1};1H${ESC}[2K`);
        } else {
          parts.push(`${ESC}[${i + 1};1H${next}${ERASE_LINE_REST}`);
        }
      }
    }

    parts.push(WRAP_ON);
    this.prev = lines.slice();
    this.prevCols = cols;
    this.prevRows = rows;

    return parts.length === 2 ? '' : parts.join('');
  }

  invalidate(): void {
    this.prev = [];
    this.prevCols = -1;
    this.prevRows = -1;
  }

  get lineCount(): number {
    return this.prev.length;
  }
}
