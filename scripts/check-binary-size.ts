// 体积门禁脚本 — 对齐 部署设计.md §8 容量与成本 / 实施开发计划.md §1 体积门禁
// MVP 预算 <= 61MB；完整版目标 <= 40MB（U-02）。
// 用法: bun scripts/check-binary-size.ts  （可用 VYNTH_SIZE_BUDGET_MB 覆盖预算）
import { existsSync, statSync } from 'node:fs';
import { resolve } from 'node:path';

const MB = 1024 * 1024;
const DEFAULT_BUDGET_MB = 61;

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
