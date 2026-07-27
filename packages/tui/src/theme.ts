export interface Palette {
  base: string;
  mantle: string;
  crust: string;
  text: string;
  subtext: string;
  mauve: string;
  lavender: string;
  teal: string;
  green: string;
  red: string;
  yellow: string;
  blue: string;
  surface0?: string;
  surface1?: string;
  surface2?: string;
  overlay0?: string;
  rosewater?: string;
  peach?: string;
  sapphire?: string;
}

const catppuccin: Record<'mocha' | 'latte', Palette> = {
  mocha: {
    base: '#1e1e2e',
    mantle: '#181825',
    crust: '#11111b',
    text: '#cdd6f4',
    subtext: '#a6adc8',
    mauve: '#cba6f7',
    lavender: '#b4befe',
    teal: '#94e2d5',
    green: '#a6e3a1',
    red: '#f38ba8',
    yellow: '#f9e2af',
    blue: '#89b4fa'
  },
  latte: {
    base: '#eff1f5',
    mantle: '#e6e9ef',
    crust: '#dce0e8',
    text: '#4c4f69',
    subtext: '#6c6f85',
    mauve: '#8839ef',
    lavender: '#7287fd',
    teal: '#179299',
    green: '#40a02b',
    red: '#d20f39',
    yellow: '#df8e1d',
    blue: '#1e66f5'
  }
};

export function palette(theme: 'mocha' | 'latte'): Palette {
  return catppuccin[theme];
}

export const reset = '\x1b[0m';

export function fg(hex: string): string {
  return `\x1b[38;2;${hexToRgb(hex)}m`;
}

export function bg(hex: string): string {
  return `\x1b[48;2;${hexToRgb(hex)}m`;
}

function hexToRgb(hex: string): string {
  const n = Number.parseInt(hex.replace('#', ''), 16);
  return `${(n >> 16) & 255};${(n >> 8) & 255};${n & 255}`;
}
