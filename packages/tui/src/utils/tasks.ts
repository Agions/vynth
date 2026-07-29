
import { spawn } from 'node:child_process';

export type TaskStatus = 'running' | 'done' | 'failed';

export interface BackgroundTask {
  id: string;
  command: string;
  status: TaskStatus;
  output: string;
  exitCode: number | null;
  startedAt: number;
  finishedAt: number | null;
}

class TaskManager {
  private tasks = new Map<string, BackgroundTask>();
  private listeners = new Set<() => void>();

  spawn(command: string, cwd: string, networkAllowed: boolean): BackgroundTask {
    const id = `task-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`;
    const task: BackgroundTask = {
      id,
      command,
      status: 'running',
      output: '',
      exitCode: null,
      startedAt: Date.now(),
      finishedAt: null
    };
    this.tasks.set(id, task);

    if (!networkAllowed) {
      task.status = 'failed';
      task.output = '[VC-030003] network blocked by sandbox policy';
      task.exitCode = 1;
      task.finishedAt = Date.now();
      this.emit();
      return task;
    }

    const shell = process.platform === 'win32' ? 'cmd' : 'sh';
    const args = process.platform === 'win32' ? ['/c', command] : ['-c', command];
    let proc: ReturnType<typeof spawn> | null = null;
    try {
      proc = spawn(shell, args, { cwd, detached: false });
    } catch (e) {
      task.status = 'failed';
      task.output = e instanceof Error ? e.message : String(e);
      task.exitCode = 1;
      task.finishedAt = Date.now();
      this.emit();
      return task;
    }

    proc.stdout?.on('data', (d: Buffer) => {
      task.output += String(d);
    });
    proc.stderr?.on('data', (d: Buffer) => {
      task.output += String(d);
    });
    proc.on('close', (code: number | null) => {
      task.status = code === 0 ? 'done' : 'failed';
      task.exitCode = code;
      task.finishedAt = Date.now();
      this.emit();
    });
    proc.on('error', (e: Error) => {
      task.status = 'failed';
      task.output += `\n${e.message}`;
      task.exitCode = 1;
      task.finishedAt = Date.now();
      this.emit();
    });

    return task;
  }

  list(): BackgroundTask[] {
    return [...this.tasks.values()].sort((a, b) => b.startedAt - a.startedAt);
  }

  get(id: string): BackgroundTask | undefined {
    return this.tasks.get(id);
  }

  runningCount(): number {
    let n = 0;
    for (const t of this.tasks.values()) if (t.status === 'running') n++;
    return n;
  }

  onChange(fn: () => void): () => void {
    this.listeners.add(fn);
    return () => {
      this.listeners.delete(fn);
    };
  }

  private emit(): void {
    for (const fn of this.listeners) fn();
  }
}

let _mgr: TaskManager | null = null;

export function getTaskManager(): TaskManager {
  if (!_mgr) _mgr = new TaskManager();
  return _mgr;
}
