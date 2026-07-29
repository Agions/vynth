export interface SlashCommand {
  name: string;
  desc: string;
  category: 'config' | 'workflow' | 'system';
  icon: string;
}

export const SLASH_COMMANDS: SlashCommand[] = [
  { name: 'config', desc: '可视化配置 AI 模型、端点与 API 密钥', category: 'config', icon: '⚙' },
  {
    name: 'model',
    desc: '一站式配置模型、Base URL 与 Key (/model <name> [url] [key])',
    category: 'config',
    icon: '🤖'
  },
  {
    name: 'theme',
    desc: '循环切换界面主题 (mocha/latte/midnight/forest)',
    category: 'config',
    icon: '🎨'
  },

  { name: 'init', desc: '初始化项目的 AGENTS.md AI 规则文件', category: 'workflow', icon: '🚀' },
  { name: 'files', desc: '开关工作区文件树抽屉', category: 'workflow', icon: '📁' },
  { name: 'search', desc: '全局内容与代码正则搜索', category: 'workflow', icon: '🔍' },
  {
    name: 'compact',
    desc: '压缩与化简上下文历史 (Context Compaction)',
    category: 'workflow',
    icon: '🗜'
  },
  { name: 'undo', desc: '回撤/恢复到上一步 AI 代码修改', category: 'workflow', icon: '↺' },

  { name: 'tokens', desc: 'Token 详细消耗与费用面板 (Ctrl+U)', category: 'system', icon: '📊' },
  { name: 'usage', desc: 'Token 用量与统计面板', category: 'system', icon: '📈' },
  { name: 'tasks', desc: '查看后台异步任务面板', category: 'system', icon: '⚡' },
  { name: 'lsp', desc: '查看 LSP 语言服务器与诊断状态', category: 'system', icon: '🩺' },
  { name: 'brief', desc: '开关折叠工具输出模式', category: 'system', icon: '📑' },
  { name: 'clear', desc: '清空当前对话与上下文历史', category: 'system', icon: '🧹' },
  { name: 'help', desc: '查看完整帮助手册与快捷键', category: 'system', icon: '❓' }
];

export function matchSlashCommands(fragment: string): string[] {
  if (!fragment.startsWith('/')) return [];
  const q = fragment.slice(1).toLowerCase();
  return SLASH_COMMANDS.filter((c) => c.name.startsWith(q)).map((c) => c.name);
}
