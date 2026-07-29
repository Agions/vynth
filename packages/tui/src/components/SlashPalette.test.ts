import { describe, expect, it } from 'bun:test';
import { filterSlashCommands, SlashPalette } from './SlashPalette';
import { SLASH_COMMANDS } from '../slash-commands';
import { palette } from '../theme';
import type { TuiState } from '../state/TuiState';

describe('SlashPalette', () => {
  it('filterSlashCommands returns all commands on empty filter', () => {
    expect(filterSlashCommands('')).toHaveLength(SLASH_COMMANDS.length);
  });

  it('filterSlashCommands matches by command name (prefix/substring)', () => {
    expect(filterSlashCommands('cle')).toEqual([
      { name: 'clear', desc: '清空当前对话与上下文历史', category: 'system', icon: '🧹' }
    ]);
    expect(filterSlashCommands('CLE')).toEqual([
      { name: 'clear', desc: '清空当前对话与上下文历史', category: 'system', icon: '🧹' }
    ]);
  });

  it('filterSlashCommands matches by description', () => {
    const r = filterSlashCommands('用量');
    expect(r.some((c) => c.name === 'usage')).toBe(true);
  });

  it('filterSlashCommands returns empty when no match', () => {
    expect(filterSlashCommands('zzz')).toEqual([]);
  });

  it('SlashPalette renders without throwing and lists commands', () => {
    const fakeState = {
      palette: palette('mocha'),
      cols: 100,
      slashPaletteOpen: true,
      slashPaletteFilter: '',
      slashPaletteIndex: 0
    } as unknown as TuiState;
    const out = SlashPalette({ state: fakeState, selectedIndex: 0, filter: '' });
    expect(typeof out).toBe('string');
    for (const c of SLASH_COMMANDS.slice(0, 8)) expect(out).toContain(`/${c.name}`);
  });
});
