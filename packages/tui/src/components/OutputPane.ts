import type { TuiState } from '../state/TuiState';
import { fg, reset } from '../theme';
import { truncateVisible } from '../utils/unicode';

export interface OutputPaneProps {
  state: TuiState;
  width: number;
  height: number;
}

export function collectOutputLines(state: TuiState): string[] {
  const c = state.palette;
  const lines: string[] = [];
  for (const entry of state.transcript) {
    if (entry.role !== 'tool' || !entry.toolId) continue;
    const name = entry.content.split('(')[0] || 'tool';
    const status = entry.status ?? 'ok';
    const mark =
      status === 'error'
        ? `${fg(c.red)}✖${reset}`
        : status === 'running'
          ? `${fg(c.yellow)}⠋${reset}`
          : `${fg(c.green)}✔${reset}`;
    lines.push(`${mark} ${fg(c.mauve)}${name}${reset}`);
    const out = (entry.output ?? '').replace(/\s+$/, '');
    if (out) {
      for (const raw of out.split('\n')) {
        lines.push(`  ${fg(c.subtext)}${raw}${reset}`);
      }
    }
    lines.push('');
  }
  return lines;
}

export function OutputPane(props: OutputPaneProps): string[] {
  const { state, width, height } = props;
  const c = state.palette;
  const focused = state.splitFocus === 'output';

  const lines: string[] = [];

  const label = focused
    ? `${fg(c.mauve)}\x1b[1m▎输出${reset}${fg(c.subtext)} · 焦点在此 · ^E 切回${reset}`
    : `${fg(c.subtext)}▎输出 · ^E 聚焦${reset}`;
  const barColor = fg(c.overlay0 ?? c.subtext);
  const labelPlain = focused ? '▎输出 · 焦点在此 · ^E 切回' : '▎输出 · ^E 聚焦';
  const dashLen = Math.max(1, width - labelPlain.length - 3);
  lines.push(`${barColor}┈┈${reset} ${label} ${barColor}${'┈'.repeat(dashLen)}${reset}`);

  const bodyH = height - 1;
  const all = collectOutputLines(state);
  const maxScroll = Math.max(0, all.length - bodyH);
  const offset = Math.min(state.outputScrollOffset, maxScroll);
  const start = Math.max(0, all.length - bodyH - offset);
  const visible = all.slice(start, start + bodyH);

  if (all.length === 0) {
    lines.push(`  ${fg(c.subtext)}暂无工具输出 —— agent 调用工具后在此展示完整结果${reset}`);
    for (let i = 1; i < bodyH; i++) lines.push('');
  } else {
    for (let i = 0; i < bodyH; i++) {
      lines.push(truncateVisible(visible[i] ?? '', width));
    }
  }

  return lines.slice(0, height);
}
