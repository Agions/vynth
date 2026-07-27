/**
 * 历史滚动缓冲：把任意字符串（多行）追加进一个固定上限的环形行缓冲，
 * 支持「虚拟滚动」：只看视口最后 N 行，并能完整取整段以做区域重绘。
 */

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

  /** 一次性替换（用于初次绘制 / 视口大小变化） */
  reset(multiLine: string): void {
    this.lines = multiLine.split('\n');
    while (this.lines.length > this.capacity) {
      this.lines.shift();
    }
  }

  /** 末尾最后 N 行（用于增量绘制） */
  tailLines(n: number): string[] {
    return this.lines.slice(-n);
  }

  /** 当前总行数 */
  size(): number {
    return this.lines.length;
  }

  /** 所有行（用于初次重绘或滚动条统计） */
  all(): readonly string[] {
    return this.lines;
  }

  /** 给定视口高度，返回应展示的最后 N 行（不做截断，由渲染层负责） */
  visibleForViewport(height: number): string[] {
    if (height <= 0) return [];
    return this.lines.slice(-height);
  }

  /** 模拟向上滚动 n 行（从末尾向上看）。0 表示贴底（最新内容）。 */
  scrollUp(delta: number): string[] {
    const end = this.lines.length - delta;
    if (end <= 0) return [...this.lines];
    return this.lines.slice(0, end);
  }

  /** 视口顶部偏移量（0=贴底） */
  offsetFromBottom(offset: number): number {
    return Math.max(0, Math.min(this.lines.length, offset));
  }
}
