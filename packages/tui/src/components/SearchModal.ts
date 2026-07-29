import type { TuiState } from '../state/TuiState';
import { fg, reset } from '../theme';
import { hexToRgb } from '../utils/color';
import { padToWidth, visibleWidth } from '../utils/unicode';

export interface SearchModalProps {
  state: TuiState;
}

export function SearchModal(props: SearchModalProps): string {
  const { state } = props;
  const c = state.palette;
  const w = state.cols;
  const panelW = Math.min(Math.max(52, w - 4), 76);
  const innerW = panelW - 2;

  const query = state.searchModalQuery || '';
  const results = state.searchModalResults || [];
  const selIdx = state.searchModalIndex || 0;
  const borderCol = fg(c.blue);

  const title = ' 🔍 全局正则搜索 ';
  const leftW = visibleWidth(title);
  const dashesW = Math.max(2, panelW - 3 - leftW);
  const topBorder = `${borderCol}╭─${fg(c.blue)}\x1b[1m${title}${borderCol}${'─'.repeat(dashesW)}╮${reset}`;

  const lines: string[] = [topBorder];

  // Query row
  const queryLabel = `${fg(c.subtext)}  Pattern: ${reset}`;
  const queryVal = query
    ? `${fg(c.teal)}${query}${reset}`
    : `${fg(c.subtext)}(输入关键词或正则表达式)${reset}`;
  const queryRow = `${queryLabel}${queryVal}`;
  lines.push(`${borderCol}│${reset}${padToWidth(queryRow, innerW)}${borderCol}│${reset}`);
  lines.push(`${borderCol}│${reset}${padToWidth('', innerW)}${borderCol}│${reset}`);

  if (results.length === 0) {
    const msg = query
      ? `${fg(c.subtext)}  ○ 无匹配结果${reset}`
      : `${fg(c.subtext)}  ○ 请输入关键词开始搜索${reset}`;
    lines.push(`${borderCol}│${reset}${padToWidth(msg, innerW)}${borderCol}│${reset}`);
  } else {
    const PAGE = 10;
    const slice = results.slice(0, PAGE);
    slice.forEach((res, i) => {
      const isSelected = i === selIdx;
      const cursor = isSelected ? `${fg(c.blue)}❯${reset}` : ' ';
      const rowStr = isSelected
        ? `${fg(c.blue)}\x1b[1m${res}${reset}`
        : `${fg(c.text)}${res}${reset}`;
      const rowContent = ` ${cursor} ${rowStr}`;
      let row = padToWidth(rowContent, innerW);
      if (isSelected) row = `\x1b[48;2;${hexToRgb(c.surface0 || c.mantle)}m${row}${reset}`;
      lines.push(`${borderCol}│${reset}${row}${borderCol}│${reset}`);
    });
    if (results.length > PAGE) {
      const more = `  ${fg(c.subtext)}… 共 ${results.length} 条命中${reset}`;
      lines.push(`${borderCol}│${reset}${padToWidth(more, innerW)}${borderCol}│${reset}`);
    }
  }

  lines.push(`${borderCol}│${reset}${padToWidth('', innerW)}${borderCol}│${reset}`);
  const footerStr = ` ${fg(c.yellow)}↑↓${fg(c.subtext)} 选择   ${fg(c.yellow)}⏎${fg(c.subtext)} 跳转   ${fg(c.yellow)}esc${fg(c.subtext)} 关闭 `;
  const footerW = visibleWidth(footerStr);
  const botDashW = Math.max(2, panelW - 3 - footerW);
  const botBorder = `${borderCol}╰─${footerStr}${borderCol}${'─'.repeat(botDashW)}╯${reset}`;
  lines.push(botBorder);

  return lines.join('\n');
}
