import ansiEscapes from 'ansi-escapes';

/**
 * 流式逃生舱：在 TUI 之外（或无头模式）用原始 ANSI 直写，
 * 绕过 ink 的 React reconciliation，避免每个 token 触发全树重渲染。
 */
export class StreamArea {
  private lastLen = 0;
  constructor(private readonly write: (s: string) => void = (s) => process.stdout.write(s)) {}

  update(text: string): void {
    if (this.lastLen > 0) this.write(ansiEscapes.cursorLeft + ansiEscapes.clearLine);
    this.write(text);
    this.lastLen = text.length;
  }

  clear(): void {
    if (this.lastLen > 0) this.write(ansiEscapes.cursorLeft + ansiEscapes.clearLine);
    this.lastLen = 0;
  }
}
