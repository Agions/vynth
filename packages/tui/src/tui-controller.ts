import type { ZenoConfig } from '@zeno/core';
import * as sandbox from '@zeno/sandbox';
import { SLASH_COMMANDS, matchSlashCommands } from './slash-commands';
import type { Store } from './state/Store';
import { saveHistory } from './utils/history';
import { getTaskManager } from './utils/tasks';

export async function scanProjectFiles(cwd: string): Promise<string[]> {
  const { readdir } = await import('node:fs/promises');
  const { join, relative } = await import('node:path');
  const files: string[] = [];
  const IGNORED = new Set([
    'node_modules',
    '.git',
    '.hg',
    '.svn',
    'dist',
    'build',
    'out',
    'coverage',
    '.next',
    '.turbo',
    '.cache',
    '__pycache__',
    '.venv',
    'venv',
    'target'
  ]);

  async function walk(dir: string, depth: number): Promise<void> {
    if (depth > 8 || files.length > 3000) return;
    let entries: Array<import('node:fs').Dirent>;
    try {
      entries = await readdir(dir, { withFileTypes: true });
    } catch {
      return;
    }
    for (const ent of entries) {
      if (files.length > 3000) break;
      if (ent.isDirectory()) {
        if (!IGNORED.has(ent.name)) {
          await walk(join(dir, ent.name), depth + 1);
        }
      } else {
        const rel = relative(cwd, join(dir, ent.name));
        files.push(rel);
      }
    }
  }

  await walk(cwd, 0);
  return files.sort();
}

export function parseSGRMouse(
  buffer: string
): { event: { button: number; row: number; col: number }; rest: string } | null {
  const ESC = '\x1b';
  const re = new RegExp(`${ESC}\\[<(\\d+);(\\d+);(\\d+)([mM])`);
  const m = re.exec(buffer);
  if (!m) return null;
  const button = Number(m[1]);
  const col = Number(m[2]);
  const row = Number(m[3]);
  const rest = buffer.slice(0, m.index) + buffer.slice(m.index + m[0].length);
  return { event: { button, row, col }, rest };
}

export function isMouseOrEscapeGarbage(str: string, key?: { name?: string }): boolean {
  if (!str) return false;
  if (/\d+;\d+/.test(str)) return true;
  if (/^<[0-9;]+[mM]?$/.test(str) || /^<\d+/.test(str) || /;\d+[mM]?$/.test(str)) return true;
  if (str.includes('\x1b') || str.includes('\x1b[')) return true;
  if (key && (key.name === 'escape' || key.name === 'undefined')) {
    if (/[0-9;<=>]+[mM]?/.test(str) && str.includes(';')) return true;
  }
  if (/^[0-9;<>]+[mM]$/.test(str) && str.includes(';')) return true;
  return false;
}

export function isPhysicalEscapeKey(str: string | null, key?: { name?: string }): boolean {
  if (!key || key.name !== 'escape') return false;
  if (!str) return true;
  return str === '\x1b';
}

export async function executeDirectCommand(
  raw: string,
  config: ZenoConfig,
  onSystemMessage: (msg: string) => void,
  onResult: (r: {
    id: string;
    command: string;
    status: 'done' | 'failed';
    output: string;
    exitCode: number | null;
  }) => void
): Promise<void> {
  if (!raw) {
    onSystemMessage('用法: ! <命令>   （命令末尾加 & 可后台运行，/tasks 查看）');
    return;
  }
  const background = raw.endsWith(' &');
  const cmd = background ? raw.slice(0, -2).trim() : raw;
  if (!cmd) {
    onSystemMessage('用法: ! <命令>   （命令末尾加 & 可后台运行，/tasks 查看）');
    return;
  }

  if (background) {
    const mgr = getTaskManager();
    const task = mgr.spawn(cmd, config.sandbox.cwd, config.sandbox.networkAllowed ?? true);
    onSystemMessage(`⚙ 后台任务已启动: ${task.id}  ·  /tasks 查看`);
    return;
  }

  const res = await sandbox.runCommand(cmd, {
    cwd: config.sandbox.cwd,
    networkAllowed: config.sandbox.networkAllowed ?? true,
    harden: config.sandbox.harden ?? false
  });
  onResult({
    id: `direct-${Date.now()}`,
    command: cmd,
    status: res.ok ? 'done' : 'failed',
    output: res.output,
    exitCode: res.ok ? 0 : 1
  });
}
