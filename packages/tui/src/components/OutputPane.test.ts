import { describe, expect, it } from 'bun:test';
import type { TuiState } from '../state/TuiState';
import { themes } from '../theme';
import { stripAnsi } from '../utils/unicode';
import { OutputPane, collectOutputLines } from './OutputPane';

function makeState(overrides: Partial<TuiState> = {}): TuiState {
  return {
    palette: themes.mocha,
    transcript: [],
    splitOpen: true,
    splitFocus: 'chat',
    outputScrollOffset: 0,
    ...overrides
  } as unknown as TuiState;
}

const toolEntry = (id: string, name: string, status: string, output?: string) => ({
  id,
  role: 'tool' as const,
  content: `${name}({})`,
  timestamp: 0,
  toolId: id,
  status,
  output
});

describe('collectOutputLines', () => {
  it('aggregates tool entries with status marks and output', () => {
    const state = makeState({
      transcript: [
        toolEntry('t1', 'read_file', 'ok', 'line1\nline2'),
        toolEntry('t2', 'run_cmd', 'error', 'VC-030006: blocked')
      ] as TuiState['transcript']
    });
    const lines = collectOutputLines(state).map(stripAnsi);
    expect(lines.some((l) => l.includes('✔ read_file'))).toBe(true);
    expect(lines.some((l) => l.includes('line2'))).toBe(true);
    expect(lines.some((l) => l.includes('✖ run_cmd'))).toBe(true);
    expect(lines.some((l) => l.includes('VC-030006'))).toBe(true);
  });

  it('ignores non-tool entries', () => {
    const state = makeState({
      transcript: [
        { id: 'm1', role: 'user', content: 'hello', timestamp: 0 }
      ] as TuiState['transcript']
    });
    expect(collectOutputLines(state).length).toBe(0);
  });
});

describe('OutputPane', () => {
  it('returns exactly height lines', () => {
    const state = makeState();
    const lines = OutputPane({ state, width: 80, height: 6 });
    expect(lines.length).toBe(6);
  });

  it('shows empty placeholder when no tool output', () => {
    const state = makeState();
    const lines = OutputPane({ state, width: 80, height: 5 }).map(stripAnsi);
    expect(lines.some((l) => l.includes('暂无工具输出'))).toBe(true);
  });

  it('marks focus state in header', () => {
    const focused = OutputPane({
      state: makeState({ splitFocus: 'output' }),
      width: 80,
      height: 5
    }).map(stripAnsi);
    expect(focused[0]).toContain('焦点在此');

    const blurred = OutputPane({ state: makeState(), width: 80, height: 5 }).map(stripAnsi);
    expect(blurred[0]).toContain('^E 聚焦');
  });

  it('windows content from the bottom and respects scroll offset', () => {
    const many = Array.from({ length: 30 }, (_, i) => toolEntry(`t${i}`, `tool_${i}`, 'ok'));
    const state = makeState({ transcript: many as TuiState['transcript'] });
    const bottom = OutputPane({ state, width: 80, height: 6 }).map(stripAnsi);
    expect(bottom.join('\n')).toContain('tool_29');

    const scrolled = OutputPane({
      state: makeState({
        transcript: many as TuiState['transcript'],
        outputScrollOffset: 999
      }),
      width: 80,
      height: 6
    }).map(stripAnsi);
    expect(scrolled.join('\n')).toContain('tool_0');
  });
});
