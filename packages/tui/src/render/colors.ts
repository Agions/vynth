export interface ColorEnv {
  COLORTERM?: string;
  TERM?: string;
  TERM_PROGRAM?: string;
}

export function supportsTruecolor(env: ColorEnv = process.env as ColorEnv): boolean {
  const colorterm = (env.COLORTERM ?? '').toLowerCase();
  if (colorterm.includes('truecolor') || colorterm.includes('24bit')) return true;

  const term = (env.TERM ?? '').toLowerCase();
  if (term.endsWith('-direct')) return true;

  const program = env.TERM_PROGRAM ?? '';
  const knownTruecolor = new Set([
    'iTerm.app',
    'WezTerm',
    'vscode',
    'ghostty',
    'Hyper',
    'Tabby',
    'kitty',
    'Alacritty',
    'rio'
  ]);
  if (knownTruecolor.has(program)) return true;

  return false;
}

export function parseHex(hex: string): [number, number, number] | null {
  const m = /^#?([0-9a-fA-F]{6})$/.exec(hex.trim());
  if (!m) return null;
  const n = Number.parseInt(m[1], 16);
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
}

const CUBE_LEVELS = [0, 95, 135, 175, 215, 255];

function nearestCubeLevel(v: number): number {
  let best = 0;
  let bestDist = Number.POSITIVE_INFINITY;
  for (let i = 0; i < CUBE_LEVELS.length; i++) {
    const d = Math.abs(CUBE_LEVELS[i] - v);
    if (d < bestDist) {
      bestDist = d;
      best = i;
    }
  }
  return best;
}

export function hexToAnsi256(hex: string): number {
  const rgb = parseHex(hex);
  if (!rgb) return 7;
  const [r, g, b] = rgb;

  const ri = nearestCubeLevel(r);
  const gi = nearestCubeLevel(g);
  const bi = nearestCubeLevel(b);
  const cubeIdx = 16 + 36 * ri + 6 * gi + bi;
  const cubeDist =
    (CUBE_LEVELS[ri] - r) ** 2 + (CUBE_LEVELS[gi] - g) ** 2 + (CUBE_LEVELS[bi] - b) ** 2;

  const gray = Math.round((r + g + b) / 3);
  let grayN = Math.round((gray - 8) / 10);
  if (grayN < 0) grayN = 0;
  if (grayN > 23) grayN = 23;
  const grayLevel = 8 + 10 * grayN;
  const grayDist = (grayLevel - r) ** 2 + (grayLevel - g) ** 2 + (grayLevel - b) ** 2;

  return grayDist < cubeDist ? 232 + grayN : cubeIdx;
}

export class ColorRegistry {
  private hexToIdx = new Map<string, number>();
  private idxToHex: string[] = [''];
  private truecolor: boolean;

  constructor(opts: { truecolor?: boolean } = {}) {
    this.truecolor = opts.truecolor ?? supportsTruecolor();
  }

  index(hex: string): number {
    const key = hex.toLowerCase();
    const existing = this.hexToIdx.get(key);
    if (existing !== undefined) return existing;
    if (this.idxToHex.length >= 256) {
      return 0;
    }
    const idx = this.idxToHex.length;
    this.idxToHex.push(key);
    this.hexToIdx.set(key, idx);
    return idx;
  }

  ansiFg(idx: number): string {
    const hex = this.idxToHex[idx];
    if (!hex) return '';
    if (this.truecolor) {
      const rgb = parseHex(hex);
      if (!rgb) return '';
      return `\x1b[38;2;${rgb[0]};${rgb[1]};${rgb[2]}m`;
    }
    return `\x1b[38;5;${hexToAnsi256(hex)}m`;
  }

  ansiBg(idx: number): string {
    const hex = this.idxToHex[idx];
    if (!hex) return '';
    if (this.truecolor) {
      const rgb = parseHex(hex);
      if (!rgb) return '';
      return `\x1b[48;2;${rgb[0]};${rgb[1]};${rgb[2]}m`;
    }
    return `\x1b[48;5;${hexToAnsi256(hex)}m`;
  }

  get size(): number {
    return this.idxToHex.length - 1;
  }

  get isTruecolor(): boolean {
    return this.truecolor;
  }
}
