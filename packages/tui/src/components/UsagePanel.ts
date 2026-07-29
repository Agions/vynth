
import type { TuiState } from '../state/TuiState';
import { fg, reset } from '../theme';
import { formatCost, resolveRate } from '../utils/pricing';
import { padToWidth, visibleWidth } from '../utils/unicode';

export interface UsagePanelProps {
  state: TuiState;
  model: string;
}

export function UsagePanel(props: UsagePanelProps): string {
  const { state, model } = props;
  const c = state.palette;
  const w = state.cols;
  const panelW = Math.min(Math.max(52, w - 4), 72);
  const innerW = panelW - 2;
  const u = state.tokenUsage;
  const rate = resolveRate(model);
  const totalTokens = u.inputTokens + u.outputTokens;
  const borderCol = fg(c.teal);

  const title = ` 📊 Token 用量统计 `;
  const leftW = visibleWidth(title);
  const dashesW = Math.max(2, panelW - 3 - leftW);
  const topBorder = `${borderCol}╭─${fg(c.teal)}\x1b[1m${title}${borderCol}${'─'.repeat(dashesW)}╮${reset}`;

  const lines: string[] = [topBorder];
  lines.push(`${borderCol}│${reset}${padToWidth('', innerW)}${borderCol}│${reset}`);

  const row = (label: string, val: string) => {
    const r = `  ${fg(c.subtext)}${label.padEnd(10)}${reset}${val}`;
    return `${borderCol}│${reset}${padToWidth(r, innerW)}${borderCol}│${reset}`;
  };

  // Model & rate
  lines.push(row('Model', `${fg(c.teal)}${model}${reset}`));
  lines.push(row('Rate', `${fg(c.subtext)}$${rate.input}/M in · $${rate.output}/M out${reset}`));
  lines.push(`${borderCol}│${reset}${padToWidth('', innerW)}${borderCol}│${reset}`);

  // Bar chart – visual breakdown
  const inputPct = totalTokens > 0 ? (u.inputTokens / totalTokens) : 0.5;
  const BAR_W = Math.max(8, innerW - 20);
  const inFill = Math.round(inputPct * BAR_W);
  const outFill = BAR_W - inFill;
  const bar = `${fg(c.teal)}${'█'.repeat(inFill)}${fg(c.mauve)}${'█'.repeat(outFill)}${reset}`;
  const barRow = `  ${fg(c.subtext)}in${reset} ${bar} ${fg(c.subtext)}out${reset}`;
  lines.push(`${borderCol}│${reset}${padToWidth(barRow, innerW)}${borderCol}│${reset}`);
  lines.push(`${borderCol}│${reset}${padToWidth('', innerW)}${borderCol}│${reset}`);

  // Token counts
  lines.push(row('↑ Input', `${fg(c.teal)}${fmt(u.inputTokens)} tokens${reset}`));
  lines.push(row('↓ Output', `${fg(c.mauve)}${fmt(u.outputTokens)} tokens${reset}`));
  lines.push(row('Total', `${fg(c.text)}${fmt(totalTokens)} tokens${reset}`));
  lines.push(`${borderCol}│${reset}${padToWidth('', innerW)}${borderCol}│${reset}`);

  // Cost and request count
  lines.push(row('Cost', `${fg(c.yellow)}\x1b[1m${formatCost(u.estimatedCost)}${reset}`));
  lines.push(row('Requests', `${fg(c.text)}${u.history.length}${reset}`));

  // Recent history
  if (u.history.length > 0) {
    lines.push(`${borderCol}│${reset}${padToWidth('', innerW)}${borderCol}│${reset}`);
    const divider = `  ${fg(c.overlay0 ?? c.subtext)}── 近期请求明细 ──${reset}`;
    lines.push(`${borderCol}│${reset}${padToWidth(divider, innerW)}${borderCol}│${reset}`);
    for (const h of u.history.slice(-6)) {
      const t = new Date(h.ts);
      const time = `${String(t.getHours()).padStart(2, '0')}:${String(t.getMinutes()).padStart(2, '0')}`;
      const rec =
        `  ${fg(c.subtext)}${time}${reset}  ` +
        `${fg(c.teal)}↑${fmt(h.input)}${reset}  ` +
        `${fg(c.mauve)}↓${fmt(h.output)}${reset}  ` +
        `${fg(c.yellow)}${formatCost(h.cost)}${reset}`;
      lines.push(`${borderCol}│${reset}${padToWidth(rec, innerW)}${borderCol}│${reset}`);
    }
  } else {
    lines.push(`${borderCol}│${reset}${padToWidth('', innerW)}${borderCol}│${reset}`);
    const empty = `  ${fg(c.subtext)}暂无请求记录 — 发送消息后即可显示统计${reset}`;
    lines.push(`${borderCol}│${reset}${padToWidth(empty, innerW)}${borderCol}│${reset}`);
  }

  lines.push(`${borderCol}│${reset}${padToWidth('', innerW)}${borderCol}│${reset}`);
  const footerStr = ` ${fg(c.yellow)}Ctrl+U${fg(c.subtext)} 开关   ${fg(c.yellow)}esc${fg(c.subtext)} 关闭 `;
  const footerW = visibleWidth(footerStr);
  const botDashW = Math.max(2, panelW - 3 - footerW);
  const botBorder = `${borderCol}╰─${footerStr}${borderCol}${'─'.repeat(botDashW)}╯${reset}`;
  lines.push(botBorder);

  return lines.join('\n');
}

function fmt(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(2)}M`;
  if (n >= 1000) return `${(n / 1000).toFixed(1)}k`;
  return String(n);
}
