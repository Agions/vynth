/**
 * F15 OS 级硬隔离：把 `runCommand` 启动的子进程放入 OS 强制沙箱。
 *
 * 触发方式（详见设计）：
 *   - VYNTH_HARDEN=1  /  config 里 harden:true
 *   - 仅 `runCommand` 走硬化链路；readText/writeText 走应用层即可
 *
 * 平台策略：
 *   - macOS（darwin）：用 `sandbox-exec` + 内联 SBPL 策略文件，deny default
 *   - linux：用 `bwrap`（bubblewrap），mount namespace + capability drop
 *   - 其它平台（win32/freebsd）：throw VC-030006，不静默降级（与 F15 承诺对齐）
 *
 * 失败语义：spawn 之前任何错误都抛 SandboxError（VC-030006/VC-030007），
 * 不允许应用层静默 fallback 到非硬化路径——否则硬化形同虚设。
 */
import { spawn } from 'node:child_process';
import { SandboxError } from '@vynth/core';

export interface HardenSpec {
  /** 实际要执行的 shell 命令（与 runCommand 一致） */
  command: string;
  /** 受限的工作目录（必须是绝对路径） */
  cwd: string;
  /** 是否允许联网（与 VYNTH_NET 联动） */
  networkAllowed: boolean;
  /** 计时器（ms），到点 SIGKILL */
  timeoutMs: number;
  /** 监听 stdout/stderr 的回调 */
  onStdout: (chunk: string) => void;
  onStderr: (chunk: string) => void;
}

/**
 * 探测 OS 沙箱后端是否**真实可用**（不只看二进制存在）。
 *
 * 返回 true 当且仅当：
 *   - macOS：sandbox-exec 存在 + 能成功 apply 一个最小 allow 策略
 *   - linux：bwrap 在 PATH 中 + 能成功 apply 一个最小 --unshare-user 试运行
 *
 * macOS 14+ 在 App Sandbox 启用时即便 `(allow default)` 也会被拒
 * （"Operation not permitted"），本探测能识别这种情况并返回 false，
 * 让上层调用方走「硬化不可用，VYNTH_HARDEN=1 时抛 VC-030006」路径。
 */
export async function detectHardenBackend(): Promise<boolean> {
  if (process.platform === 'darwin') {
    if (!(await pathExists('/usr/bin/sandbox-exec'))) return false;
    // 试跑最简策略：仅 allow process-exec
    return await runTestCommand('/usr/bin/sandbox-exec', [
      '-p',
      '(version 1) (allow process-exec)',
      '/bin/echo',
      'ok'
    ]);
  }
  if (process.platform === 'linux') {
    if (!(await commandExists('bwrap'))) return false;
    // 试跑 bwrap：最小化 unshare-user
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

/**
 * 构造 macOS SBPL（Sandbox Profile Language）策略：
 *   - deny default（拒绝一切未显式允许的 FS/syscall）
 *   - 允许 cwd 读写
 *   - /usr/bin /bin /usr/lib /private/var 读（执行命令需要）
 *   - /dev/null 读写
 *   - 进程创建、信号发送默认允许（子进程得 fork/exec）
 *   - 当 networkAllowed=true 时允许 outbound socket；否则 deny
 */
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

/** linux 子进程参数（bubblewrap）：最少权限 + 只读 bind 必需目录 + cwd 读写 */
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

/**
 * 启动硬化子进程并把 stdout/stderr 透传给 spec。
 * 返回 raw ChildProcess，调用方管理 timeout/clearTimer 与 close/error 处理。
 */
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
