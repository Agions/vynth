import type { TuiState } from '../state/TuiState';
import { fg, reset } from '../theme';
import { hexToRgb } from '../utils/color';
import { padToWidth, visibleWidth } from '../utils/unicode';

export interface ConfigModalProps {
  state: TuiState;
}

export function ConfigModal(props: ConfigModalProps): string {
  const { state } = props;
  const c = state.palette;
  const w = state.cols;
  const panelW = Math.min(Math.max(58, w - 4), 78);
  const innerW = panelW - 2;

  const draft = state.configDraft || {
    model: 'deepseek-v4-pro',
    llmBaseUrl: 'https://api.deepseek.com/v1',
    apiKey: 'sk-***',
    theme: state.theme || 'mocha',
    networkAllowed: true
  };
  const idx = state.configFieldIndex || 0;
  const borderCol = fg(c.mauve);

  const title = ` ⚙ AI 配置中心 (模型与自定义 API) `;
  const leftW = visibleWidth(title);
  const dashesW = Math.max(2, panelW - 3 - leftW);
  const topBorder = `${borderCol}╭─${fg(c.mauve)}\x1b[1m${title}${borderCol}${'─'.repeat(dashesW)}╮${reset}`;

  const fields = [
    { label: '模型 (Model)', val: `${fg(c.teal)}\x1b[1m${draft.model}${reset}` },
    { label: '端点 (Base URL)', val: `${fg(c.blue)}${draft.llmBaseUrl}${reset}` },
    { label: '密钥 (API Key)', val: `${fg(c.subtext)}${draft.apiKey ? '•••••••• (已配置)' : '未设置'}${reset}` },
    { label: '主题 (Theme)', val: `${fg(c.mauve)}${draft.theme}${reset}` },
    { label: '联网沙箱 (Network)', val: draft.networkAllowed ? `${fg(c.green)}✔ 已开启${reset}` : `${fg(c.red)}✖ 已关闭${reset}` }
  ];

  const lines: string[] = [topBorder];
  lines.push(`${borderCol}│${reset}${padToWidth('', innerW)}${borderCol}│${reset}`);

  fields.forEach((f, i) => {
    const isSelected = i === idx;
    const cursor = isSelected ? `${fg(c.mauve)}❯${reset}` : ' ';
    const labelStr = isSelected ? `${fg(c.mauve)}\x1b[1m${f.label.padEnd(18)}${reset}` : `${fg(c.text)}${f.label.padEnd(18)}${reset}`;
    const rowContent = `  ${cursor} ${labelStr} ${f.val}`;

    let formattedRow = padToWidth(rowContent, innerW);
    if (isSelected) {
      formattedRow = `\x1b[48;2;${hexToRgb(c.surface0 || c.mantle)}m${formattedRow}${reset}`;
    }
    lines.push(`${borderCol}│${reset}${formattedRow}${borderCol}│${reset}`);
  });

  lines.push(`${borderCol}│${reset}${padToWidth('', innerW)}${borderCol}│${reset}`);

  const cmdHint = `  ${fg(c.subtext)}一站式指令: ${fg(c.teal)}/model <模型名> [端点] [密钥]${reset}`;
  lines.push(`${borderCol}│${reset}${padToWidth(cmdHint, innerW)}${borderCol}│${reset}`);

  const footerStr = ` ${fg(c.yellow)}↑↓${fg(c.subtext)} 移动   ${fg(c.yellow)}⏎/←→${fg(c.subtext)} 预设   ${fg(c.yellow)}c${fg(c.subtext)} 输入自定义   ${fg(c.yellow)}esc${fg(c.subtext)} 退出 `;
  const footerW = visibleWidth(footerStr);
  const botDashW = Math.max(2, panelW - 3 - footerW);
  const botBorder = `${borderCol}╰─${footerStr}${borderCol}${'─'.repeat(botDashW)}╯${reset}`;
  lines.push(botBorder);

  return lines.join('\n');
}
