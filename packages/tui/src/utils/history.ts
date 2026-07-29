import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const HISTORY_FILE = 'history.json';
const DEFAULT_MAX = 500;

export function historyPath(dataDir: string): string {
  return join(dataDir, HISTORY_FILE);
}

export function loadHistory(dataDir: string, max = DEFAULT_MAX): string[] {
  try {
    const p = historyPath(dataDir);
    if (!existsSync(p)) return [];
    const raw = JSON.parse(readFileSync(p, 'utf8'));
    if (!Array.isArray(raw)) return [];
    const items = raw.filter((x): x is string => typeof x === 'string');
    const dedup: string[] = [];
    for (const it of items) {
      if (dedup[dedup.length - 1] !== it) dedup.push(it);
    }
    return dedup.slice(-max);
  } catch {
    return [];
  }
}

export function saveHistory(dataDir: string, entries: string[], max = DEFAULT_MAX): void {
  try {
    mkdirSync(dataDir, { recursive: true });
    const dedup: string[] = [];
    for (const it of entries) {
      if (dedup[dedup.length - 1] !== it) dedup.push(it);
    }
    const trimmed = dedup.slice(-max);
    writeFileSync(historyPath(dataDir), JSON.stringify(trimmed), 'utf8');
  } catch {}
}
