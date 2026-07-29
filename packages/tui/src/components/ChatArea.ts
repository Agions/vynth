import type { TuiState } from '../state/TuiState';
import { renderMessage, renderToolBlock } from '../utils/text';
import { renderWelcome } from './Welcome';

import { fg, reset } from '../theme';

export function ChatArea(state: TuiState): string[] {
  const lines: string[] = [];
  const innerW = Math.max(10, state.cols - 4);

  if (state.transcript.length === 0) {
    const welcomeText = renderWelcome({
      version: '0.1.1',
      model: state.configDraft?.model || 'deepseek-v4-pro',
      mode: state.mode || 'vibe',
      cwd: process.cwd(),
      theme: state.theme || 'mocha',
      palette: state.palette,
      width: state.cols
    });
    return welcomeText.split('\n');
  }

  for (const entry of state.transcript) {
    if (entry.role === 'tool' && entry.toolId) {
      const isCollapsed = state.collapsedTools.has(entry.toolId);
      const isSelected = state.selectedToolId === entry.toolId;
      const name = entry.content.split('(')[0] || 'tool';
      const args = entry.content.includes('(')
        ? entry.content.slice(entry.content.indexOf('('))
        : '{}';
      const block = renderToolBlock({
        name,
        args,
        status: entry.status ?? 'ok',
        output: entry.output ?? '',
        palette: state.palette,
        width: innerW,
        collapsed: isCollapsed,
        selected: isSelected,
        spinnerFrame: state.spinnerFrame
      });
      for (const line of block.split('\n')) {
        lines.push(line);
      }
    } else {
      const block = renderMessage({
        role: entry.role,
        content: entry.content,
        palette: state.palette,
        width: innerW
      });
      for (const line of block.split('\n')) {
        lines.push(line);
      }
    }
    lines.push(''); // empty line between messages
  }

  if (state.liveStatus && state.liveStatus !== 'idle') {
    const c = state.palette;
    const SPINNER_FRAMES = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
    const frame = SPINNER_FRAMES[(state.spinnerFrame || 0) % SPINNER_FRAMES.length];
    let liveBadge = '';
    if (state.liveStatus === 'thinking') {
      liveBadge = `  ${fg(c.blue)}\x1b[1m∷ ${frame} thinking…${reset}`;
    } else if (state.liveStatus === 'streaming') {
      liveBadge = `  ${fg(c.yellow)}\x1b[1m⠿ ${frame} streaming response…${reset}`;
    } else if (state.liveStatus === 'tool') {
      liveBadge = `  ${fg(c.lavender)}\x1b[1m⚙ ${frame} executing ${state.currentTool || 'tool'}…${reset}`;
    }
    if (liveBadge) {
      lines.push(liveBadge);
      lines.push('');
    }
  }

  return lines;
}
