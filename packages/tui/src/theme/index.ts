
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

export type ThemeName = 'mocha' | 'latte' | 'midnight' | 'forest' | 'light' | 'neon';

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

const midnight: Palette = {
  base: '#0f172a',
  mantle: '#020617',
  crust: '#000000',
  text: '#e2e8f0',
  subtext: '#94a3b8',
  mauve: '#c084fc',
  lavender: '#a78bfa',
  teal: '#2dd4bf',
  green: '#4ade80',
  red: '#f87171',
  yellow: '#facc15',
  blue: '#60a5fa',
  surface0: '#1e293b',
  surface1: '#334155',
  surface2: '#475569',
  overlay0: '#64748b',
  rosewater: '#fb7185',
  peach: '#fdba74',
  sapphire: '#38bdf8'
};

const forest: Palette = {
  base: '#0d1117',
  mantle: '#010409',
  crust: '#000000',
  text: '#c9d1d9',
  subtext: '#8b949e',
  mauve: '#a371f7',
  lavender: '#8957e5',
  teal: '#3fb950',
  green: '#56d364',
  red: '#f85149',
  yellow: '#d29922',
  blue: '#58a6ff',
  surface0: '#161b22',
  surface1: '#21262d',
  surface2: '#30363d',
  overlay0: '#484f58',
  rosewater: '#f778ba',
  peach: '#ffa657',
  sapphire: '#79c0ff'
};

const light: Palette = {
  base: '#ffffff',
  mantle: '#f8f9fa',
  crust: '#f1f3f5',
  text: '#212529',
  subtext: '#495057',
  mauve: '#7950f2',
  lavender: '#6741d9',
  teal: '#0ca678',
  green: '#2f9e44',
  red: '#e03131',
  yellow: '#f08c00',
  blue: '#1c7ed6',
  surface0: '#e9ecef',
  surface1: '#dee2e6',
  surface2: '#ced4da',
  overlay0: '#adb5bd',
  rosewater: '#e64980',
  peach: '#fd7e14',
  sapphire: '#339af0'
};

export const neon: Palette = {
  base: '#0b0e14',
  mantle: '#0e1620',
  crust: '#060810',
  text: '#5ef2c4',
  subtext: '#9fb3c8',
  mauve: '#22d3ee',
  lavender: '#38bdf8',
  teal: '#5ef2c4',
  green: '#34d399',
  red: '#f87171',
  yellow: '#facc15',
  blue: '#22d3ee',
  surface0: '#11202a',
  surface1: '#16283a',
  surface2: '#1d3346',
  overlay0: '#3b4a5a',
  rosewater: '#fb7185',
  peach: '#fdba74',
  sapphire: '#38bdf8'
};

export const themes: Record<ThemeName, Palette> = {
  mocha: catppuccin.mocha,
  latte: catppuccin.latte,
  midnight,
  forest,
  light,
  neon
};

export function palette(theme: ThemeName): Palette {
  return themes[theme];
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
