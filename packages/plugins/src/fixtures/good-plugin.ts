import type { ToolRegistry } from '@vynth/engine';

export const pluginName = 'good-plugin';

export function activate(reg: ToolRegistry): void {
  reg.register({
    name: 'good_tool',
    description: 'a tool contributed by good-plugin (headless-loadable)',
    parameters: [{ name: 'x', type: 'string', description: 'input', required: true }],
    run: (args) => ({ ok: true, output: `good:${args.x}` })
  });
}
