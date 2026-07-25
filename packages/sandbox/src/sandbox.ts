import { spawn } from 'node:child_process';
import { realpathSync } from 'node:fs';
import { readFile, writeFile } from 'node:fs/promises';
import { basename, dirname, join, resolve, sep } from 'node:path';
import { SandboxError, type ToolResult } from '@vynth/core';

function safeResolve(target: string, cwd: string): string {
  const abs = resolve(cwd, target);
  if (abs !== cwd && !abs.startsWith(cwd + sep)) {
    throw new SandboxError(`path escapes sandbox: ${target}`, 'VC-030001');
  }
  // Re-check after resolving symlinks: a cwd-internal symlink may point outside.
  const realCwd = realpathSync(cwd);
  let realAbs: string;
  try {
    realAbs = realpathSync(abs);
  } catch {
    // Target does not exist yet (e.g. a write) — resolve its parent instead.
    realAbs = join(realpathSync(dirname(abs)), basename(abs));
  }
  if (realAbs !== realCwd && !realAbs.startsWith(realCwd + sep)) {
    throw new SandboxError(`path escapes sandbox via symlink: ${target}`, 'VC-030002');
  }
  return realAbs;
}

export async function readText(path: string, cwd: string): Promise<ToolResult> {
  try {
    const abs = safeResolve(path, cwd);
    const content = await readFile(abs, 'utf8');
    return { ok: true, output: content };
  } catch (err) {
    return { ok: false, output: '', error: formatErr(err) };
  }
}

export async function writeText(path: string, content: string, cwd: string): Promise<ToolResult> {
  try {
    const abs = safeResolve(path, cwd);
    await writeFile(abs, content, 'utf8');
    return { ok: true, output: `wrote ${abs}` };
  } catch (err) {
    return { ok: false, output: '', error: formatErr(err) };
  }
}

export interface RunOpts {
  cwd: string;
  networkAllowed?: boolean;
  timeoutMs?: number;
}

export async function runCommand(command: string, opts: RunOpts): Promise<ToolResult> {
  const timeout = opts.timeoutMs ?? 30_000;
  if (!opts.networkAllowed) {
    return { ok: false, output: '', error: '[VC-030003] network blocked by sandbox policy' };
  }
  return new Promise<ToolResult>((resolveResult) => {
    const shell = process.platform === 'win32' ? 'cmd' : 'sh';
    const args = process.platform === 'win32' ? ['/c', command] : ['-c', command];
    const proc = spawn(shell, args, { cwd: opts.cwd });
    let out = '';
    let errOut = '';
    const timer = setTimeout(() => {
      proc.kill('SIGKILL');
      resolveResult({ ok: false, output: out, error: `[VC-030004] timeout after ${timeout}ms` });
    }, timeout);

    proc.stdout.on('data', (d) => {
      out += String(d);
    });
    proc.stderr.on('data', (d) => {
      errOut += String(d);
    });
    proc.on('close', (code) => {
      clearTimeout(timer);
      const output = out + (errOut ? `\n[stderr]\n${errOut}` : '');
      resolveResult({
        ok: code === 0,
        output,
        error: code === 0 ? undefined : `[VC-030005] exit ${code}`
      });
    });
    proc.on('error', (e) => {
      clearTimeout(timer);
      resolveResult({ ok: false, output: '', error: formatErr(e) });
    });
  });
}

function errMsg(err: unknown): string {
  return err instanceof Error ? err.message : String(err);
}

/** 把已知 VynthError 带上 6 位码前缀（其它错误原样） */
function formatErr(err: unknown): string {
  if (err && typeof err === 'object' && 'numericCode' in err && 'message' in err) {
    const e = err as { numericCode?: string; message?: string };
    if (e.numericCode && /VC-\d{6}/.test(e.numericCode)) {
      return `[${e.numericCode}] ${e.message ?? ''}`.trim();
    }
  }
  return errMsg(err);
}
