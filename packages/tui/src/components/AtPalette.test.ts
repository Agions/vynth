import { describe, expect, it } from 'bun:test';
import { AtPalette, filterAtFiles } from './AtPalette';
import { palette } from '../theme';
import type { TuiState } from '../state/TuiState';

describe('AtPalette', () => {
  const sampleFiles = ['src/main.ts', 'src/config.ts', 'package.json', 'README.md'];

  it('filterAtFiles returns all files on empty filter', () => {
    expect(filterAtFiles(sampleFiles, '')).toHaveLength(4);
    expect(filterAtFiles(sampleFiles, '@')).toHaveLength(4);
  });

  it('filterAtFiles filters matching files case-insensitively', () => {
    expect(filterAtFiles(sampleFiles, 'main')).toEqual(['src/main.ts']);
    expect(filterAtFiles(sampleFiles, '@config')).toEqual(['src/config.ts']);
    expect(filterAtFiles(sampleFiles, 'json')).toEqual(['package.json']);
  });

  it('AtPalette renders formatted dropdown box', () => {
    const fakeState = {
      palette: palette('mocha'),
      cols: 80
    } as unknown as TuiState;
    const out = AtPalette({
      state: fakeState,
      selectedIndex: 0,
      filter: '@src',
      files: sampleFiles
    });
    expect(typeof out).toBe('string');
    expect(out).toContain('src/main.ts');
  });
});
