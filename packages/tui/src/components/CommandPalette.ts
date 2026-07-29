
import type { TuiState } from '../state/TuiState';
import { fg, reset } from '../theme';
import { hexToRgb } from '../utils/color';
import { padToWidth, visibleWidth } from '../utils/unicode';

export interface Command {
  key: string;
  label: string;
  description: string;
  category: 'navigation' | 'editing' | 'view' | 'system';
}

export const COMMANDS: Command[] = [
  // Navigation
  { key: '?', label: 'Command Palette', description: '显示命令面板', category: 'navigation' },
  { key: 'Ctrl+F', label: 'Search', description: '搜索对话内容', category: 'navigation' },
  { key: 'Ctrl+N', label: 'Next Message', description: '跳转到下一条消息', category: 'navigation' },
  { key: 'Ctrl+P', label: 'Previous Message', description: '跳转到上一条消息', category: 'navigation' },
  { key: 'Home', label: 'Scroll Top', description: '滚动到顶部', category: 'navigation' },
  { key: 'End', label: 'Scroll Bottom', description: '滚动到底部', category: 'navigation' },

  // Editing
  { key: 'Tab', label: 'Next Tool', description: '选中下一个工具块', category: 'editing' },
  { key: 'Shift+Tab', label: 'Previous Tool', description: '选中上一个工具块', category: 'editing' },
  { key: 'Enter', label: 'Toggle Tool', description: '展开/折叠工具块', category: 'editing' },
  { key: 'Esc', label: 'Deselect', description: '取消工具选中', category: 'editing' },

  // View
  { key: '/brief', label: 'Brief Mode', description: '开关折叠工具输出', category: 'view' },
  { key: '/theme', label: 'Cycle Theme', description: '循环切换主题', category: 'view' },
  { key: '/config', label: 'AI Config', description: '配置模型/端点/密钥', category: 'view' },
  { key: '/clear', label: 'Clear Chat', description: '清空对话历史', category: 'view' },
  { key: '/usage', label: 'Token Usage', description: 'Token 用量与费用统计', category: 'view' },
  { key: '/tasks', label: 'Tasks', description: '后台任务面板', category: 'view' },

  // System
  { key: 'Ctrl+C', label: 'Quit', description: '退出 Zeno', category: 'system' },
  { key: 'Ctrl+L', label: 'Clear Screen', description: '清空终端屏幕', category: 'system' },
];

export interface CommandPaletteProps {
  state: TuiState;
  selectedIndex: number;
  filter: string;
}

export function CommandPalette(props: CommandPaletteProps): string {
  const { state, selectedIndex, filter } = props;
  const c = state.palette;
  const w = state.cols;
  const panelW = Math.min(Math.max(56, w - 4), 80);
  const innerW = panelW - 2;
  const borderCol = fg(c.mauve);

  const filtered = COMMANDS.filter((cmd) => {
    if (!filter) return true;
    const q = filter.toLowerCase();
    return (
      cmd.label.toLowerCase().includes(q) ||
      cmd.description.toLowerCase().includes(q) ||
      cmd.key.toLowerCase().includes(q) ||
      cmd.category.toLowerCase().includes(q)
    );
  });

  const title = ` ⌘ 命令面板 (${filtered.length}) `;
  const leftW = visibleWidth(title);
  const dashesW = Math.max(2, panelW - 3 - leftW);
  const topBorder = `${borderCol}╭─${fg(c.mauve)}\x1b[1m${title}${borderCol}${'─'.repeat(dashesW)}╮${reset}`;

  const lines: string[] = [topBorder];
  lines.push(`${borderCol}│${reset}${padToWidth('', innerW)}${borderCol}│${reset}`);

  const categories: Array<Command['category']> = ['navigation', 'editing', 'view', 'system'];
  const categoryMeta: Record<string, { label: string; color: string }> = {
    navigation: { label: '─ 导航 ', color: c.blue },
    editing:    { label: '─ 编辑 ', color: c.green },
    view:       { label: '─ 视图 ', color: c.yellow },
    system:     { label: '─ 系统 ', color: c.red },
  };

  const groups = new Map<string, Command[]>();
  for (const cmd of filtered) {
    if (!groups.has(cmd.category)) groups.set(cmd.category, []);
    groups.get(cmd.category)!.push(cmd);
  }

  for (const cat of categories) {
    const cmds = groups.get(cat);
    if (!cmds || cmds.length === 0) continue;

    const meta = categoryMeta[cat];
    const catRow = `  ${fg(meta.color)}${meta.label}${reset}`;
    lines.push(`${borderCol}│${reset}${padToWidth(catRow, innerW)}${borderCol}│${reset}`);

    for (const cmd of cmds) {
      const idx = filtered.indexOf(cmd);
      const isSelected = idx === selectedIndex;
      const cursor = isSelected ? `${fg(c.mauve)}❯${reset}` : ' ';
      const keyStr = `${fg(c.mauve)}${cmd.key.padEnd(12)}${reset}`;
      const labelStr = isSelected
        ? `${fg(c.text)}\x1b[1m${cmd.label.padEnd(16)}${reset}`
        : `${fg(c.text)}${cmd.label.padEnd(16)}${reset}`;
      const descStr = `${fg(c.subtext)}${cmd.description}${reset}`;
      const rowContent = ` ${cursor} ${keyStr}${labelStr}${descStr}`;
      let row = padToWidth(rowContent, innerW);
      if (isSelected) row = `\x1b[48;2;${hexToRgb(c.surface0 || c.mantle)}m${row}${reset}`;
      lines.push(`${borderCol}│${reset}${row}${borderCol}│${reset}`);
    }
  }

  lines.push(`${borderCol}│${reset}${padToWidth('', innerW)}${borderCol}│${reset}`);
  const footerStr = ` ${fg(c.yellow)}↑↓${fg(c.subtext)} 导航   ${fg(c.yellow)}⏎${fg(c.subtext)} 执行   ${fg(c.yellow)}esc${fg(c.subtext)} 关闭 `;
  const footerW = visibleWidth(footerStr);
  const botDashW = Math.max(2, panelW - 3 - footerW);
  const botBorder = `${borderCol}╰─${footerStr}${borderCol}${'─'.repeat(botDashW)}╯${reset}`;
  lines.push(botBorder);

  return lines.join('\n');
}
