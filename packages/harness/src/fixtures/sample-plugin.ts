import type { ToolRegistry } from '@vynth/engine';

export const pluginName = 'sample-plugin';

export function activate(reg: ToolRegistry): void {
  reg.register({
    name: 'sample_tool',
    description: 'sample tool for tests',
    parameters: [{ name: 'x', type: 'number', description: 'a number', required: true }],
    run: (args) => ({ ok: true, output: `x=${args.x}` })
  });
}
