import { describe, expect, it } from 'bun:test';
import type { TuiState } from '../state/TuiState';
import { palette } from '../theme';
import type { BackgroundTask } from '../utils/tasks';
import { stripAnsi, visibleWidth } from '../utils/unicode';
import { SidePanel } from './SidePanel';

function fakeState(tab: 'files' | 'tasks' | 'tools'): TuiState {
  return {
    palette: palette('mocha'),
    sidebarTab: tab,
    sidebarOpen: true,
    spinnerFrame: 0,
    currentTool: null
  } as unknown as TuiState;
}

const WIDTH = 30;
const HEIGHT = 12;

describe('SidePanel', () => {
  it('输出行数与 height 严格相等', () => {
    const lines = SidePanel({
      state: fakeState('files'),
      width: WIDTH,
      height: HEIGHT,
      files: ['src/a.ts', 'src/b.ts'],
      tasks: [],
      toolNames: []
    });
    expect(lines).toHaveLength(HEIGHT);
  });

  it('每行可见宽度恰好等于 width（水平拼接契约）', () => {
    const lines = SidePanel({
      state: fakeState('files'),
      width: WIDTH,
      height: HEIGHT,
      files: ['src/very/deep/nested/path/that/is/really/long/file.ts'],
      tasks: [],
      toolNames: []
    });
    for (const line of lines) {
      expect(visibleWidth(line)).toBe(WIDTH);
    }
  });

  it('files tab 渲染文件列表，超出高度显示总数', () => {
    const files = Array.from({ length: 40 }, (_, i) => `src/f${i}.ts`);
    const lines = SidePanel({
      state: fakeState('files'),
      width: WIDTH,
      height: HEIGHT,
      files,
      tasks: [],
      toolNames: []
    });
    const text = lines.map(stripAnsi).join('\n');
    expect(text).toContain('f0.ts');
    expect(text).toContain('共 40 个文件');
  });

  it('tasks tab 空态给出启动提示', () => {
    const lines = SidePanel({
      state: fakeState('tasks'),
      width: WIDTH,
      height: HEIGHT,
      files: [],
      tasks: [],
      toolNames: []
    });
    const text = lines.map(stripAnsi).join('\n');
    expect(text).toContain('暂无后台任务');
  });

  it('tasks tab 渲染 running/done/failed 状态', () => {
    const tasks: BackgroundTask[] = [
      {
        id: '1',
        command: 'bun test',
        status: 'running',
        output: '',
        exitCode: null,
        startedAt: 1,
        finishedAt: null
      },
      {
        id: '2',
        command: 'ls',
        status: 'done',
        output: '',
        exitCode: 0,
        startedAt: 2,
        finishedAt: 3
      },
      {
        id: '3',
        command: 'bad',
        status: 'failed',
        output: '',
        exitCode: 1,
        startedAt: 4,
        finishedAt: 5
      }
    ];
    const lines = SidePanel({
      state: fakeState('tasks'),
      width: 40,
      height: 14,
      files: [],
      tasks,
      toolNames: []
    });
    const text = lines.map(stripAnsi).join('\n');
    expect(text).toContain('running');
    expect(text).toContain('✔ done');
    expect(text).toContain('✖ failed');
  });

  it('tools tab 渲染工具名与 on 徽标', () => {
    const lines = SidePanel({
      state: fakeState('tools'),
      width: WIDTH,
      height: HEIGHT,
      files: [],
      tasks: [],
      toolNames: ['read_file', 'edit_file', 'run_shell']
    });
    const text = lines.map(stripAnsi).join('\n');
    expect(text).toContain('read_file');
    expect(text).toContain('● on');
  });

  it('底部包含快捷键提示', () => {
    const lines = SidePanel({
      state: fakeState('files'),
      width: WIDTH,
      height: HEIGHT,
      files: [],
      tasks: [],
      toolNames: []
    });
    const text = stripAnsi(lines[lines.length - 1] ?? '');
    expect(text).toContain('^B');
    expect(text).toContain('^T');
  });
});
