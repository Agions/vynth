
import type { TuiState } from '../state/TuiState';
import { fg, reset } from '../theme';
import { hexToRgb } from '../utils/color';
import { visibleWidth } from '../utils/unicode';



function pill(text: string, fgHex: string, bgHex: string): string {
  return `\x1b[48;2;${hexToRgb(bgHex)}m${fg(fgHex)} ${text} ${reset}`;
}

function fmtTokens(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1000) return `${(n / 1000).toFixed(1)}k`;
  return String(n);
}

export function TopBar(state: TuiState): string[] {
  const c = state.palette;
  const cols = state.cols;
  const bgHex = c.mantle;

  // ─── Line 1: Brand + Mode + Model + Theme + Connection ───
  const modeUpper = (state.mode || 'vibe').toUpperCase();
  const modeColor = modeUpper === 'PLAN' ? c.mauve : modeUpper === 'AUTO' ? (c.peach || c.yellow) : c.teal;
  const activeModel = state.configDraft?.model || 'deepseek-v4-pro';

  const connColor =
    state.connectionStatus === 'connected' ? c.green
    : state.connectionStatus === 'error' ? c.yellow
    : c.red;
  const connIcon =
    state.connectionStatus === 'connected' ? '●'
    : state.connectionStatus === 'error' ? '◐'
    : '○';

  const brand = `${fg(c.mauve)}\x1b[1mZeno${reset}`;
  const modePill = pill(modeUpper, c.crust ?? c.base, modeColor);
  const modelPill = pill(activeModel, c.crust ?? c.base, c.teal);
  const themePill = pill(state.theme, c.crust ?? c.base, c.blue);
  const connPill = pill(`${connIcon} ${state.connectionStatus}`, c.crust ?? c.base, connColor);

  const leftTop = `\x1b[48;2;${hexToRgb(bgHex)}m ${brand}  ${modePill}  ${modelPill}  ${themePill}  ${connPill}`;
  const leftTopVW = visibleWidth(leftTop);
  const rightTop = '';
  const padTop = Math.max(0, cols - leftTopVW);
  const line1 = `${leftTop}${' '.repeat(padTop)}${reset}`;

  // ─── Line 2: Live status + Token quick stats + Turn count ───
  const liveColor =
    state.liveStatus === 'streaming' ? c.yellow
    : state.liveStatus === 'tool' ? c.lavender
    : state.liveStatus === 'thinking' ? c.blue
    : c.green;
  const liveIcon =
    state.liveStatus === 'streaming' ? '⠿'
    : state.liveStatus === 'tool' ? '⚙'
    : state.liveStatus === 'thinking' ? '◈'
    : '✔';
  const liveText =
    state.liveStatus === 'streaming' ? 'streaming'
    : state.liveStatus === 'tool' ? `tool: ${state.currentTool ?? '…'}`
    : state.liveStatus === 'thinking' ? 'thinking'
    : 'ready';
  const statusPill = pill(`${liveIcon} ${liveText}`, c.crust ?? c.base, liveColor);

  // Token quick stats
  const u = state.tokenUsage;
  const tokenStr = u && (u.inputTokens + u.outputTokens > 0)
    ? `${fg(c.teal)}↑${fmtTokens(u.inputTokens)}${reset}  ${fg(c.mauve)}↓${fmtTokens(u.outputTokens)}${reset}` +
      (u.estimatedCost > 0 ? `  ${fg(c.yellow)}$${u.estimatedCost.toFixed(4)}${reset}` : '')
    : `${fg(c.subtext)}no tokens${reset}`;

  const turnPill = pill(`T${state.turnCount}`, c.crust ?? c.base, c.overlay0 || c.subtext);

  const leftBot = `\x1b[48;2;${hexToRgb(bgHex)}m  ${statusPill}  ${tokenStr}`;
  const leftBotVW = visibleWidth(leftBot);
  const rightBotStr = `${turnPill} `;
  const rightBotVW = visibleWidth(rightBotStr);
  const padBot = Math.max(0, cols - leftBotVW - rightBotVW);
  const line2 = `${leftBot}${' '.repeat(padBot)}${rightBotStr}${reset}`;

  return [line1, line2];
}
