import type { Mode } from '@zeno/core';
import type { Palette } from '../theme';
import { fg, reset } from '../theme';
import { ansiBackground, hexToRgb } from '../utils/color';
import { visibleWidth } from '../utils/unicode';

const WORDMARK = [
  '███████╗███████╗███╗   ██╗ ██████╗',
  '╚══███╔╝██╔════╝████╗  ██║██╔═══██╗',
  '  ███╔╝ █████╗  ██╔██╗ ██║██║   ██║',
  ' ███╔╝  ██╔══╝  ██║╚██╗██║██║   ██║',
  '███████╗███████╗██║ ╚████║╚██████╔╝',
  '╚══════╝╚══════╝╚═╝  ╚═══╝ ╚═════╝ '
];

const MOTD: Array<{ icon: string; tip: string }> = [
  { icon: '💡', tip: '输入 / 唤出斜杠命令面板，Tab 补全命令名。' },
  { icon: '📎', tip: '输入 @ 后接文件名，将文件内容注入上下文。' },
  { icon: '⚡', tip: '! cmd 在后台运行 shell 命令，/tasks 查看状态。' },
  { icon: '🎨', tip: '/theme 循环切换主题：mocha / latte / midnight / forest。' },
  { icon: '↺', tip: '/undo 可回撤上一次 AI 操作回合（不可逆，请谨慎）。' },
  { icon: '📊', tip: '/usage 查看累计 Token 消耗与 API 费用统计。' },
  { icon: '🔍', tip: 'Ctrl+F 全局搜索对话内容，支持正则表达式。' },
  { icon: '🗂', tip: 'F2 打开文件树，@ 引用文件时自动补全路径。' },
  { icon: '⌘', tip: '? 键打开命令面板，快速发现所有快捷操作。' },
  { icon: '🤖', tip: '/model 切换 AI 模型，/url 配置自定义 API 端点。' }
];

export interface WelcomeOpts {
  version: string;
  model: string;
  mode: Mode;
  cwd: string;
  theme: string;
  palette: Palette;
  width: number;
}

export function renderWelcome(opts: WelcomeOpts): string {
  const c = opts.palette;
  const lines: string[] = [];

  lines.push('');

  const GRADIENT = [c.mauve, c.lavender, c.lavender, c.teal, c.teal, c.blue];
  for (let i = 0; i < WORDMARK.length; i++) {
    lines.push(`  ${fg(GRADIENT[i] || c.mauve)}\x1b[1m${WORDMARK[i]}${reset}`);
  }
  lines.push('');

  lines.push(
    `  ${fg(c.lavender)}\x1b[1mZeno Code Synthesizer v${opts.version}${reset}  ${fg(c.subtext)}·  Autonomous Terminal AI Agent${reset}`
  );
  lines.push('');

  lines.push(`  ${fg(c.mauve)}\x1b[1m❖ WORKSPACE CONTEXT${reset}`);
  lines.push(`  ${fg(c.subtext)}📁 Workspace  :${reset} ${fg(c.text)}${opts.cwd}${reset}`);
  lines.push(
    `  ${fg(c.subtext)}🤖 Project    :${reset} ${fg(c.green)}AGENTS.md loaded (TypeScript, Bun Monorepo)${reset}`
  );
  lines.push(
    `  ${fg(c.subtext)}🗺 Symbol Map :${reset} ${fg(c.teal)}repo-map indexed symbols across workspace${reset}`
  );
  lines.push(`  ${fg(c.subtext)}⚙ Model Engine:${reset} ${fg(c.lavender)}${opts.model}${reset}`);
  lines.push('');

  lines.push(`  ${fg(c.mauve)}\x1b[1m⌨ CONTROL & KEYBOARD SHORTCUTS${reset}`);
  const kbd = (k: string) => `${ansiBackground(c.surface0 ?? c.mantle)}${fg(c.text)} ${k} ${reset}`;
  lines.push(
    `  ${kbd('Tab')}      ${fg(c.subtext)}独立模式切换 (⚡ Vibe ⇄ 🎯 Plan ⇄ 🤖 Auto)${reset}`
  );
  lines.push(
    `  ${kbd('/model')}   ${fg(c.subtext)}一站式配置 AI 模型名、Base URL 端点与 API Key${reset}`
  );
  lines.push(`  ${kbd('/init')}    ${fg(c.subtext)}一键初始化项目的 AGENTS.md AI 规则文件${reset}`);
  lines.push(
    `  ${kbd('F2')}       ${fg(c.subtext)}开关工作区目录文件树抽屉 (File Tree Drawer)${reset}`
  );
  lines.push(
    `  ${kbd('Ctrl+F')}   ${fg(c.subtext)}全局代码与文件内容正则搜索 (Global Search)${reset}`
  );
  lines.push(`  ${kbd('Ctrl+U')}   ${fg(c.subtext)}Token 详细用量与实时费用计算面板${reset}`);
  lines.push('');

  // ── Motd ──────────────────────────────────────────────────────
  const motd = MOTD[Math.floor(Date.now() / 86_400_000) % MOTD.length];
  lines.push(`  ${fg(c.yellow)}${motd.icon}${reset}  ${fg(c.subtext)}${motd.tip}${reset}`);
  lines.push('');

  return lines.join('\n');
}

// ── Helpers ───────────────────────────────────────────────────────────────────

function padToW(str: string, target: number): string {
  const vw = visibleWidth(str);
  if (vw >= target) return str;
  return str + ' '.repeat(target - vw);
}

function centerStr(str: string, target: number): string {
  const vw = visibleWidth(str);
  const pad = Math.max(0, target - vw);
  const lpad = Math.floor(pad / 2);
  const rpad = pad - lpad;
  return ' '.repeat(lpad) + str + ' '.repeat(rpad);
}

function clipPath(p: string, max: number): string {
  if (max <= 8 || p.length <= max) return p;
  const keep = Math.floor((max - 1) / 2);
  return `${p.slice(0, keep)}…${p.slice(p.length - keep)}`;
}
