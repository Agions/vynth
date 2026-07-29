const VC_RE = /VC-\d{6}/;

export function parseVcCode(text: string): string | null {
  const m = VC_RE.exec(text ?? '');
  return m ? m[0] : null;
}

const HINTS: Record<string, string> = {
  'VC-010004': '检查命令是否缺参数（-g/-m/-p 后需跟值）。',
  'VC-010005': '密钥不得写入配置文件，改用环境变量 VYNTH_API_KEY。',
  'VC-010006': '配置文件 JSON / schema 非法，校验后重试。',
  'VC-020001': 'API Key 无效或过期，检查 VYNTH_API_KEY。',
  'VC-020002': '触发限流，稍后重试或降低并发。',
  'VC-020003': 'LLM 网络不可达，检查代理 / 网络 / llmBaseUrl。',
  'VC-020005': '拒绝明文 http，llmBaseUrl 改用 https。',
  'VC-030001': '路径越出授权目录，移除 ../ 越界片段或调整 cwd。',
  'VC-030002': '符号链接指向沙箱外，改用真实路径。',
  'VC-030003': '联网被 VYNTH_NET=0 阻断；需联网请 unset VYNTH_NET。',
  'VC-030006': 'OS 级隔离后端不可用：macOS 15+ 需以 root 运行，或改用 Linux bubblewrap。',
  'VC-030007': 'OS 级隔离子进程启动失败，检查 sandbox-exec / bwrap 是否安装。',
  'VC-040001': '工具未注册，确认插件已加载或工具名拼写正确。',
  'VC-040002': '工具执行失败，查看上方 output 定位原因。',
  'VC-040003': '工具参数非法，检查调用参数。',
  'VC-050001': '插件动态 import 失败，确认路径正确且为合法 ES 模块。',
  'VC-060001': '该 MCP 能力尚未实现。',
  'VC-060003': 'MCP 请求超时，确认 server 进程存活。'
};

export function errorHintFor(code: string): string {
  return HINTS[code] ?? '可重试，或查看文档与日志定位原因。';
}
