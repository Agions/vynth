import { describe, expect, it } from 'bun:test';
import { SLASH_COMMANDS, matchSlashCommands } from './slash-commands';

describe('slash-commands', () => {
  it('exposes the authoritative command list', () => {
    const names = SLASH_COMMANDS.map((c) => c.name);
    expect(names).toContain('model');
    expect(names).toContain('config');
    expect(names).toContain('files');
    expect(names).toContain('files');
    expect(names).toContain('tokens');
    for (const c of SLASH_COMMANDS) expect(c.desc.length).toBeGreaterThan(0);
  });

  it('legacy commands /api /url /key are merged into /model and NOT exposed', () => {
    const names = SLASH_COMMANDS.map((c) => c.name);
    expect(names).not.toContain('api');
    expect(names).not.toContain('url');
    expect(names).not.toContain('key');
    expect(matchSlashCommands('/ap')).toEqual([]);
    expect(matchSlashCommands('/ur')).toEqual([]);
    expect(matchSlashCommands('/ke')).toEqual([]);
  });

  it('returns no candidates for non-slash input', () => {
    expect(matchSlashCommands('hello')).toEqual([]);
    expect(matchSlashCommands('')).toEqual([]);
  });

  it('matches by case-insensitive prefix', () => {
    expect(matchSlashCommands('/cle')).toEqual(['clear']);
    expect(matchSlashCommands('/CLE')).toEqual(['clear']);
    expect(matchSlashCommands('/t')).toContain('tokens');
    expect(matchSlashCommands('/')).toHaveLength(SLASH_COMMANDS.length);
  });

  it('returns empty when no command matches the fragment', () => {
    expect(matchSlashCommands('/zzz')).toEqual([]);
    expect(matchSlashCommands('/xyz')).toEqual([]);
  });

  it('does not match beyond the command name (no space-delimited args)', () => {
    // completion only operates on the command fragment, not arguments
    expect(matchSlashCommands('/clear somefile')).toEqual([]);
  });
});
