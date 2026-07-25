import { describe, expect, it } from 'bun:test';
import {
  DEFAULT_CODE_BY_FAMILY,
  allCodes,
  describe as describeCode,
  fromLegacy,
  isVynthErrorCode
} from './error-codes';

describe('error-codes 表（VC-XXXXXX）', () => {
  it('isVynthErrorCode 接受合法 6 位码并拒绝其它', () => {
    expect(isVynthErrorCode('VC-030001')).toBe(true);
    expect(isVynthErrorCode('VC-060003')).toBe(true);
    expect(isVynthErrorCode('config')).toBe(false);
    expect(isVynthErrorCode('VC-999')).toBe(false);
    expect(isVynthErrorCode('VC-9999999')).toBe(false);
    expect(isVynthErrorCode('')).toBe(false);
  });

  it('allCodes 中每个码都通过 isVynthErrorCode 校验（防漂移）', () => {
    for (const code of allCodes()) {
      expect(isVynthErrorCode(code)).toBe(true);
      expect(code).toMatch(/^VC-\d{6}$/);
    }
  });

  it('describe() 给每个码都返回稳定的语义名（防漂移）', () => {
    expect(describeCode('VC-030001')).toBe('SANDBOX_PATH_ESCAPE');
    expect(describeCode('VC-030002')).toBe('SANDBOX_SYMLINK_ESCAPE');
    expect(describeCode('VC-030003')).toBe('SANDBOX_NETWORK_BLOCKED');
    expect(describeCode('VC-030004')).toBe('SANDBOX_READ_FAILED');
    expect(describeCode('VC-030005')).toBe('SANDBOX_WRITE_FAILED');
    expect(describeCode('VC-010003')).toBe('CONFIG_UNKNOWN_FLAG');
    expect(describeCode('VC-050001')).toBe('PLUGIN_LOAD_FAILED');
  });

  it('fromLegacy 把旧字符串族名解码为族默认 6 位码', () => {
    expect(fromLegacy('config')).toBe('VC-010099');
    expect(fromLegacy('llm')).toBe('VC-020099');
    expect(fromLegacy('sandbox')).toBe('VC-030099');
    expect(fromLegacy('tool')).toBe('VC-040099');
    expect(fromLegacy('plugin')).toBe('VC-050099');
    expect(fromLegacy('mcp')).toBe('VC-060099');
  });

  it('fromLegacy 对 6 位码原样返回', () => {
    expect(fromLegacy('VC-050002')).toBe('VC-050002');
  });

  it('fromLegacy 对未知值返回 null', () => {
    expect(fromLegacy('garbage')).toBeNull();
    expect(fromLegacy('')).toBeNull();
    expect(fromLegacy(null)).toBeNull();
    expect(fromLegacy(undefined)).toBeNull();
  });

  it('DEFAULT_CODE_BY_FAMILY 覆盖全部已知族（防漂移）', () => {
    for (const family of ['config', 'llm', 'sandbox', 'tool', 'plugin', 'mcp']) {
      expect(DEFAULT_CODE_BY_FAMILY[family as keyof typeof DEFAULT_CODE_BY_FAMILY]).toMatch(
        /^VC-\d{6}$/
      );
    }
  });
});
