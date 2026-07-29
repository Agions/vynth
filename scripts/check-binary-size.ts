import { existsSync, statSync } from 'node:fs';
import { resolve } from 'node:path';

const MB = 1024 * 1024;
const DEFAULT_BUDGET_MB = 95;

const budgetMb = Number(process.env.VYNTH_SIZE_BUDGET_MB ?? DEFAULT_BUDGET_MB);
const binaryPath = resolve(process.cwd(), 'dist', 'vynth');

if (!existsSync(binaryPath)) {
  console.error('[size-gate] binary not found at', binaryPath);
  console.error('[size-gate] run `bun run compile` before the size gate.');
  process.exit(1);
}

const bytes = statSync(binaryPath).size;
const mb = bytes / MB;
const budgetBytes = budgetMb * MB;

console.log(`[size-gate] dist/vynth = ${mb.toFixed(2)} MB (budget ${budgetMb} MB)`);

if (bytes > budgetBytes) {
  console.error(`[size-gate] FAIL: exceeds budget by ${(mb - budgetMb).toFixed(2)} MB`);
  process.exit(1);
}

console.log('[size-gate] PASS');
