import { describe, expect, it } from 'bun:test';
import type { TuiState } from '../state/TuiState';
import { palette } from '../theme';
import { UndoModal } from './UndoModal';

describe('UndoModal', () => {
  it('UndoModal renders confirmation prompt', () => {
    const fakeState = {
      palette: palette('mocha'),
      cols: 80,
      undoModalOpen: true
    } as unknown as TuiState;

    const out = UndoModal({ state: fakeState });
    expect(typeof out).toBe('string');
    expect(out).toContain('回撤');
    expect(out).toContain('⏎ / Y');
  });
});
