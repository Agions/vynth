import { appendFileSync, existsSync, mkdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';

/**
 * F14 配置合规层 · 5 维审计。
 *
 * 审计落盘采用 append-only JSONL（零依赖、无原生模块），以兼容 ADR-0003 对单二进制的约束
 * （禁止引入无法被 bun 打包的原生 / wasm 模块）。审计记录默认不落盘，`initAudit(config)`
 * 仅在 `config.audit === true` 时启用持久化。
 */
export type AuditKind =
  | 'tool_exec' // 工具执行
  | 'file_access' // 文件访问（读 / 写）
  | 'network_egress' // 网络出站（沙箱 run_shell）
  | 'config_change' // 配置变更（配置文件加载 / 生效）
  | 'plugin_load'; // 插件加载

export interface AuditRecord {
  ts: string; // ISO 8601
  kind: AuditKind;
  ok: boolean;
  detail: Record<string, unknown>;
}

export class AuditLog {
  private readonly file: string;
  private readonly enabledFlag: boolean;

  constructor(dataDir: string, enabled: boolean) {
    this.enabledFlag = enabled;
    this.file = join(dataDir, 'audit.log');
  }

  get enabled(): boolean {
    return this.enabledFlag;
  }

  /** 追加一条审计记录（禁用时为 no-op）。文件/目录懒创建。 */
  record(kind: AuditKind, detail: Record<string, unknown>, ok = true): void {
    if (!this.enabledFlag) return;
    try {
      const dir = this.file.slice(0, this.file.lastIndexOf('/'));
      if (dir && !existsSync(dir)) mkdirSync(dir, { recursive: true });
      const rec: AuditRecord = { ts: new Date().toISOString(), kind, ok, detail };
      appendFileSync(this.file, `${JSON.stringify(rec)}\n`, 'utf8');
    } catch {
      // 审计写入失败不得影响主流程；静默忽略。
    }
  }

  /** 测试 / 调试用：读取全部记录。 */
  readAllSync(): AuditRecord[] {
    if (!existsSync(this.file)) return [];
    const text = readFileSync(this.file, 'utf8');
    return text
      .split('\n')
      .filter((l) => l.trim().length > 0)
      .map((l) => JSON.parse(l) as AuditRecord);
  }
}

// ---- 单例：避免把 AuditLog 透传到每个工具 / 沙箱签名 ----

const NOOP = new AuditLog(process.cwd(), false);
let active: AuditLog = NOOP;

/** 用配置初始化审计单例（应在 loadConfig 之后、工具执行之前调用）。 */
export function initAudit(config: { dataDir: string; audit: boolean }): AuditLog {
  active = new AuditLog(config.dataDir, config.audit);
  return active;
}

/** 取当前审计实例（未初始化时返回 no-op，record() 安全无副作用）。 */
export function audit(): AuditLog {
  return active;
}

/** 重置为 no-op（测试隔离用）。 */
export function resetAudit(): void {
  active = NOOP;
}
