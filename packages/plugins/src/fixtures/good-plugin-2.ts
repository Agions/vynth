import type { ToolRegistry } from '@vynth/engine';

export const pluginName = 'good-plugin-2';

export function activate(reg: ToolRegistry): void {
  reg.register({
    name: 'good_tool_2',
    description: 'second tool provider for loadAll coverage',
    parameters: [{ name: 'y', type: 'string', description: 'input', required: true }],
    run: (args) => ({ ok: true, output: `good2:${args.y}` })
  });
}
