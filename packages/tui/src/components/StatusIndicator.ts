import type { TuiState } from '../state/TuiState';
import { renderBadge } from '../utils/text';

export interface StatusIndicatorProps {
  state: TuiState;
}

export function StatusIndicator(props: StatusIndicatorProps): string {
  const { state } = props;
  const c = state.palette;

  const statusText =
    state.liveStatus === 'streaming'
      ? 'streaming'
      : state.liveStatus === 'tool'
        ? `tool: ${state.currentTool ?? '…'}`
        : state.liveStatus === 'thinking'
          ? 'thinking'
          : 'ready';
  const statusColor =
    state.liveStatus === 'streaming'
      ? c.yellow
      : state.liveStatus === 'tool'
        ? c.lavender
        : state.liveStatus === 'thinking'
          ? c.blue
          : c.green;

  return renderBadge(` ${statusText} `, c.crust ?? c.base, statusColor);
}
