import type { TuiState } from '../state/TuiState';
import type { Palette } from '../theme';
import { fg, reset } from '../theme';
import { hexToRgb } from '../utils/color';
import { getTaskManager } from '../utils/tasks';
import { renderInputPanel, renderStatusBar } from '../utils/text';

export interface BottomAreaProps {
  state: TuiState;
}

export function BottomArea(props: BottomAreaProps): string[] {
  const { state } = props;
  const c = state.palette;
  const w = state.cols;
  const lines: string[] = [];

  const activeModel = state.configDraft?.model || 'deepseek-v4-pro';
  const inputPanel = renderInputPanel({
    width: w,
    input: state.input,
    model: activeModel,
    mode: state.mode || 'vibe',
    theme: state.theme,
    status: getStatusText(state),
    statusColor: getStatusColor(state),
    palette: c,
    multiline: state.input.includes('\n'),
    liveStatus: state.liveStatus,
    spinnerFrame: state.spinnerFrame || 0,
    currentTool: state.currentTool
  });
  lines.push(inputPanel);
  lines.push('');

  if (!state.commandPaletteOpen) {
    const statusBar = renderStatusBar({
      width: w,
      left: buildStatusLeft(state),
      right: buildStatusRight(state),
      color: c.subtext,
      bgHex: c.mantle,
      textColor: c.text
    });
    lines.push(statusBar);
  }

  return lines;
}

function buildStatusLeft(state: TuiState): string {
  const c = state.palette;
  const parts: string[] = [];

  const modeUpper = (state.mode || 'vibe').toUpperCase();
  const modeBg =
    modeUpper === 'VIBE' ? c.teal : modeUpper === 'PLAN' ? c.mauve : c.peach || c.yellow;
  parts.push(`${fg(c.crust || c.base)}\x1b[48;2;${hexToRgb(modeBg)}m\x1b[1m ${modeUpper} ${reset}`);

  const filePath = state.activeFilePath || 'no file';
  const pos = state.cursorPos ? ` [L${state.cursorPos.line}:C${state.cursorPos.col}]` : '';
  parts.push(`${fg(c.text)}${filePath}${fg(c.subtext)}${pos}${reset}`);

  return parts.join('  ');
}

function buildStatusRight(state: TuiState): string {
  const c = state.palette;
  const parts: string[] = [];

  const u = state.tokenUsage || { inputTokens: 0, outputTokens: 0, estimatedCost: 0 };
  const costStr = u.estimatedCost > 0 ? ` ($${u.estimatedCost.toFixed(4)})` : '';
  parts.push(
    `${fg(c.yellow)}${formatTokens(u.inputTokens)}↑ ${formatTokens(u.outputTokens)}↓${costStr}${reset}`
  );

  const connColor = state.connectionStatus === 'connected' ? c.green : c.red;
  const connIcon = state.connectionStatus === 'connected' ? '●' : '○';
  parts.push(`${fg(connColor)}${connIcon} ${state.connectionStatus}${reset}`);

  return parts.join('  ');
}

function formatTokens(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1000) return `${(n / 1000).toFixed(1)}k`;
  return String(n);
}

function getStatusText(state: TuiState): string {
  switch (state.liveStatus) {
    case 'streaming':
      return 'streaming';
    case 'tool':
      return `tool: ${state.currentTool ?? '…'}`;
    case 'thinking':
      return 'thinking';
    default:
      return 'ready';
  }
}

function getStatusColor(state: TuiState): string {
  switch (state.liveStatus) {
    case 'streaming':
      return state.palette.yellow;
    case 'tool':
      return state.palette.lavender;
    case 'thinking':
      return state.palette.blue;
    default:
      return state.palette.green;
  }
}
