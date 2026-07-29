import type { TuiState } from '../state/TuiState';
import { renderInputPanel } from '../utils/text';

export interface InputPanelProps {
  state: TuiState;
}

export function InputPanel(props: InputPanelProps): string {
  const { state } = props;
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
      ? state.palette.yellow
      : state.liveStatus === 'tool'
        ? state.palette.lavender
        : state.liveStatus === 'thinking'
          ? state.palette.blue
          : state.palette.green;

  return renderInputPanel({
    width: state.cols,
    input: state.input,
    model: state.theme,
    mode: state.liveStatus === 'thinking' ? 'plan' : 'vibe',
    theme: state.theme,
    status: statusText,
    statusColor,
    palette: state.palette
  });
}
