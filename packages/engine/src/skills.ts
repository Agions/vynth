import { existsSync, readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';

export interface SkillDef {
  name: string;
  description: string;
  instructions: string;
  source: 'builtin' | 'project';
}

const BUILTIN_SKILLS: SkillDef[] = [
  {
    name: 'a11y-debugging',
    description: '无障碍性与 Web 可访问性调试 (WCAG 2.1 规范、ARIA 标注与色彩对比度)',
    instructions: '检查 HTML5 语义标签，确保所有交互元素具备 aria-label 与键盘 Focus 环。',
    source: 'builtin'
  },
  {
    name: 'memory-leak-debugging',
    description: 'Node.js & 浏览器内存泄露排查 (Heap Snapshot 分析、闭包与事件监听器引用链追踪)',
    instructions: '分析堆快照、检查 EventListener 未解绑、闭包引用与未清理的 Timer/Interval。',
    source: 'builtin'
  },
  {
    name: 'lcp-optimization',
    description: 'Web Performance 核心指标优化 (LCP、FID/INP、CLS 渲染性能调优)',
    instructions: '分析 Largest Contentful Paint 渲染路径、图片预加载与关键 CSS 路径。',
    source: 'builtin'
  },
  {
    name: 'security-audit',
    description: '代码安全审计与漏洞排查 (OWASP Top 10、SQL 注入、XSS 与路径穿越防范)',
    instructions: '审计输入校验、过滤 SQL 与 Shell 命令拼接、防止未授权文件路径读取。',
    source: 'builtin'
  }
];

export function loadProjectSkills(cwd: string): SkillDef[] {
  const skills: SkillDef[] = [...BUILTIN_SKILLS];
  const skillsDir = join(cwd, '.zeno', 'skills');
  if (!existsSync(skillsDir)) return skills;

  try {
    const entries = readdirSync(skillsDir, { withFileTypes: true });
    for (const ent of entries) {
      if (ent.isDirectory()) {
        const skillMd = join(skillsDir, ent.name, 'SKILL.md');
        if (existsSync(skillMd)) {
          const content = readFileSync(skillMd, 'utf8');
          skills.push({
            name: ent.name,
            description: `项目自定义 Skill (${ent.name})`,
            instructions: content,
            source: 'project'
          });
        }
      }
    }
  } catch {}

  return skills;
}
