
const ESC = '\x1b';

export function setScrollRegion(
  topEnd: number,
  botStart: number,
  rows: number
): { top: string; mid: string; bot: string } {
  const top = `${ESC}[1;${topEnd}r`;
  const mid = `${ESC}[${topEnd + 1};${botStart - 1}r`;
  const bot = `${ESC}[${botStart};${rows}r`;
  return { top, mid, bot };
}

export const resetScrollRegion = `${ESC}[r`;

export const cursorHome = `${ESC}[H`;
export const cursorTo = (row: number, col: number): string => `${ESC}[${row};${col}H`;
