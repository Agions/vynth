import { spawn } from 'node:child_process';
import { SandboxError } from '@zeno/core';

export interface HardenSpec {
  command: string;
  cwd: string;
  networkAllowed: boolean;
  timeoutMs: number;
  onStdout: (chunk: string) => void;
  onStderr: (chunk: string) => void;
}

export async function detectHardenBackend(): Promise<boolean> {
  if (process.platform === 'darwin') {
    if (!(await pathExists('/usr/bin/sandbox-exec'))) return false;
    return await runTestCommand('/usr/bin/sandbox-exec', [
      '-p',
      '(version 1) (allow process-exec)',
      '/bin/echo',
      'ok'
    ]);
  }
  if (process.platform === 'linux') {
    if (!(await commandExists('bwrap'))) return false;
    return await runTestCommand('bwrap', ['--unshare-user-try', '/bin/echo', 'ok']);
  }
  return false;
}

function runTestCommand(cmd: string, args: string[]): Promise<boolean> {
  return new Promise((resolve) => {
    try {
      const proc = spawn(cmd, args, { stdio: 'ignore' });
      proc.on('close', (code) => resolve(code === 0));
      proc.on('error', () => resolve(false));
    } catch {
      resolve(false);
    }
  });
}

function pathExists(p: string): Promise<boolean> {
  return new Promise((resolve) => {
    const proc = spawn('/bin/sh', ['-c', `test -e "${p}"`], { stdio: 'ignore' });
    proc.on('close', (code) => resolve(code === 0));
  });
}

function commandExists(cmd: string): Promise<boolean> {
  return new Promise((resolve) => {
    const proc = spawn('/bin/sh', ['-c', `command -v ${cmd} >/dev/null 2>&1`], { stdio: 'ignore' });
    proc.on('close', (code) => resolve(code === 0));
  });
}

export function buildSbplProfile(
  spec: Pick<HardenSpec, 'cwd' | 'networkAllowed' | 'command'>
): string {
  const cwd = spec.cwd;
  const net = spec.networkAllowed
    ? '(allow network-outbound)\n(allow network-inbound)'
    : '(deny network-outbound)\n(deny network-inbound)';
  return `(version 1)
(deny default)
(allow process-exec)
(allow process-fork)
(allow signal)
(allow sysctl-read)
(allow mach-lookup)
(allow file-read* file-write* file-ioctl)
(allow file-read*
  (subpath "/usr")
  (subpath "/bin")
  (subpath "/private/var")
  (subpath "/dev")
  (subpath "/etc")
  (subpath "/tmp")
  (subpath "/var")
  (subpath "${cwd}")
  (literal "/dev/null")
  (literal "/dev/tty")
  (literal "/dev/stdin")
  (literal "/dev/stdout")
  (literal "/dev/stderr")
)
${net}
`;
}

export function buildBwrapArgs(
  spec: Pick<HardenSpec, 'cwd' | 'networkAllowed' | 'command'>
): string[] {
  const args = [
    '--unshare-user-try',
    '--unshare-pid',
    '--unshare-uts',
    '--die-with-parent',
    '--cap-drop',
    'ALL',
    '--new-session',
    '--dir',
    '/tmp',
    '--dir',
    '/var',
    '--proc',
    '/proc',
    '--dev',
    '/dev',
    '--ro-bind',
    '/usr',
    '/usr',
    '--ro-bind',
    '/bin',
    '/bin',
    '--ro-bind',
    '/etc/resolv.conf',
    '/etc/resolv.conf',
    '--ro-bind',
    '/etc/hosts',
    '/etc/hosts',
    '--ro-bind',
    '/etc/ssl',
    '/etc/ssl',
    '--bind',
    spec.cwd,
    spec.cwd,
    '--chdir',
    spec.cwd
  ];
  if (!spec.networkAllowed) {
    args.push('--unshare-net');
  }
  args.push('/bin/sh', '-c', spec.command);
  return args;
}

export function spawnHardened(spec: HardenSpec): ReturnType<typeof spawn> {
  if (process.platform === 'darwin') {
    const profile = buildSbplProfile({
      cwd: spec.cwd,
      networkAllowed: spec.networkAllowed,
      command: spec.command
    });
    return spawn('/usr/bin/sandbox-exec', ['-p', profile, '/bin/sh', '-c', spec.command], {
      cwd: spec.cwd
    });
  }
  if (process.platform === 'linux') {
    const args = buildBwrapArgs({
      cwd: spec.cwd,
      networkAllowed: spec.networkAllowed,
      command: spec.command
    });
    return spawn('bwrap', args, { cwd: spec.cwd });
  }
  throw new SandboxError(
    `OS 级硬隔离在 ${process.platform} 上未实现（不支持 sandbox-exec / bwrap）`,
    'VC-030006'
  );
}
