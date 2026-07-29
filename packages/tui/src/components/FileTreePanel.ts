import type { TuiState } from '../state/TuiState';
import { fg, reset } from '../theme';
import { hexToRgb } from '../utils/color';
import { padToWidth, visibleWidth } from '../utils/unicode';

export interface FileTreePanelProps {
  state: TuiState;
}

export function FileTreePanel(props: FileTreePanelProps): string {
  const { state } = props;
  const c = state.palette;
  const w = state.cols;
  const panelW = Math.min(Math.max(44, w - 4), 64);
  const innerW = panelW - 2;

  const items = state.fileTreeData?.length
    ? state.fileTreeData
    : ['📁 src/', '  📄 index.ts', '  📄 config.ts', '📁 tests/', '  📄 main.test.ts'];
  const selIdx = state.fileTreeIndex || 0;
  const borderCol = fg(c.teal);

  const title = ` 📁 工作区文件树 `;
  const leftW = visibleWidth(title);
  const dashesW = Math.max(2, panelW - 3 - leftW);
  const topBorder = `${borderCol}╭─${fg(c.teal)}\x1b[1m${title}${borderCol}${'─'.repeat(dashesW)}╮${reset}`;

  const lines: string[] = [topBorder];
  lines.push(`${borderCol}│${reset}${padToWidth('', innerW)}${borderCol}│${reset}`);

  const PAGE = 20;
  const slice = items.slice(0, PAGE);
  slice.forEach((item, i) => {
    const isSelected = i === selIdx;
    const cursor = isSelected ? `${fg(c.teal)}❯${reset}` : ' ';
    const isDir = item.trimStart().startsWith('📁');
    const itemColored = isSelected
      ? `${fg(c.teal)}\x1b[1m${item}${reset}`
      : isDir
        ? `${fg(c.lavender)}${item}${reset}`
        : `${fg(c.text)}${item}${reset}`;
    const rowContent = ` ${cursor} ${itemColored}`;
    let row = padToWidth(rowContent, innerW);
    if (isSelected) row = `\x1b[48;2;${hexToRgb(c.surface0 || c.mantle)}m${row}${reset}`;
    lines.push(`${borderCol}│${reset}${row}${borderCol}│${reset}`);
  });

  if (items.length > PAGE) {
    const more = `  ${fg(c.subtext)}… 共 ${items.length} 个条目${reset}`;
    lines.push(`${borderCol}│${reset}${padToWidth(more, innerW)}${borderCol}│${reset}`);
  }

  lines.push(`${borderCol}│${reset}${padToWidth('', innerW)}${borderCol}│${reset}`);
  const footerStr = ` ${fg(c.yellow)}↑↓${fg(c.subtext)} 导航   ${fg(c.yellow)}⏎${fg(c.subtext)} 引用   ${fg(c.yellow)}esc${fg(c.subtext)} 关闭 `;
  const footerW = visibleWidth(footerStr);
  const botDashW = Math.max(2, panelW - 3 - footerW);
  const botBorder = `${borderCol}╰─${footerStr}${borderCol}${'─'.repeat(botDashW)}╯${reset}`;
  lines.push(botBorder);

  return lines.join('\n');
}
