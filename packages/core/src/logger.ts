export type LogLevel = 'debug' | 'info' | 'warn' | 'error';

const order: Record<LogLevel, number> = { debug: 0, info: 1, warn: 2, error: 3 };
let current: LogLevel = 'info';

export function setLogLevel(level: LogLevel): void {
  current = level;
}

export function log(level: LogLevel, message: string, meta?: unknown): void {
  if (order[level] < order[current]) return;
  const tag = level.toUpperCase().padEnd(5);
  const line = meta === undefined ? `[${tag}] ${message}` : `[${tag}] ${message} ${fmtMeta(meta)}`;
  if (level === 'error') console.error(line);
  else console.error(line);
}

function fmtMeta(meta: unknown): string {
  try {
    return JSON.stringify(meta);
  } catch {
    return String(meta);
  }
}
