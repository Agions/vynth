import { describe, expect, it } from 'bun:test';
import { Store } from '../state/Store';
import { palette } from '../theme';
import { isMouseOrEscapeGarbage, isPhysicalEscapeKey } from '../tui-controller';

describe('Mouse & Escape Sanitization', () => {
  it('identifies and blocks mouse SGR coordinate strings', () => {
    expect(isMouseOrEscapeGarbage('35;68;24M')).toBe(true);
    expect(isMouseOrEscapeGarbage('<64;35;20M')).toBe(true);
    expect(isMouseOrEscapeGarbage('64;45;20M')).toBe(true);
    expect(isMouseOrEscapeGarbage(';24M')).toBe(true);
    expect(isMouseOrEscapeGarbage('<0;32;15m')).toBe(true);
    expect(isMouseOrEscapeGarbage('\x1b[<64;20;10M')).toBe(true);
  });

  it('allows normal user typing text', () => {
    expect(isMouseOrEscapeGarbage('hello')).toBe(false);
    expect(isMouseOrEscapeGarbage('123')).toBe(false);
    expect(isMouseOrEscapeGarbage('/model gpt-4o')).toBe(false);
    expect(isMouseOrEscapeGarbage('@src/main.ts')).toBe(false);
    expect(isMouseOrEscapeGarbage('npm run test')).toBe(false);
  });

  it('distinguishes physical Esc key from escape sequence fragments', () => {
    expect(isPhysicalEscapeKey('\x1b', { name: 'escape' })).toBe(true);
    expect(isPhysicalEscapeKey(null, { name: 'escape' })).toBe(true);
    expect(isPhysicalEscapeKey('<64;35;20M', { name: 'escape' })).toBe(false);
    expect(isPhysicalEscapeKey('35;68;24M', { name: 'escape' })).toBe(false);
  });

  it('Store.setInput automatically strips mouse garbage like 65;35;29M64;35;29M', () => {
    const store = new Store({
      input: '',
      palette: palette('mocha'),
      cols: 80
    } as unknown as ConstructorParameters<typeof Store>[0]);
    store.setInput('65;35;29M64;35;29M64;35;29M');
    expect(store.getState().input).toBe('');

    store.setInput('npm test 65;35;29M');
    expect(store.getState().input).toBe('npm test ');
  });
});
