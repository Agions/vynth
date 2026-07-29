import { spawn } from 'node:child_process';
import { existsSync, realpathSync } from 'node:fs';
import { readFile, writeFile } from 'node:fs/promises';
import { basename, dirname, join, resolve, sep } from 'node:path';
import { SandboxError, type ToolResult, audit, formatZenoError } from '@zeno/core';
import { detectHardenBackend, spawnHardened } from './harden';

function resolveExistingAncestor(abs: string): { realAncestor: string; tail: string } {
  let probe = abs;
  const stack: string[] = [];
  while (!existsSync(probe)) {
    stack.unshift(basename(probe));
    const parent = dirname(probe);
    if (parent === probe) break;
    probe = parent;
  }
  const realAncestor = realpathSync(probe);
  const tail = stack.length > 0 ? join(...stack) : '';
  return { realAncestor, tail };
}

function safeResolve(target: string, cwd: string): string {
  const abs = resolve(cwd, target);
  if (abs !== cwd && !abs.startsWith(cwd + sep)) {
    throw new SandboxError(`path escapes sandbox: ${target}`, 'VC-030001');
  }
  // Re-check after resolving symlinks: a cwd-internal symlink may point outside.
  const realCwd = realpathSync(cwd);
  let realAbs: string;
  if (existsSync(abs)) {
    realAbs = realpathSync(abs);
  } else {
    // Target (and possibly several parent dirs) does not exist yet — resolve the
    // nearest existing ancestor, then re-append the unresolved tail.
    const { realAncestor, tail } = resolveExistingAncestor(abs);
    realAbs = tail ? join(realAncestor, tail) : realAncestor;
  }
  if (realAbs !== realCwd && !realAbs.startsWith(realCwd + sep)) {
    throw new SandboxError(`path escapes sandbox via symlink: ${target}`, 'VC-030002');
  }
  return realAbs;
}

export function resolveInSandbox(target: string, cwd: string): string {
  return safeResolve(target, cwd);
}

function hardenUnavailableMessage(): string {
  const platform = process.platform;
  if (platform === 'darwin') {
    const isRoot = typeof process.getuid === 'function' && process.getuid() === 0;
    if (isRoot) {
      return 'OS 级硬隔离后端不可用（platform=darwin）：即便以 root 运行，当前 macOS 版本仍可能拒绝 sandbox-exec apply，请改用 Linux（bwrap）获得真正的 OS 级隔离';
    }
    return 'OS 级硬隔离后端不可用（platform=darwin）：macOS 15+ 已禁止非 root 进程 apply 任意 seatbelt 策略（连纯允许策略也报 Operation not permitted）。要启用 OS 级隔离请以 root 运行 Vynth（不推荐——AI 代理以 root 运行风险更高），或改用 Linux（bubblewrap）；若不需要 OS 级隔离，移除 VYNTH_HARDEN=1 / sandbox.harden 即可';
  }
  if (platform === 'linux') {
    return 'OS 级硬隔离后端不可用（platform=linux）：未检测到 bubblewrap（bwrap）。请先 `apt install bubblewrap`（或等价命令）再试；若不需要 OS 级隔离，移除 VYNTH_HARDEN=1 / sandbox.harden';
  }
  return `OS 级硬隔离后端不可用（platform=${platform}）：sandbox-exec / bwrap 均不支持该平台，开启时拒绝执行（Fail-Closed）`;
}

export async function readText(path: string, cwd: string): Promise<ToolResult> {
  try {
    const abs = safeResolve(path, cwd);
    const content = await readFile(abs, 'utf8');
    audit().record('file_access', { op: 'read', path: abs, ok: true }, true);
    return { ok: true, output: content };
  } catch (err) {
    audit().record('file_access', { op: 'read', path, ok: false }, false);
    return { ok: false, output: '', error: formatZenoError(err) };
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
    return { ok: false, output: '', error: formatZenoError(err) };
  }
}

export interface RunOpts {
  cwd: string;
  networkAllowed?: boolean;
  timeoutMs?: number;
  harden?: boolean;
}

export async function runCommand(command: string, opts: RunOpts): Promise<ToolResult> {
  const timeout = opts.timeoutMs ?? 30_000;
  if (!opts.networkAllowed) {
    audit().record('network_egress', { command, allowed: false, ok: false }, false);
    return { ok: false, output: '', error: '[VC-030003] network blocked by sandbox policy' };
  }
  audit().record('network_egress', { command, allowed: true, ok: true }, true);
  const harden = opts.harden ?? process.env.VYNTH_HARDEN === '1';
  if (harden) {
    const ok = await detectHardenBackend();
    if (!ok) {
      return {
        ok: false,
        output: '',
        error: formatZenoError(new SandboxError(hardenUnavailableMessage(), 'VC-030006'))
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
      if (harden) {
        resolveResult({
          ok: false,
          output: '',
          error: formatZenoError(
            new SandboxError(
              `OS 级硬隔离后端不可用: ${e instanceof Error ? e.message : String(e)}`,
              'VC-030006'
            )
          )
        });
      } else {
        resolveResult({ ok: false, output: '', error: formatZenoError(e) });
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
          error: formatZenoError(
            new SandboxError(`硬化启动失败: ${e.message}（后端=${process.platform}）`, 'VC-030007')
          )
        });
      } else {
        resolveResult({ ok: false, output: '', error: formatZenoError(e) });
      }
    });
  });
}
