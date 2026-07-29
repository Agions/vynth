
// Re-export from new locations for backward compatibility
export { visibleWidth, wrapLine, truncateVisible, padToWidth } from './utils/unicode';
export {
  renderKbd,
  renderDivider,
  renderPanel,
  renderBadge,
  renderStatusBar,
  renderSection,
  renderInline,
  renderMessage,
  renderInputPanel,
  renderToolBlock
} from './utils/text';

import { type Palette, fg, reset } from './theme';
// Import for local use in renderInputArea
import { visibleWidth } from './utils/unicode';

// Legacy compatibility - these will be fully migrated in Phase 2
export const renderFrameStart = (width: number): string => {
  const ESC = '\x1b';
  return `${ESC}[0;${width}H${ESC}[J`;
};

export const renderInputArea = (opts: {
  width: number;
  input: string;
  model: string;
  mode: string;
  theme: string;
  status: string;
  statusColor: string;
  palette: Palette;
}): string[] => {
  const c = opts.palette;
  const w = opts.width;

  const leftText = `${opts.model} · ${opts.mode} · ${opts.theme}`;
  const rightText = opts.status;
  const leftVis = visibleWidth(leftText);
  const rightVis = visibleWidth(rightText);
  const dashTotal = Math.max(4, w - leftVis - rightVis - 4);
  const dashLeft = Math.floor(dashTotal / 2);
  const dashRight = dashTotal - dashLeft;
  const sep =
    `${fg(c.overlay0 ?? c.subtext)}${'─'.repeat(2)} ` +
    `${fg(c.subtext)}${leftText}${reset}` +
    ` ${fg(c.overlay0 ?? c.subtext)}${'─'.repeat(dashLeft)}` +
    `${fg(opts.statusColor)}${rightText}` +
    `${fg(c.overlay0 ?? c.subtext)}${'─'.repeat(dashRight)}${reset}`;

  const blank = '';

  const promptIcon = `${fg(c.mauve)}❯${reset}`;
  const inputText = `${fg(c.text)}${opts.input}${reset}`;
  const cursor = `${fg(c.subtext)}▏${reset}`;
  const inputLine = `  ${promptIcon} ${inputText}${cursor}`;

  const kbd = (key: string, label: string): string =>
    `${fg(c.overlay0 ?? c.subtext)} ${key} ${reset}` + `${fg(c.subtext)}${label}${reset}`;

  const hints = [
    kbd('⏎', ' send'),
    kbd('⎋', ' esc'),
    kbd('^c', ' quit'),
    kbd('⇧↑↓', ' scroll'),
    kbd('⇟', ' page'),
    kbd('⇥', ' tool'),
    kbd('enter', ' fold')
  ].join('   ');
  const hintLine = `  ${hints}`;

  return [sep, blank, inputLine, blank, hintLine];
};

export const clipHistory = (messages: string[], maxLines: number): string[] => {
  const total = messages.reduce((acc, m) => acc + m.split('\n').length, 0);
  if (total <= maxLines) return messages;
  const head = messages.slice(0, 1);
  const tail = messages.slice(-3);
  return [...head, `${'─'.repeat(40)}  earlier turns hidden  ${'─'.repeat(40)}`, ...tail];
};
