import { spawn } from 'node:child_process';
import { realpathSync } from 'node:fs';
import { readFile, writeFile } from 'node:fs/promises';
import { basename, dirname, join, resolve, sep } from 'node:path';
import { SandboxError, type ToolResult, audit } from '@vynth/core';
import { detectHardenBackend, spawnHardened } from './harden';

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
    audit().record('file_access', { op: 'read', path: abs, ok: true }, true);
    return { ok: true, output: content };
  } catch (err) {
    audit().record('file_access', { op: 'read', path, ok: false }, false);
    return { ok: false, output: '', error: formatErr(err) };
  }
}

export async function writeText(path: string, content: string, cwd: string): Promise<ToolResult> {
  try {
    const abs = safeResolve(path, cwd);
    await writeFile(abs, content, 'utf8');
    audit().record('file_access', { op: 'write', path: abs, ok: true }, true);
    return { ok: true, output: `wrote ${abs}` };
  } catch (err) {
    audit().record('file_access', { op: 'write', path, ok: false }, false);
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
    audit().record('network_egress', { command, allowed: false, ok: false }, false);
    return { ok: false, output: '', error: '[VC-030003] network blocked by sandbox policy' };
  }
  audit().record('network_egress', { command, allowed: true, ok: true }, true);
  const harden = process.env.VYNTH_HARDEN === '1';
  // 硬化路径在 spawn 前先探测后端：缺失/不可用 → VC-030006，不静默降级
  if (harden) {
    const ok = await detectHardenBackend();
    if (!ok) {
      return {
        ok: false,
        output: '',
        error: formatErr(
          new SandboxError(
            `OS 级硬隔离后端不可用（platform=${process.platform}）；VYNTH_HARDEN=1 但 sandbox-exec/bwrap 无法 apply`,
            'VC-030006'
          )
        )
      };
    }
  }
  return new Promise<ToolResult>((resolveResult) => {
    let proc: ReturnType<typeof spawn> | null = null;
    let hardened = false;
    try {
      if (harden) {
        proc = spawnHardened({
          command,
          cwd: opts.cwd,
          networkAllowed: true,
          timeoutMs: timeout,
          onStdout: (chunk) => {
            /* collected below */
          },
          onStderr: (chunk) => {
            /* collected below */
          }
        });
        hardened = true;
      } else {
        const shell = process.platform === 'win32' ? 'cmd' : 'sh';
        const args = process.platform === 'win32' ? ['/c', command] : ['-c', command];
        proc = spawn(shell, args, { cwd: opts.cwd });
      }
    } catch (e) {
      // 硬化路径上 spawnHardened 抛错（不支持的平台 / 策略非法）→ VC-030006
      if (harden) {
        resolveResult({
          ok: false,
          output: '',
          error: formatErr(
            new SandboxError(
              `OS 级硬隔离后端不可用: ${e instanceof Error ? e.message : String(e)}`,
              'VC-030006'
            )
          )
        });
      } else {
        resolveResult({ ok: false, output: '', error: formatErr(e) });
      }
      return;
    }
    if (!proc) {
      resolveResult({ ok: false, output: '', error: '[VC-030005] spawn returned no process' });
      return;
    }
    let out = '';
    let errOut = '';
    const timer = setTimeout(() => {
      proc?.kill('SIGKILL');
      resolveResult({ ok: false, output: out, error: `[VC-030004] timeout after ${timeout}ms` });
    }, timeout);

    proc.stdout?.on('data', (d) => {
      out += String(d);
    });
    proc.stderr?.on('data', (d) => {
      errOut += String(d);
    });
    proc.on('close', (code) => {
      clearTimeout(timer);
      const output = out + (errOut ? `\n[stderr]\n${errOut}` : '');
      if (code === 0) {
        resolveResult({ ok: true, output });
      } else {
        resolveResult({
          ok: false,
          output,
          error: hardened ? `[VC-030005] exit ${code} (hardened)` : `[VC-030005] exit ${code}`
        });
      }
    });
    proc.on('error', (e) => {
      clearTimeout(timer);
      if (hardened) {
        resolveResult({
          ok: false,
          output: '',
          error: formatErr(
            new SandboxError(`硬化启动失败: ${e.message}（后端=${process.platform}）`, 'VC-030007')
          )
        });
      } else {
        resolveResult({ ok: false, output: '', error: formatErr(e) });
      }
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
