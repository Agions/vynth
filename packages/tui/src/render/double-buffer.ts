import type { ScreenBuffer } from '../kernel/buffer';
import { clearBuffer, computeDiff, createBuffer, swapBuffers } from '../kernel/buffer';
import { ColorRegistry } from '../render/colors';
import { renderDiff } from '../render/diff';
import {
  clearBufferWithBg,
  renderDividerToBuffer,
  renderLineToBuffer,
  renderTextToBuffer
} from '../render/primitives';

export class DoubleBufferedRenderer {
  private front: ScreenBuffer;
  private back: ScreenBuffer;
  private cols: number;
  private rows: number;
  readonly colors: ColorRegistry;

  constructor(cols: number, rows: number, opts: { truecolor?: boolean } = {}) {
    this.cols = cols;
    this.rows = rows;
    this.front = createBuffer(cols, rows);
    this.back = createBuffer(cols, rows);
    this.colors = new ColorRegistry(opts);
  }

  color(hex: string): number {
    return this.colors.index(hex);
  }

  resize(cols: number, rows: number): void {
    this.cols = cols;
    this.rows = rows;
    this.front = createBuffer(cols, rows);
    this.back = createBuffer(cols, rows);
  }

  clear(fgIdx = 0, bgIdx = 0): void {
    clearBufferWithBg(this.front, bgIdx);
  }

  renderText(x: number, y: number, text: string, fgIdx = 0, bgIdx = 0): void {
    renderTextToBuffer(this.front, x, y, text, fgIdx, bgIdx);
  }

  renderLine(y: number, text: string, fgIdx = 0, bgIdx = 0): void {
    renderLineToBuffer(this.front, y, text, fgIdx, bgIdx);
  }

  renderDivider(y: number, char = '─', fgIdx = 0): void {
    renderDividerToBuffer(this.front, y, char, fgIdx);
  }

  flush(): string {
    const diff = computeDiff(this.front, this.back);
    const chunks = renderDiff(this.front, this.back, { colors: this.colors });
    const output = chunks.map((c) => c.output).join('');
    swapBuffers(this.front, this.back);
    return output;
  }

  get size(): { cols: number; rows: number } {
    return { cols: this.cols, rows: this.rows };
  }
}
