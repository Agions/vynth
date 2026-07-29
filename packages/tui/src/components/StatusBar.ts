
import type { TuiState } from '../state/TuiState';
import { fg, reset } from '../theme';
import { renderStatusBar } from '../utils/text';

export interface StatusBarProps {
  state: TuiState;
}

export function StatusBar(props: StatusBarProps): string {
  const { state } = props;
  const c = state.palette;
  const w = state.cols;

  const left = buildLeft(state);
  const right = buildRight(state);

  return renderStatusBar({
    width: w,
    left,
    right,
    color: c.subtext,
    bgHex: c.mantle,
    textColor: c.text
  });
}

function buildLeft(state: TuiState): string {
  const c = state.palette;
  const parts: string[] = [];

  const modeName = (state.mode || 'vibe').toUpperCase();
  const modeBg = modeName === 'VIBE' ? c.teal : c.mauve;
  parts.push(`${fg(c.crust || c.base)}\x1b[48;2;${hexToRgb(modeBg)}m ${modeName} ${reset}`);

  const file = state.activeFilePath || 'no file';
  const pos = state.cursorPos ? ` [L${state.cursorPos.line}:C${state.cursorPos.col}]` : '';
  parts.push(`${fg(c.text)}${file}${fg(c.subtext)}${pos}${reset}`);

  return parts.join('  ');
}

function buildRight(state: TuiState): string {
  const c = state.palette;
  const parts: string[] = [];

  const inTok = state.tokenUsage?.inputTokens ?? (state.turnCount * 450);
  const outTok = state.tokenUsage?.outputTokens ?? (state.turnCount * 820);
  const cost = state.tokenUsage?.estimatedCost ?? ( (inTok * 0.000002) + (outTok * 0.000008) );
  parts.push(`${fg(c.yellow)}Token: ${formatTokens(inTok)} in / ${formatTokens(outTok)} out ($${cost.toFixed(4)})${reset}`);

  const connColor = state.connectionStatus === 'connected' ? c.green : c.red;
  parts.push(`${fg(connColor)}● connected${reset}`);

  return parts.join('  ');
}

function hexToRgb(hex: string): string {
  const n = Number.parseInt((hex || '#ffffff').replace('#', ''), 16);
  return `${(n >> 16) & 255};${(n >> 8) & 255};${n & 255}`;
}

function formatTokens(n: number): string {
  if (n >= 1000) return `${(n / 1000).toFixed(1)}k`;
  return String(n);
}

