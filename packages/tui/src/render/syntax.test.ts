import { describe, expect, it } from 'bun:test';
import { palette } from '../theme';
import { highlightCode } from './syntax';

describe('syntax highlighting', () => {
  const p = palette('mocha');

  it('highlights python keywords and strings', () => {
    const code = 'def hello():\n    return "world"';
    const highlighted = highlightCode(code, 'python', p);
    expect(highlighted).toContain('\x1b[');
    expect(highlighted).toContain('world');
  });

  it('highlights typescript keywords and functions', () => {
    const code = 'const x: number = 42;\nfunction run() {}';
    const highlighted = highlightCode(code, 'typescript', p);
    expect(highlighted).toContain('\x1b[');
    expect(highlighted).toContain('run');
  });

  it('highlights json values', () => {
    const code = '{\n  "key": "value",\n  "count": 100\n}';
    const highlighted = highlightCode(code, 'json', p);
    expect(highlighted).toContain('\x1b[');
    expect(highlighted).toContain('key');
  });

  it('highlights markdown headers', () => {
    const code = '# Title\n- item 1';
    const highlighted = highlightCode(code, 'markdown', p);
    expect(highlighted).toContain('\x1b[');
    expect(highlighted).toContain('Title');
  });
});
