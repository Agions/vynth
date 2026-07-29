import type { TuiState } from '../state/TuiState';
import { fg, reset } from '../theme';
import { ansiBackground, hexToRgb } from '../utils/color';

export interface HintsBarProps {
  width: number;
  palette: ReturnType<typeof import('../theme').palette>;
  liveStatus: TuiState['liveStatus'];
}

export function renderHintsBar(opts: HintsBarProps): string {
  const c = opts.palette;
  const w = Math.max(20, opts.width);
  const bgHex = c.mantle ?? c.base;

  type KbdPair = [string, string];
  let pairs: KbdPair[];

  if (opts.liveStatus === 'streaming' || opts.liveStatus === 'thinking') {
    pairs = [
      ['Esc×2', 'interrupt'],
      ['/', 'slash'],
      ['?', 'help']
    ];
  } else if (opts.liveStatus === 'tool') {
    pairs = [
      ['↵', 'toggle'],
      ['Tab', 'next'],
      ['Esc', 'deselect'],
      ['?', 'help']
    ];
  } else {
    pairs = [
      ['⏎', 'send'],
      ['⇧↵', 'newline'],
      ['/', 'slash'],
      ['@', 'file'],
      ['↑↓', 'hist'],
      ['?', 'help']
    ];
  }

  const parts = pairs.map(
    ([key, label]) =>
      `${ansiBackground(c.surface0 ?? c.mantle)}${fg(c.text)} ${key} ${reset}${fg(c.subtext)} ${label}${reset}`
  );
  const hintStr = `  ${parts.join('   ')}`;
  const len = hintStr.replace(/\x1b\[[^m]*m/g, '').length;
  const pad = Math.max(0, w - len - 2);

  return `${ansiBackground(bgHex)}${hintStr}${' '.repeat(pad)}  ${reset}`;
}
