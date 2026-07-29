export class Scrollback {
  private lines: string[] = [];
  constructor(private readonly capacity: number) {}

  push(multiLine: string): void {
    const lines = multiLine.split('\n');
    for (const line of lines) {
      this.lines.push(line);
    }
    if (this.lines.length > this.capacity) {
      this.lines.splice(0, this.lines.length - this.capacity);
    }
  }

  reset(multiLine: string): void {
    this.lines = multiLine.split('\n');
    while (this.lines.length > this.capacity) {
      this.lines.shift();
    }
  }

  tailLines(n: number): string[] {
    return this.lines.slice(-n);
  }

  size(): number {
    return this.lines.length;
  }

  all(): readonly string[] {
    return this.lines;
  }

  visibleForViewport(height: number): string[] {
    if (height <= 0) return [];
    return this.lines.slice(-height);
  }

  scrollUp(delta: number): string[] {
    const end = this.lines.length - delta;
    if (end <= 0) return [...this.lines];
    return this.lines.slice(0, end);
  }

  offsetFromBottom(offset: number): number {
    return Math.max(0, Math.min(this.lines.length, offset));
  }
}
