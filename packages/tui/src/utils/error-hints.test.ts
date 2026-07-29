import { describe, expect, it } from 'bun:test';
import { parseVcCode, errorHintFor } from './error-hints';

describe('error-hints', () => {
  it('parses VC code from arbitrary text', () => {
    expect(parseVcCode('boom [VC-030006] sandbox-exec unavailable')).toBe('VC-030006');
    expect(parseVcCode('plain message without code')).toBeNull();
  });

  it('returns actionable hint for known code', () => {
    expect(errorHintFor('VC-030006')).toContain('bubblewrap');
    expect(errorHintFor('VC-030003')).toContain('ZENO_NET');
  });

  it('falls back for unknown code', () => {
    expect(errorHintFor('VC-999999')).toContain('重试');
  });
});
