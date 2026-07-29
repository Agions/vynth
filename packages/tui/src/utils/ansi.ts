const ESC = '\x1b';

export function cursorTo(row: number, col: number): string {
  return `${ESC}[${row};${col}H`;
}

export function cursorToXY(x: number, y: number): string {
  return `${ESC}[${y};${x}H`;
}

export const clearScreen = `${ESC}[2J${ESC}[H`;

export const eraseDown = `${ESC}[J`;

export const eraseUp = `${ESC}[1J`;

export const eraseLine = `${ESC}[2K`;

export const hideCursor = `${ESC}[?25l`;

export const showCursor = `${ESC}[?25h`;

export const enterAltScreen = `${ESC}[?1049h`;

export const leaveAltScreen = `${ESC}[?1049l`;

export const enableMouseTracking = [
  `${ESC}[?1006h`,
  `${ESC}[?1003h`,
  `${ESC}[?1005h`,
  `${ESC}[?1004h`
].join('');

export const disableMouseTracking = [
  `${ESC}[?1006l`,
  `${ESC}[?1003l`,
  `${ESC}[?1005l`,
  `${ESC}[?1004l`
].join('');
