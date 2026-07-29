import { spawn } from 'node:child_process';
import { resolve } from 'node:path';

const REPO = resolve(import.meta.dir, '..');
const BIN = resolve(REPO, 'dist/zeno');
const RUNS = Number(process.env.BENCH_RUNS ?? 20);
const LIMIT_MS = Number(process.env.BENCH_LIMIT_MS ?? 150);

function coldMs(): Promise<number> {
  return new Promise((resolveResult, reject) => {
    const t0 = performance.now();
    const proc = spawn(BIN, ['-g', 'cold start probe'], {
      cwd: REPO,
      env: { ...process.env, ZENO_API_KEY: 'bench-fake-key-not-used' }
    });
    let settled = false;
    const finish = (ms: number) => {
      if (settled) return;
      settled = true;
      resolveResult(ms);
    };
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
