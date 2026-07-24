import type { ToolRegistry } from '../src/index';

/**
 * 示例插件：注册一个 `hello` 工具。
 * 加载方式： vynth -g "用 hello 工具向世界问好" -p packages/plugins/examples/hello-plugin.ts
 */
export const pluginName = 'hello-plugin';

export function activate(reg: ToolRegistry): void {
  reg.register({
    name: 'hello',
    description: '向指定对象问好',
    parameters: [{ name: 'name', type: 'string', description: '问候对象', required: true }],
    run: (args) => ({ ok: true, output: `你好, ${String(args.name)}!（来自 hello-plugin）` })
  });
}
