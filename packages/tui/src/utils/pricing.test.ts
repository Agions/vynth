import { describe, expect, it } from 'bun:test';
import { estimateCost, formatCost, resolveRate } from './pricing';

describe('pricing', () => {
  it('resolves DeepSeek rate by keyword match', () => {
    const rate = resolveRate('deepseek-v4-pro');
    expect(rate.input).toBe(0.44);
    expect(rate.output).toBe(0.87);
  });

  it('resolves GPT-5.6 family rates', () => {
    expect(resolveRate('gpt-5.6-sol').input).toBe(5);
    expect(resolveRate('gpt-5.6-terra').input).toBe(2.5);
    expect(resolveRate('gpt-5.6-luna').input).toBe(1);
  });

  it('resolves Claude 5 generation rates before legacy opus', () => {
    expect(resolveRate('claude-opus-5').input).toBe(5);
    expect(resolveRate('claude-sonnet-5').output).toBe(15);
  });

  it('resolves OpenAI GPT-4o rate', () => {
    const rate = resolveRate('gpt-4o');
    expect(rate.input).toBe(2.5);
    expect(rate.output).toBe(10);
  });

  it('falls back for unknown model', () => {
    const rate = resolveRate('some-unknown-model-xyz');
    expect(rate.input).toBe(1.0);
    expect(rate.output).toBe(3.0);
  });

  it('estimates cost correctly', () => {
    // 1M input @ $0.44 + 1M output @ $0.87 = $1.31 for deepseek-v4
    const cost = estimateCost(1_000_000, 1_000_000, 'deepseek-v4-pro');
    expect(cost).toBeCloseTo(1.31, 5);
  });

  it('estimates zero cost for zero tokens', () => {
    expect(estimateCost(0, 0, 'deepseek-v4-pro')).toBe(0);
  });

  it('formats cost with appropriate precision', () => {
    expect(formatCost(0)).toBe('$0');
    expect(formatCost(0.001)).toBe('<$0.01');
    expect(formatCost(0.1234)).toBe('$0.1234');
    expect(formatCost(5.5)).toBe('$5.50');
  });
});
