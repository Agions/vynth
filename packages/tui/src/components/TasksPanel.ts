import { type Palette, fg, reset } from '../theme';
import { type BackgroundTask, getTaskManager } from '../utils/tasks';
import { padToWidth, visibleWidth } from '../utils/unicode';

export interface TasksPanelProps {
  palette: Palette;
  cols: number;
}

function statusIcon(s: BackgroundTask['status']): string {
  return s === 'running' ? '⠿' : s === 'done' ? '✔' : '✖';
}

function statusColor(s: BackgroundTask['status'], c: Palette): string {
  return s === 'running' ? c.yellow : s === 'done' ? c.green : c.red;
}

export function TasksPanel(props: TasksPanelProps): string {
  const { palette: c, cols } = props;
  const panelW = Math.min(Math.max(54, cols - 4), 84);
  const innerW = panelW - 2;
  const tasks = getTaskManager().list();
  const borderCol = fg(c.yellow);

  const running = getTaskManager().runningCount();
  const done = tasks.filter((t) => t.status === 'done').length;
  const failed = tasks.filter((t) => t.status === 'failed').length;

  const title = ' ⚡ 后台任务 ';
  const leftW = visibleWidth(title);
  const dashesW = Math.max(2, panelW - 3 - leftW);
  const topBorder = `${borderCol}╭─${fg(c.yellow)}\x1b[1m${title}${borderCol}${'─'.repeat(dashesW)}╮${reset}`;

  const lines: string[] = [topBorder];

  // Summary row
  const summaryStr =
    `  ${fg(c.yellow)}⠿ ${running} running${reset}   ` +
    `${fg(c.green)}✔ ${done} done${reset}   ` +
    `${fg(c.red)}✖ ${failed} failed${reset}`;
  lines.push(`${borderCol}│${reset}${padToWidth(summaryStr, innerW)}${borderCol}│${reset}`);
  lines.push(`${borderCol}│${reset}${padToWidth('', innerW)}${borderCol}│${reset}`);

  if (tasks.length === 0) {
    const msg = `  ${fg(c.subtext)}暂无后台任务。使用 ${fg(c.text)}! cmd${fg(c.subtext)} 启动后台 shell 命令。${reset}`;
    lines.push(`${borderCol}│${reset}${padToWidth(msg, innerW)}${borderCol}│${reset}`);
  } else {
    for (const t of tasks) {
      const col = statusColor(t.status, c);
      const icon = statusIcon(t.status);
      const exit = t.exitCode === null ? '—' : String(t.exitCode);
      const cmdStr = `${fg(c.text)}! ${t.command}${reset}`;
      const metaStr = `${fg(col)}${icon}${reset} ${fg(c.lavender)}${t.id}${reset}  ${fg(c.subtext)}exit=${exit}${reset}`;
      lines.push(`${borderCol}│${reset}${padToWidth(metaStr, innerW)}${borderCol}│${reset}`);
      lines.push(`${borderCol}│${reset}${padToWidth(`  ${cmdStr}`, innerW)}${borderCol}│${reset}`);

      const preview = t.output
        .trim()
        .split('\n')
        .slice(-2)
        .join(' ⏎ ')
        .slice(0, innerW - 6);
      if (preview) {
        const prevStr = `  ${fg(c.subtext)}${preview}${reset}`;
        lines.push(`${borderCol}│${reset}${padToWidth(prevStr, innerW)}${borderCol}│${reset}`);
      }
      lines.push(`${borderCol}│${reset}${padToWidth('', innerW)}${borderCol}│${reset}`);
    }
  }

  const footerStr = ` ${fg(c.yellow)}↑↓${fg(c.subtext)} 浏览   ${fg(c.yellow)}!cmd${fg(c.subtext)} 新任务   ${fg(c.yellow)}esc${fg(c.subtext)} 关闭 `;
  const footerW = visibleWidth(footerStr);
  const botDashW = Math.max(2, panelW - 3 - footerW);
  const botBorder = `${borderCol}╰─${footerStr}${borderCol}${'─'.repeat(botDashW)}╯${reset}`;
  lines.push(botBorder);

  return lines.join('\n');
}
