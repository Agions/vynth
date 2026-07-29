import type { ToolRegistry } from '../src/index';

export const pluginName = 'hello-plugin';

export function activate(reg: ToolRegistry): void {
  reg.register({
    name: 'hello',
    description: '向指定对象问好',
    parameters: [{ name: 'name', type: 'string', description: '问候对象', required: true }],
    run: (args) => ({ ok: true, output: `你好, ${String(args.name)}!（来自 hello-plugin）` })
  });
}
