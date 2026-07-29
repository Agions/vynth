
import { SLASH_COMMANDS, type SlashCommand } from '../slash-commands';
import type { TuiState } from '../state/TuiState';
import { fg, reset } from '../theme';
import { hexToRgb } from '../utils/color';
import { padToWidth } from '../utils/unicode';

export interface SlashPaletteProps {
  state: TuiState;
  selectedIndex: number;
  filter: string;
}

export function filterSlashCommands(filter: string): SlashCommand[] {
  const q = filter.trim().toLowerCase().replace(/^\//, '');
  if (!q) return SLASH_COMMANDS;
  return SLASH_COMMANDS.filter(
    (c) =>
      c.name.toLowerCase().includes(q) ||
      c.desc.toLowerCase().includes(q) ||
      c.category.toLowerCase().includes(q)
  );
}

export function SlashPalette(props: SlashPaletteProps): string {
  const { state, selectedIndex, filter } = props;
  const c = state.palette;
  const w = state.cols;
  const innerW = Math.min(Math.max(52, w - 6), 76);

  const filtered = filterSlashCommands(filter);
  const title = `  ${fg(c.mauve)}\x1b[1m⚡ 斜杠快捷指令 (Slash Commands)${reset} ${fg(c.subtext)}[${filtered.length}]${reset}`;

  const lines: string[] = [title];

  if (filtered.length === 0) {
    const emptyMsg = `  ${fg(c.subtext)}(无匹配命令 - 按 Esc 关闭)${reset}`;
    lines.push(padToWidth(emptyMsg, innerW));
  } else {
    const categories: Array<SlashCommand['category']> = ['config', 'workflow', 'system'];
    const categoryMeta: Record<SlashCommand['category'], { label: string; color: string }> = {
      config:   { label: '─ 配置管理 (Config) ', color: c.blue },
      workflow: { label: '─ 工作流 (Workflow) ', color: c.green },
      system:   { label: '─ 系统与统计 (System) ', color: c.yellow },
    };

    const groups = new Map<SlashCommand['category'], SlashCommand[]>();
    for (const cmd of filtered) {
      if (!groups.has(cmd.category)) groups.set(cmd.category, []);
      groups.get(cmd.category)!.push(cmd);
    }

    const safeIdx = Math.max(0, Math.min(selectedIndex, filtered.length - 1));

    for (const cat of categories) {
      const cmds = groups.get(cat);
      if (!cmds || cmds.length === 0) continue;

      const meta = categoryMeta[cat];
      const catRow = `  ${fg(meta.color)}${meta.label}${reset}`;
      lines.push(padToWidth(catRow, innerW));

      for (const cmd of cmds) {
        const actualIdx = filtered.indexOf(cmd);
        const isSelected = actualIdx === safeIdx;
        const cursor = isSelected ? `${fg(c.mauve)}❯${reset}` : ' ';
        const nameStr = `${fg(c.mauve)}/${cmd.name.padEnd(9)}${reset}`;
        const iconStr = `${cmd.icon} `;
        const descStr = isSelected
          ? `${fg(c.text)}\x1b[1m${cmd.desc}${reset}`
          : `${fg(c.subtext)}${cmd.desc}${reset}`;
        const rowContent = `  ${cursor} ${nameStr} ${iconStr}${descStr}`;

        let formattedRow = padToWidth(rowContent, innerW);
        if (isSelected) {
          formattedRow = `\x1b[48;2;${hexToRgb(c.surface0 || c.mantle)}m${formattedRow}${reset}`;
        }
        lines.push(formattedRow);
      }
    }
  }

  const pageIndicator = filtered.length > 0 ? ` [${Math.min(selectedIndex + 1, filtered.length)}/${filtered.length}]` : '';
  const footerStr = `  ${fg(c.yellow)}↑↓${fg(c.subtext)} 选择   ${fg(c.yellow)}⏎${fg(c.subtext)} 确认执行   ${fg(c.yellow)}esc${fg(c.subtext)} 关闭${fg(c.mauve)}${pageIndicator}${reset}`;
  lines.push(footerStr);

  return lines.join('\n');
}
