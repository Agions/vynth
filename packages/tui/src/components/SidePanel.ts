
import type { TuiState } from '../state/TuiState';
import { fg, reset } from '../theme';
import type { BackgroundTask } from '../utils/tasks';
import { padToWidth, truncateVisible, visibleWidth } from '../utils/unicode';

const SPINNER_FRAMES = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

export type SidebarTab = 'files' | 'tasks' | 'tools';

export interface SidePanelProps {
  state: TuiState;
  width: number;
  height: number;
  files: string[];
  tasks: BackgroundTask[];
  toolNames: string[];
}

export function SidePanel(props: SidePanelProps): string[] {
  const { state, width, height } = props;
  const c = state.palette;
  const innerW = Math.max(10, width - 2);
  const border = fg(c.overlay0 ?? c.subtext);

  const row = (content: string): string => {
    const clipped = truncateVisible(content, innerW);
    return `${border}│${reset} ${padToWidth(clipped, innerW)}`;
  };

  const lines: string[] = [];

  const tabs: Array<[SidebarTab, string]> = [
    ['files', '文件'],
    ['tasks', '任务'],
    ['tools', '工具']
  ];
  const tabStr = tabs
    .map(([id, label]) =>
      id === state.sidebarTab
        ? `${fg(c.mauve)}\x1b[1m▎${label}${reset}`
        : `${fg(c.subtext)} ${label}${reset}`
    )
    .join('  ');
  lines.push(row(tabStr));
  lines.push(`${border}├${'─'.repeat(Math.max(1, width - 1))}${reset}`);

  const bodyH = Math.max(1, height - 3);
  const body: string[] = [];

  if (state.sidebarTab === 'files') {
    const files = props.files;
    if (files.length === 0) {
      body.push(`${fg(c.subtext)}扫描中…${reset}`);
    } else {
      for (const f of files.slice(0, bodyH - (files.length > bodyH ? 1 : 0))) {
        const slash = f.lastIndexOf('/');
        const dir = slash >= 0 ? f.slice(0, slash + 1) : '';
        const base = slash >= 0 ? f.slice(slash + 1) : f;
        body.push(`${fg(c.blue)}${dir}${fg(c.text)}${base}${reset}`);
      }
      if (files.length > bodyH) {
        body.push(`${fg(c.subtext)}… 共 ${files.length} 个文件${reset}`);
      }
    }
  } else if (state.sidebarTab === 'tasks') {
    const tasks = props.tasks;
    if (tasks.length === 0) {
      body.push(`${fg(c.subtext)}暂无后台任务${reset}`);
      body.push('');
      body.push(`${fg(c.subtext)}用 ${fg(c.yellow)}! cmd &${fg(c.subtext)} 启动${reset}`);
    } else {
      const frame = SPINNER_FRAMES[(state.spinnerFrame || 0) % SPINNER_FRAMES.length];
      for (const t of tasks) {
        if (body.length + 2 > bodyH) {
          body.push(`${fg(c.subtext)}… 共 ${tasks.length} 个任务${reset}`);
          break;
        }
        const st =
          t.status === 'running'
            ? `${fg(c.yellow)}${frame} running${reset}`
            : t.status === 'done'
              ? `${fg(c.green)}✔ done${reset}`
              : `${fg(c.red)}✖ failed · exit ${t.exitCode ?? '?'}${reset}`;
        body.push(`${fg(c.text)}${t.command}${reset}`);
        body.push(`  ${st}`);
      }
    }
  } else {
    // tools
    if (props.toolNames.length === 0) {
      body.push(`${fg(c.subtext)}无已注册工具${reset}`);
    } else {
      for (const name of props.toolNames.slice(0, bodyH)) {
        const isLive = state.currentTool === name;
        const badge = isLive
          ? `${fg(c.yellow)}● live${reset}`
          : `${fg(c.green)}● on${reset}`;
        const nameStr = `${fg(isLive ? c.yellow : c.text)}${name}${reset}`;
        const gap = Math.max(1, innerW - 1 - visibleWidth(name) - (isLive ? 6 : 4));
        body.push(`${nameStr}${' '.repeat(gap)}${badge}`);
      }
    }
  }

  for (let i = 0; i < bodyH; i++) {
    lines.push(row(body[i] ?? ''));
  }

  lines.push(
    row(`${fg(c.yellow)}^B${fg(c.subtext)} 侧栏  ${fg(c.yellow)}^T${fg(c.subtext)} 切换${reset}`)
  );

  return lines.slice(0, height);
}
