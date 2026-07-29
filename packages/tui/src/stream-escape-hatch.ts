const ESC = String.fromCharCode(0x1b);
const clearLineSeq = `${ESC}[G${ESC}[2K`;

export class StreamArea {
  private lastLen = 0;
  constructor(private readonly write: (s: string) => void = (s) => process.stdout.write(s)) {}

  update(text: string): void {
    if (this.lastLen > 0) this.write(clearLineSeq);
    this.write(text);
    this.lastLen = text.length;
  }

  clear(): void {
    if (this.lastLen > 0) this.write(clearLineSeq);
    this.lastLen = 0;
  }
}
