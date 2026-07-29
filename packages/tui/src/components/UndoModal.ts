import type { TuiState } from '../state/TuiState';
import { fg, reset } from '../theme';
import { padToWidth, visibleWidth } from '../utils/unicode';

export interface UndoModalProps {
  state: TuiState;
}

export function UndoModal(props: UndoModalProps): string {
  const { state } = props;
  const c = state.palette;
  const w = state.cols;
  const panelW = Math.min(Math.max(46, w - 4), 68);
  const innerW = panelW - 2;

  const borderCol = fg(c.red);
  const title = ' ↺ 确认回撤上一步修改？ ';
  const leftW = visibleWidth(title);
  const dashesW = Math.max(2, panelW - 3 - leftW);
  const topBorder = `${borderCol}╭─${fg(c.red)}\x1b[1m${title}${borderCol}${'─'.repeat(dashesW)}╮${reset}`;

  const lines: string[] = [topBorder];
  lines.push(`${borderCol}│${reset}${padToWidth('', innerW)}${borderCol}│${reset}`);

  const bodyLines = [
    `  ${fg(c.yellow)}⚠  即将回撤上一次 AI 对话与代码修改回合${reset}`,
    `  ${fg(c.subtext)}对话历史将退回上一节点，便于重新尝试或更正指令。${reset}`
  ];
  for (const line of bodyLines) {
    lines.push(`${borderCol}│${reset}${padToWidth(line, innerW)}${borderCol}│${reset}`);
  }

  lines.push(`${borderCol}│${reset}${padToWidth('', innerW)}${borderCol}│${reset}`);
  const footerStr = ` ${fg(c.green)}⏎ / Y${fg(c.subtext)} 确认回撤   ${fg(c.red)}esc / N${fg(c.subtext)} 取消 `;
  const footerW = visibleWidth(footerStr);
  const botDashW = Math.max(2, innerW - footerW - 2);
  const botBorder = `${borderCol}╰─${footerStr}${borderCol}${'─'.repeat(botDashW)}╯${reset}`;
  lines.push(botBorder);

  return lines.join('\n');
}
