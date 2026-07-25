import { resolve } from 'node:path';
import { type Mode, loadConfig } from '@vynth/core';
import { builtinTools, createProvider, runAgent } from '@vynth/engine';
import { loadPlugin } from '@vynth/plugins';
import { startTui } from '@vynth/tui';

const VERSION = '0.1.0';

/** 把已知 VynthError 带上 6 位码前缀（其它错误原样） */
function formatErr(err: unknown): string {
  if (err && typeof err === 'object' && 'numericCode' in err && 'message' in err) {
    const e = err as { numericCode?: string; message?: string };
    if (e.numericCode && /^VC-\d{6}$/.test(e.numericCode)) {
      return `[${e.numericCode}] ${e.message ?? ''}`.trim();
    }
  }
  return err instanceof Error ? err.message : String(err);
}

interface Parsed {
  goal?: string;
  mode?: Mode;
  plugin?: string;
  help?: boolean;
  version?: boolean;
  issues: string[];
}

function parseArgs(argv: string[]): Parsed {
  const out: Parsed = { issues: [] };
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a === '-v' || a === '--version') out.version = true;
    else if (a === '-h' || a === '--help') out.help = true;
    else if (a === '-g' || a === '--goal') {
      const v = argv[++i];
      if (v === undefined) out.issues.push('[VC-010004] 缺少 -g/--goal 的目标参数');
      else out.goal = v;
    } else if (a === '-m' || a === '--mode') {
      const v = argv[++i];
      if (v === undefined) out.issues.push('[VC-010004] 缺少 -m/--mode 的模式参数');
      else if (v !== 'plan' && v !== 'vibe')
        out.issues.push(`[VC-010002] 非法模式: ${v}（应为 plan|vibe）`);
      else out.mode = v;
    } else if (a === '-p' || a === '--plugin') {
      const v = argv[++i];
      if (v === undefined) out.issues.push('[VC-010004] 缺少 -p/--plugin 的路径参数');
      else out.plugin = v;
    } else out.issues.push(`[VC-010003] 未知参数: ${a}（使用 --help 查看用法）`);
  }
  return out;
}

function printHelp(): void {
  console.log(`Vynth ${VERSION} — 你 terminal 里的代码合成器

用法:
  vynth                启动交互式 TUI（需真实终端）
  vynth -g "<目标>"   无头 agent 模式（流式输出到终端）
  vynth -m plan       指定模式 plan|vibe
  vynth -p <路径>     加载插件（示例见 packages/plugins/examples）

环境变量:
  VYNTH_API_KEY       LLM API key（必填）
  VYNTH_MODEL         模型名（默认 deepseek-v4-pro）
  VYNTH_LLM_BASE_URL  OpenAI 兼容端点
  VYNTH_MODE          plan | vibe
  VYNTH_THEME         mocha | latte
`);
}

async function runHeadless(goal: string, pluginPath?: string): Promise<void> {
  const config = loadConfig();
  const provider = createProvider(config);
  const tools = builtinTools(config.sandbox.cwd, { networkAllowed: config.sandbox.networkAllowed });
  if (pluginPath) {
    const abs = resolve(config.sandbox.cwd, pluginPath);
    const plugin = await loadPlugin(abs);
    plugin.activate(tools);
    console.log(`› 已加载插件: ${plugin.name}`);
  }
  console.log(`› ${goal}`);
  for await (const ev of runAgent(goal, { provider, tools })) {
    if (ev.type === 'token') {
      process.stdout.write(ev.text);
    } else if (ev.type === 'tool') {
      console.log(`\n  ⚙ ${ev.call.name}(${JSON.stringify(ev.call.args)})`);
    }
  }
  console.log();
}

async function main(): Promise<void> {
  try {
    const parsed = parseArgs(process.argv.slice(2));
    if (parsed.issues.length > 0) {
      for (const m of parsed.issues) console.error(`✗ ${m}`);
      process.exit(2);
    }
    if (parsed.version) {
      console.log(VERSION);
      return;
    }
    if (parsed.help) {
      printHelp();
      return;
    }
    const config = loadConfig({ mode: parsed.mode });
    if (parsed.goal) {
      await runHeadless(parsed.goal, parsed.plugin);
      return;
    }
    if (!process.stdout.isTTY || !process.stdin.isTTY) {
      console.error('当前不是交互式终端。请使用无头模式： vynth -g "<目标>"');
      process.exit(2);
    }
    startTui(config);
  } catch (err) {
    // 顶层捕获：所有 VynthError 都带 [VC-XXXXXX] 前缀输出
    console.error(`✗ ${formatErr(err)}`);
    process.exit(1);
  }
}

void main();
