import type { ViewportLayout } from '../state/TuiState';

const ESC = '\x1b';

export interface LayoutOptions {
  cols: number;
  rows: number;
  topLines: number;
  botLines: number;
}

export function computeLayout(opts: LayoutOptions): ViewportLayout {
  const { cols, rows, topLines, botLines } = opts;
  const safeRows = Math.max(10, rows);
  const topEnd = Math.min(safeRows - 6, Math.max(1, topLines));
  const botStart = Math.max(topEnd + 6, safeRows - Math.max(3, botLines) + 1);
  const midStart = topEnd + 1;
  const midEnd = botStart - 1;

  return {
    topStart: 1,
    topEnd,
    midStart,
    midEnd,
    botStart,
    botEnd: safeRows
  };
}

export type Breakpoint = 'narrow' | 'medium' | 'wide';

export function getBreakpoint(cols: number): Breakpoint {
  if (cols < 80) return 'narrow';
  if (cols <= 120) return 'medium';
  return 'wide';
}

export function getBreakpointConfig(breakpoint: Breakpoint) {
  switch (breakpoint) {
    case 'narrow':
      return {
        showScrollbar: false,
        compactTopBar: true,
        showFullHints: false,
        inputPanelBorder: false
      };
    case 'medium':
      return {
        showScrollbar: false,
        compactTopBar: false,
        showFullHints: true,
        inputPanelBorder: true
      };
    case 'wide':
      return {
        showScrollbar: true,
        compactTopBar: false,
        showFullHints: true,
        inputPanelBorder: true
      };
  }
}
