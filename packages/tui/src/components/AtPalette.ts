import type { TuiState } from '../state/TuiState';
import { fg, reset } from '../theme';
import { hexToRgb } from '../utils/color';
import { padToWidth } from '../utils/unicode';

export interface AtPaletteProps {
  state: TuiState;
  selectedIndex: number;
  filter: string;
  files: string[];
}

export function filterAtFiles(files: string[], filter: string): string[] {
  const q = filter.trim().toLowerCase().replace(/^@/, '');
  if (!q) return files;
  return files.filter((f) => f.toLowerCase().includes(q));
}

export function AtPalette(props: AtPaletteProps): string {
  const { state, selectedIndex, filter, files } = props;
  const c = state.palette;
  const w = state.cols;
  const innerW = Math.min(Math.max(44, w - 6), 72);

  const filtered = filterAtFiles(files, filter);
  const title = `  ${fg(c.mauve)}\x1b[1m@ 项目文件引用 (${filtered.length})${reset}`;

  const lines: string[] = [title];

  if (filtered.length === 0) {
    const emptyMsg = `  ${fg(c.subtext)}(未检索到匹配文件)${reset}`;
    lines.push(padToWidth(emptyMsg, innerW));
  } else {
    const PAGE_SIZE = 10;
    const safeIdx = Math.max(0, Math.min(selectedIndex, filtered.length - 1));
    const startIdx = Math.max(
      0,
      Math.min(safeIdx - Math.floor(PAGE_SIZE / 2), Math.max(0, filtered.length - PAGE_SIZE))
    );
    const visibleSlice = filtered.slice(startIdx, startIdx + PAGE_SIZE);

    visibleSlice.forEach((file, relativeIdx) => {
      const actualIdx = startIdx + relativeIdx;
      const isSelected = actualIdx === safeIdx;
      const cursor = isSelected ? `${fg(c.mauve)}❯${reset}` : ' ';
      const icon = file.endsWith('/') ? '📁' : '📄';
      const fileStr = `${fg(c.text)}${icon} ${file}${reset}`;
      const rowContent = `  ${cursor} ${fileStr}`;

      let formattedRow = padToWidth(rowContent, innerW);
      if (isSelected) {
        formattedRow = `\x1b[48;2;${hexToRgb(c.surface0 || c.mantle)}m${formattedRow}${reset}`;
      }
      lines.push(formattedRow);
    });
  }

  const footerStr = `  ${fg(c.yellow)}↑↓${fg(c.subtext)} 选择   ${fg(c.yellow)}⏎${fg(c.subtext)} 引用上下文   ${fg(c.yellow)}esc${fg(c.subtext)} 关闭`;
  lines.push(footerStr);

  return lines.join('\n');
}
