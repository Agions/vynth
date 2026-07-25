/**
 * 冷启动基线测量（Sprint 2 验收项：冷启动 P95 ≤ 150ms）。
 * 测量「spawn 编译后的单二进制 → 首字节 stdout」的耗时分布（first-byte latency），
 * 作为冷启动代理指标。多次采样取 P50 / P95，超限退出码 1。
 *
 * 用法：
 *   bun run compile && BENCH_RUNS=20 BENCH_LIMIT_MS=150 bun scripts/bench-cold-start.ts
 *
 * 注：v0.1.0 起 demo 模式已移除——CLI 在缺 apiKey 时会立刻抛 LlmError 并退出，
 * 但冷启动时间（启动 → 抛错前的 IO）依然稳定可测。我们注入一个 fake key 让 CLI
 * 走到真实首字节分支（goal echo 行），再让 LLM 不可达自然退出，计时不受影响。
 */
import { spawn } from 'node:child_process';
import { resolve } from 'node:path';

const REPO = resolve(import.meta.dir, '..');
const BIN = resolve(REPO, 'dist/vynth');
const RUNS = Number(process.env.BENCH_RUNS ?? 20);
const LIMIT_MS = Number(process.env.BENCH_LIMIT_MS ?? 150);

function coldMs(): Promise<number> {
  return new Promise((resolveResult, reject) => {
    const t0 = performance.now();
    const proc = spawn(BIN, ['-g', 'cold start probe'], {
      cwd: REPO,
      env: { ...process.env, VYNTH_API_KEY: 'bench-fake-key-not-used' }
    });
    let settled = false;
    const finish = (ms: number) => {
      if (settled) return;
      settled = true;
      resolveResult(ms);
    };
    // 首个 stdout / stderr 字节 ≈ 冷启动完成（CLI 在 agent 流式前会先打印目标行）。
    proc.stdout.once('data', () => finish(performance.now() - t0));
    proc.stderr.once('data', () => finish(performance.now() - t0));
    proc.on('error', (e) => {
      if (!settled) {
        settled = true;
        reject(e);
      }
    });
    proc.on('close', () => finish(performance.now() - t0));
    setTimeout(() => finish(performance.now() - t0), 30_000);
  });
}

const samples: number[] = [];
for (let i = 0; i < RUNS; i++) samples.push(await coldMs());
samples.sort((a, b) => a - b);

const pct = (q: number): number =>
  samples[Math.min(samples.length - 1, Math.floor((samples.length - 1) * q))];
const min = samples[0];
const max = samples[samples.length - 1];
const p50 = pct(0.5);
const p95 = pct(0.95);

console.log(`cold-start runs=${RUNS}`);
console.log(
  `min=${min.toFixed(1)}ms  p50=${p50.toFixed(1)}ms  p95=${p95.toFixed(1)}ms  max=${max.toFixed(1)}ms`
);

if (p95 > LIMIT_MS) {
  console.error(`✗ cold-start P95 ${p95.toFixed(1)}ms 超出 ${LIMIT_MS}ms 限制`);
  process.exit(1);
}
console.log(`✓ cold-start P95 ${p95.toFixed(1)}ms ≤ ${LIMIT_MS}ms`);
