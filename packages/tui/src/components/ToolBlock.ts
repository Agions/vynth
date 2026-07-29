import type { TuiState } from '../state/TuiState';
import { renderToolBlock } from '../utils/text';

export interface ToolBlockProps {
  state: TuiState;
  name: string;
  args: string;
  status: 'queued' | 'running' | 'ok' | 'error' | 'blocked';
  output?: string;
  collapsed?: boolean;
  selected?: boolean;
  hint?: string;
}

export function ToolBlock(props: ToolBlockProps): string {
  const { state, name, args, status, output, collapsed, selected, hint } = props;
  const innerW = Math.max(20, state.cols - 4);
  return renderToolBlock({
    name,
    args,
    status,
    output,
    palette: state.palette,
    width: innerW,
    collapsed,
    selected,
    hint
  });
}
