export interface TokenRate {
  input: number;
  output: number;
}

const RATES: Array<{ match: string[]; rate: TokenRate; label: string }> = [
  // DeepSeek (V4 generation, 2026)
  { match: ['deepseek-v4'], rate: { input: 0.44, output: 0.87 }, label: 'DeepSeek-V4' },
  { match: ['deepseek-v3'], rate: { input: 0.27, output: 1.1 }, label: 'DeepSeek' },
  { match: ['deepseek-r1'], rate: { input: 0.55, output: 2.19 }, label: 'DeepSeek-R1' },
  // OpenAI (GPT-5.6 family, 2026)
  { match: ['gpt-5.6-sol'], rate: { input: 5, output: 30 }, label: 'GPT-5.6 Sol' },
  { match: ['gpt-5.6-terra'], rate: { input: 2.5, output: 15 }, label: 'GPT-5.6 Terra' },
  { match: ['gpt-5.6-luna'], rate: { input: 1, output: 6 }, label: 'GPT-5.6 Luna' },
  { match: ['gpt-5.5'], rate: { input: 5, output: 30 }, label: 'GPT-5.5' },
  // OpenAI (legacy)
  { match: ['gpt-4o-mini'], rate: { input: 0.15, output: 0.6 }, label: 'GPT-4o mini' },
  { match: ['gpt-4o'], rate: { input: 2.5, output: 10 }, label: 'GPT-4o' },
  { match: ['gpt-4.1'], rate: { input: 2, output: 8 }, label: 'GPT-4.1' },
  // Anthropic Claude (Claude 5 / 4.x generation, 2026)
  { match: ['claude-opus-5'], rate: { input: 5, output: 25 }, label: 'Opus 5' },
  { match: ['claude-sonnet-5'], rate: { input: 3, output: 15 }, label: 'Sonnet 5' },
  { match: ['claude-fable-5'], rate: { input: 10, output: 50 }, label: 'Fable 5' },
  { match: ['claude-haiku-4-5'], rate: { input: 1, output: 5 }, label: 'Haiku 4.5' },
  // Anthropic Claude (legacy)
  {
    match: ['claude-3-5-haiku', 'claude-3.5-haiku'],
    rate: { input: 0.8, output: 4 },
    label: 'Haiku'
  },
  { match: ['claude-3-7-sonnet', 'claude-3.7'], rate: { input: 3, output: 15 }, label: 'Sonnet' },
  { match: ['claude-opus'], rate: { input: 15, output: 75 }, label: 'Opus' }
];

const FALLBACK_RATE: TokenRate = { input: 1.0, output: 3.0 };

export function resolveRate(model: string): TokenRate {
  const m = model.toLowerCase();
  for (const entry of RATES) {
    if (entry.match.some((k) => m.includes(k))) return entry.rate;
  }
  return FALLBACK_RATE;
}

export function estimateCost(inputTokens: number, outputTokens: number, model: string): number {
  const rate = resolveRate(model);
  return (inputTokens / 1_000_000) * rate.input + (outputTokens / 1_000_000) * rate.output;
}

export function formatCost(usd: number): string {
  if (usd <= 0) return '$0';
  if (usd < 0.01) return '<$0.01';
  if (usd < 1) return `$${usd.toFixed(4)}`;
  return `$${usd.toFixed(2)}`;
}
