import type { ViewportLayout } from '../state/TuiState';
import { type Breakpoint, computeLayout, getBreakpoint, getBreakpointConfig } from './Viewport';

export interface LayoutStrategy {
  breakpoint: Breakpoint;
  layout: ViewportLayout;
  config: ReturnType<typeof getBreakpointConfig>;
}

export { getBreakpoint, getBreakpointConfig };
export type { Breakpoint };

export function computeLayoutStrategy(opts: {
  cols: number;
  rows: number;
  topLines: number;
  botLines: number;
}): LayoutStrategy {
  const breakpoint = getBreakpoint(opts.cols);
  const layout = computeLayout(opts);
  const config = getBreakpointConfig(breakpoint);

  return {
    breakpoint,
    layout,
    config
  };
}
