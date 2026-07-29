
export function hexToRgb(hex: string): string {
  const n = Number.parseInt((hex || '#ffffff').replace('#', ''), 16);
  return `${(n >> 16) & 255};${(n >> 8) & 255};${n & 255}`;
}

export function ansiBackground(hex: string): string {
  return `\x1b[48;2;${hexToRgb(hex)}m`;
}

export function ansiForeground(hex: string): string {
  return `\x1b[38;2;${hexToRgb(hex)}m`;
}

export function lerpHex(a: string, b: string, t: number): string {
  const na = Number.parseInt((a || '#ffffff').replace('#', ''), 16);
  const nb = Number.parseInt((b || '#000000').replace('#', ''), 16);
  const lerp = (ca: number, cb: number) => Math.round(ca + (cb - ca) * t);
  const r = lerp((na >> 16) & 255, (nb >> 16) & 255);
  const g = lerp((na >> 8) & 255, (nb >> 8) & 255);
  const bVal = lerp(na & 255, nb & 255);
  return `#${r.toString(16).padStart(2, '0')}${g.toString(16).padStart(2, '0')}${bVal.toString(16).padStart(2, '0')}`;
}
